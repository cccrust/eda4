use std::collections::HashMap;

use crate::ast::{self, *};
use crate::netlist::{self, *};
use crate::parser::eval_const;

#[derive(Clone)]
struct Signal {
    bits: Vec<BitId>,
    #[allow(dead_code)]
    width: u32,
}

struct Ctx<'a> {
    net: &'a mut Netlist,
    sigs: HashMap<String, Signal>,
}

impl<'a> Ctx<'a> {
    fn resolve(&self, name: &str) -> Signal {
        self.sigs.get(name).cloned()
            .unwrap_or_else(|| panic!("訊號 '{name}' 未定義"))
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Vec<BitId> {
        match expr {
            Expr::Number(v, w) => {
                let width = if *w > 0 { *w } else { 1u32.max(64 - v.leading_zeros()) };
                let mut bits = Vec::new();
                for i in 0..width {
                    let bit_val = ((*v >> i) & 1) as u8;
                    let b = self.net.alloc_bit();
                    if bit_val == 1 {
                        self.net.add_cell(CellKind::Const1, vec![], vec![("Y", vec![b])]);
                    } else {
                        self.net.add_cell(CellKind::Const0, vec![], vec![("Y", vec![b])]);
                    }
                    bits.push(b);
                }
                bits
            }
            Expr::Ident(name) => self.resolve(name).bits,
            Expr::BitSel { base, index } => {
                let base_bits = self.resolve_expr(base);
                let idx = eval_const(index).unwrap_or(0) as usize;
                vec![base_bits[idx]]
            }
            Expr::Range { base, msb, lsb } => {
                let base_bits = self.resolve_expr(base);
                let m = eval_const(msb).unwrap_or(0) as usize;
                let l = eval_const(lsb).unwrap_or(0) as usize;
                if m >= l { base_bits[l..=m].to_vec() }
                else { base_bits[m..=l].to_vec() }
            }
            Expr::Concat(exprs) => {
                let mut bits = Vec::new();
                for e in exprs { bits.extend(self.resolve_expr(e)); }
                bits
            }
            Expr::Binary(op, lhs, rhs) => {
                let lb = self.resolve_expr(lhs);
                let rb = self.resolve_expr(rhs);
                let max_len = lb.len().max(rb.len());
                let mut la = lb;
                let mut ra = rb;
                if la.len() < max_len { let fill = *la.last().unwrap_or(&0); la.resize(max_len, fill); }
                if ra.len() < max_len { let fill = *ra.last().unwrap_or(&0); ra.resize(max_len, fill); }
                let mut out = Vec::new();
                for i in 0..max_len {
                    let b = self.net.alloc_bit();
                    let kind = match op {
                        BinOp::And => CellKind::And,
                        BinOp::Or => CellKind::Or,
                        BinOp::Xor => CellKind::Xor,
                        BinOp::Add => CellKind::Add,
                        BinOp::Sub => CellKind::Sub,
                        _ => CellKind::And,
                    };
                    self.net.add_cell(kind, vec![("A", vec![la[i]]), ("B", vec![ra[i]])], vec![("Y", vec![b])]);
                    out.push(b);
                }
                out
            }
            Expr::Unary(op, expr) => {
                let bits = self.resolve_expr(expr);
                let mut out = Vec::new();
                for &b in &bits {
                    let ob = self.net.alloc_bit();
                    match op {
                        UnaryOp::Neg | UnaryOp::Not => self.net.add_cell(CellKind::Not, vec![("A", vec![b])], vec![("Y", vec![ob])]),
                        _ => self.net.add_cell(CellKind::And, vec![("A", vec![b]), ("B", vec![b])], vec![("Y", vec![ob])]),
                    }
                    out.push(ob);
                }
                out
            }
        }
    }
}

pub fn elaborate(module: &Module) -> Netlist {
    let mut net = Netlist::new(&module.name);
    let mut sigs: HashMap<String, Signal> = HashMap::new();

    for p in &module.ports {
        let width = if let (Some(m), Some(l)) = (p.msb, p.lsb) {
            (m - l + 1) as u32
        } else { 1 };
        let bits = net.alloc_bits(width);
        for (i, &b) in bits.iter().enumerate() {
            net.name_bit(b, &format!("{}.{}", p.name, i));
        }
        sigs.insert(p.name.clone(), Signal { bits: bits.clone(), width });
        net.ports.push(NetPort {
            name: p.name.clone(),
            direction: match p.direction {
                ast::PortDir::Input => netlist::PortDir::Input,
                ast::PortDir::Output => netlist::PortDir::Output,
                ast::PortDir::Inout => netlist::PortDir::Inout,
            },
            bits: bits.clone(),
        });
    }

    let port_names: Vec<String> = module.ports.iter().map(|p| p.name.clone()).collect();

    for item in &module.items {
        match item {
            ModuleItem::Wire { name, msb, lsb } | ModuleItem::Reg { name, msb, lsb } => {
                if sigs.contains_key(name) || port_names.contains(name) { continue; }
                let width = if let (Some(m), Some(l)) = (msb, lsb) {
                    (m - l + 1) as u32
                } else { 1 };
                let bits = net.alloc_bits(width);
                sigs.insert(name.clone(), Signal { bits, width });
            }
            _ => {}
        }
    }

    let mut ctx = Ctx { net: &mut net, sigs };

    for item in &module.items {
        match item {
            ModuleItem::Assign(a) => {
                let target_bits = ctx.resolve_expr(&a.target);
                let value_bits = ctx.resolve_expr(&a.value);
                for i in 0..target_bits.len().min(value_bits.len()) {
                    let t = target_bits[i];
                    let v = value_bits[i];
                    if t != v {
                        ctx.net.add_cell(CellKind::And, vec![("A", vec![v]), ("B", vec![v])], vec![("Y", vec![t])]);
                    }
                }
            }
            ModuleItem::Always(a) => {
                let has_posedge = a.sensitivity.iter().any(|e| e.edge == Edge::Posedge);
                if !has_posedge { panic!("僅支援 posedge 觸發的 always") }
                for stmt in &a.stmts {
                    process_always_stmt(&mut ctx, stmt);
                }
            }
            _ => {}
        }
    }

    for p in &module.ports {
        if p.direction == ast::PortDir::Output || p.direction == ast::PortDir::Inout {
            if let Some(sig) = ctx.sigs.get(&p.name) {
                for &b in &sig.bits {
                    ctx.net.add_cell(CellKind::Output, vec![("A", vec![b])], vec![("Y", vec![b])]);
                }
            }
        }
        if p.direction == ast::PortDir::Input {
            if let Some(sig) = ctx.sigs.get(&p.name) {
                for &b in &sig.bits {
                    ctx.net.add_cell(CellKind::Input, vec![], vec![("Y", vec![b])]);
                }
            }
        }
    }

    drop(ctx);
    net
}

fn process_always_stmt(ctx: &mut Ctx, stmt: &Stmt) {
    match stmt {
        Stmt::Nonblocking { target, value } | Stmt::Blocking { target, value } => {
            let target_bits = ctx.resolve_expr(target);
            let value_bits = ctx.resolve_expr(value);
            for i in 0..target_bits.len().min(value_bits.len()) {
                let d = target_bits[i];
                let q = ctx.net.alloc_bit();
                ctx.net.name_bit(q, &format!("_dff_{}", d));
                ctx.net.add_cell(CellKind::Dff, vec![("D", vec![value_bits[i]]), ("C", vec![])], vec![("Q", vec![q])]);
                ctx.net.add_cell(CellKind::And, vec![("A", vec![q]), ("B", vec![q])], vec![("Y", vec![d])]);
            }
        }
        Stmt::If { cond: _, then, else_ } => {
            for stmt in then { process_always_stmt(ctx, stmt); }
            if let Some(else_) = else_ {
                for stmt in else_ { process_always_stmt(ctx, stmt); }
            }
        }
        Stmt::Block(stmts) => {
            for stmt in stmts { process_always_stmt(ctx, stmt); }
        }
    }
}

use std::collections::{HashMap, HashSet};

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
                let has_comb = a.sensitivity.iter().any(|e| e.edge == Edge::None);
                if has_comb {
                    process_combinational_always(&mut ctx, &a.stmts);
                } else if has_posedge {
                    for stmt in &a.stmts {
                        process_always_stmt(&mut ctx, stmt);
                    }
                }
                // negedge-only is unsupported; silently skip
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
        Stmt::Case { .. } => {
            panic!("case statements are only supported in combinational (always @*) blocks");
        }
    }
}

// ---------------------------------------------------------------------------
// Combinational always @(*) support
// ---------------------------------------------------------------------------

/// Process an `always @(*)` block: treat blocking assigns as wire connections,
/// case/if as MUX trees.
fn process_combinational_always(ctx: &mut Ctx, stmts: &[Stmt]) {
    // Track the current driven value for each signal.
    // Initialized to the signal's own bits (self-loop = no change).
    let mut drives: HashMap<String, Vec<BitId>> = HashMap::new();
    for (name, sig) in &ctx.sigs {
        drives.insert(name.clone(), sig.bits.clone());
    }

    for stmt in stmts {
        process_comb_stmt(ctx, stmt, &mut drives);
    }

    // Connect drives to target signal bits (buffer connection).
    for (name, drive_bits) in &drives {
        if let Some(target) = ctx.sigs.get(name) {
            for (i, &bit) in target.bits.iter().enumerate() {
                let val = *drive_bits.get(i).unwrap_or(&bit);
                if val != bit {
                    ctx.net.add_cell(
                        CellKind::And,
                        vec![("A", vec![val]), ("B", vec![val])],
                        vec![("Y", vec![bit])],
                    );
                }
            }
        }
    }
}

fn process_comb_stmt(
    ctx: &mut Ctx,
    stmt: &Stmt,
    drives: &mut HashMap<String, Vec<BitId>>,
) {
    match stmt {
        Stmt::Blocking { target, value } => {
            let target_bits = ctx.resolve_expr(target);
            let value_bits = ctx.resolve_expr(value);
            overlay_drive(ctx, target, target_bits, value_bits, drives);
        }
        Stmt::Nonblocking { .. } => {
            // Non-blocking assigns in combinational blocks are ignored
        }
        Stmt::If { cond, then, else_ } => {
            let cond_bits = ctx.resolve_expr(cond);
            let cond_bit = cond_bits[0];

            let pre = snapshot_drives(drives);

            let mut then_drives = pre.clone();
            for s in then {
                process_comb_stmt(ctx, s, &mut then_drives);
            }

            let mut else_drives = pre;
            if let Some(else_stmts) = else_ {
                for s in else_stmts {
                    process_comb_stmt(ctx, s, &mut else_drives);
                }
            }

            // MUX between then and else values
            for (name, then_bits) in &then_drives {
                let else_bits = else_drives.get(name).cloned().unwrap_or_else(|| {
                    ctx.sigs.get(name).map(|s| s.bits.clone()).unwrap_or_default()
                });
                let muxed = build_bus_mux(ctx, cond_bit, then_bits.clone(), else_bits);
                drives.insert(name.clone(), muxed);
            }
        }
        Stmt::Case { expr, items } => {
            process_case_stmt(ctx, expr, items, drives);
        }
        Stmt::Block(stmts) => {
            for s in stmts {
                process_comb_stmt(ctx, s, drives);
            }
        }
    }
}

fn snapshot_drives(drives: &HashMap<String, Vec<BitId>>) -> HashMap<String, Vec<BitId>> {
    drives.clone()
}

/// Write a target expression's bits into the drives map, handling
/// partial assignments (BitSel, Range) correctly.
fn overlay_drive(
    ctx: &Ctx,
    target: &Expr,
    target_bits: Vec<BitId>,
    value_bits: Vec<BitId>,
    drives: &mut HashMap<String, Vec<BitId>>,
) {
    let name = extract_target_name(target);
    let entry = drives.entry(name.clone()).or_insert_with(|| {
        ctx.sigs.get(&name).map(|s| s.bits.clone()).unwrap_or_default()
    });
    // Map target bit ID to position within entry, then write value bit.
    for (i, &tb) in target_bits.iter().enumerate() {
        if let Some(pos) = entry.iter().position(|&b| b == tb) {
            if let Some(&vb) = value_bits.get(i) {
                entry[pos] = vb;
            }
        }
    }
}

fn extract_target_name(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::BitSel { base, .. } => extract_target_name(base),
        Expr::Range { base, .. } => extract_target_name(base),
        Expr::Concat(exprs) => extract_target_name(&exprs[0]),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Case statement synthesis
// ---------------------------------------------------------------------------

fn process_case_stmt(
    ctx: &mut Ctx,
    expr: &Expr,
    items: &[CaseItem],
    drives: &mut HashMap<String, Vec<BitId>>,
) {
    let case_bits = ctx.resolve_expr(expr);

    // Step 1: for each item, compute match condition and statement drives
    struct ItemPrep {
        match_cond: Option<BitId>,
        item_drives: HashMap<String, Vec<BitId>>,
    }

    let mut prep: Vec<ItemPrep> = Vec::new();
    let mut default_idx: Option<usize> = None;

    for (idx, item) in items.iter().enumerate() {
        let match_cond = if item.exprs.is_empty() {
            default_idx = Some(idx);
            None
        } else {
            let mut acc: Option<BitId> = None;
            for item_expr in &item.exprs {
                let item_bits = ctx.resolve_expr(item_expr);
                let eq = build_eq_bit(ctx, &case_bits, &item_bits);
                acc = Some(match acc {
                    None => eq,
                    Some(prev) => build_or(ctx, prev, eq),
                });
            }
            acc
        };

        // Process item's statements to get local drives
        let mut item_drives = HashMap::new();
        for stmt in &item.stmts {
            process_comb_stmt(ctx, stmt, &mut item_drives);
        }

        prep.push(ItemPrep { match_cond, item_drives });
    }

    // Step 2: synthesize default match condition
    if let Some(di) = default_idx {
        let mut others: Vec<BitId> = Vec::new();
        for (j, p) in prep.iter().enumerate() {
            if j != di {
                if let Some(c) = p.match_cond {
                    others.push(c);
                }
            }
        }
        if !others.is_empty() {
            let or_all = build_or_chain(ctx, &others);
            let not_bit = ctx.net.alloc_bit();
            ctx.net.add_cell(CellKind::Not, vec![("A", vec![or_all])], vec![("Y", vec![not_bit])]);
            prep[di].match_cond = Some(not_bit);
        }
    }

    // Step 3: for each unique target, build MUX chain from items
    let all_targets: HashSet<String> = prep.iter()
        .flat_map(|p| p.item_drives.keys().cloned())
        .collect();

    for target_name in all_targets {
        let default_bits = drives.get(&target_name).cloned().unwrap_or_default();

        let mut current = default_bits;

        for (idx, p) in prep.iter().enumerate() {
            if let Some(cond) = p.match_cond {
                if let Some(item_bits) = p.item_drives.get(&target_name) {
                    if !item_bits.is_empty() {
                        current = build_bus_mux(ctx, cond, item_bits.clone(), current);
                    }
                }
            } else if let Some(item_bits) = p.item_drives.get(&target_name) {
                // default case: directly use the item value
                if !item_bits.is_empty() {
                    current = item_bits.clone();
                }
            }
        }

        drives.insert(target_name, current);
    }
}

// ---------------------------------------------------------------------------
// Helper cells
// ---------------------------------------------------------------------------

/// Build a 1-bit equality check: returns a single bit that is 1 iff all bits match.
fn build_eq_bit(ctx: &mut Ctx, a: &[BitId], b: &[BitId]) -> BitId {
    let max_len = a.len().max(b.len());
    let mut la = a.to_vec();
    let mut ra = b.to_vec();
    if la.len() < max_len { let fill = *la.last().unwrap_or(&0); la.resize(max_len, fill); }
    if ra.len() < max_len { let fill = *ra.last().unwrap_or(&0); ra.resize(max_len, fill); }

    let mut eq_bits = Vec::new();
    for i in 0..max_len {
        // xnor = not(xor)
        let x = ctx.net.alloc_bit();
        ctx.net.add_cell(CellKind::Xor, vec![("A", vec![la[i]]), ("B", vec![ra[i]])], vec![("Y", vec![x])]);
        let n = ctx.net.alloc_bit();
        ctx.net.add_cell(CellKind::Not, vec![("A", vec![x])], vec![("Y", vec![n])]);
        eq_bits.push(n);
    }

    // AND all eq_bits together
    if eq_bits.is_empty() { return ctx.net.alloc_bit(); } // will be const1
    let mut acc = eq_bits[0];
    for &b in &eq_bits[1..] {
        let o = ctx.net.alloc_bit();
        ctx.net.add_cell(CellKind::And, vec![("A", vec![acc]), ("B", vec![b])], vec![("Y", vec![o])]);
        acc = o;
    }
    acc
}

fn build_or(ctx: &mut Ctx, a: BitId, b: BitId) -> BitId {
    let o = ctx.net.alloc_bit();
    ctx.net.add_cell(CellKind::Or, vec![("A", vec![a]), ("B", vec![b])], vec![("Y", vec![o])]);
    o
}

fn build_or_chain(ctx: &mut Ctx, bits: &[BitId]) -> BitId {
    if bits.is_empty() {
        let z = ctx.net.alloc_bit();
        ctx.net.add_cell(CellKind::Const0, vec![], vec![("Y", vec![z])]);
        return z;
    }
    let mut acc = bits[0];
    for &b in &bits[1..] {
        acc = build_or(ctx, acc, b);
    }
    acc
}

/// Build a 1-bit 2-to-1 MUX: Y = sel ? B : A
fn build_mux2(ctx: &mut Ctx, sel: BitId, b: BitId, a: BitId) -> BitId {
    let o = ctx.net.alloc_bit();
    ctx.net.add_cell(CellKind::Mux2, vec![("A", vec![a]), ("B", vec![b]), ("S", vec![sel])], vec![("Y", vec![o])]);
    o
}

/// Build a multi-bit MUX: each bit is sel ? then_bits[i] : else_bits[i]
fn build_bus_mux(ctx: &mut Ctx, sel: BitId, then_bits: Vec<BitId>, else_bits: Vec<BitId>) -> Vec<BitId> {
    let max_len = then_bits.len().max(else_bits.len());
    let mut out = Vec::new();
    for i in 0..max_len {
        let t = then_bits.get(i).copied().unwrap_or(0);
        let e = else_bits.get(i).copied().unwrap_or(0);
        out.push(build_mux2(ctx, sel, t, e));
    }
    out
}

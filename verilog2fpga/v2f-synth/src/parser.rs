//! Verilog parser wrapper - uses verilog-parser internally
//!
//! This module provides a conversion layer that adapts verilog-parser's parser
//! output to v2f-synth's internal AST types.

use crate::ast::*;
use verilog_parser::parse_verilog as v2r_parse;
use verilog_parser::ast as v2r;

pub struct Parser {
    module: Module,
}

impl Parser {
    pub fn new(src: &str) -> Self {
        let v2r_modules = v2r_parse(src);
        let v2r_module = v2r_modules.into_iter().next()
            .expect("No module found in input");
        let module = convert_module(v2r_module);
        Parser { module }
    }

    pub fn parse_module(&mut self) -> Module {
        self.module.clone()
    }
}

fn convert_module(v2r_mod: v2r::Module) -> Module {
    let name = v2r_mod.name;
    let ports = v2r_mod.ports.into_iter().map(|p| {
        let (msb, lsb) = match p.width {
            Some(r) => (Some(r.msb as i64), Some(r.lsb as i64)),
            None => (None, None),
        };
        Port { name: p.name, direction: convert_port_dir(p.direction), msb, lsb }
    }).collect();
    let items = v2r_mod.items.into_iter().filter_map(convert_module_item).collect();
    Module { name, ports, items }
}

fn convert_port_dir(dir: v2r::PortDir) -> PortDir {
    match dir {
        v2r::PortDir::Input => PortDir::Input,
        v2r::PortDir::Output => PortDir::Output,
        v2r::PortDir::Inout => PortDir::Inout,
    }
}

fn convert_module_item(item: v2r::ModuleItem) -> Option<ModuleItem> {
    match item {
        v2r::ModuleItem::Wire(vd) => {
            let (msb, lsb) = match vd.width {
                Some(r) => (Some(r.msb as i64), Some(r.lsb as i64)),
                None => (None, None),
            };
            Some(ModuleItem::Wire { name: vd.name, msb, lsb })
        }
        v2r::ModuleItem::Reg(vd) => {
            let (msb, lsb) = match vd.width {
                Some(r) => (Some(r.msb as i64), Some(r.lsb as i64)),
                None => (None, None),
            };
            Some(ModuleItem::Reg { name: vd.name, msb, lsb })
        }
        v2r::ModuleItem::Assign { lhs, rhs } => {
            Some(ModuleItem::Assign(Assign {
                target: convert_expr(lhs),
                value: convert_expr(rhs),
            }))
        }
        v2r::ModuleItem::Always(ab) => {
            let sensitivity: Vec<SigEvent> = ab.sensitivity.into_iter().map(|s| {
                let edge = match s {
                    v2r::Sensitivity::Posedge(_) => Edge::Posedge,
                    v2r::Sensitivity::Negedge(_) => Edge::Negedge,
                    v2r::Sensitivity::All => Edge::None,
                };
                let signal = match s {
                    v2r::Sensitivity::Posedge(s) => s,
                    v2r::Sensitivity::Negedge(s) => s,
                    v2r::Sensitivity::All => String::new(),
                };
                SigEvent { edge, signal }
            }).collect();
            let stmts: Vec<Stmt> = ab.stmts.into_iter().filter_map(convert_stmt).collect();
            Some(ModuleItem::Always(Always { sensitivity, stmts }))
        }
        v2r::ModuleItem::ModuleInst(inst) => {
            let conns: Vec<Conn> = inst.connections.into_iter().map(|c| {
                match c {
                    v2r::Conn::ByName { port, wire } => Conn { port, expr: convert_expr(wire) },
                    v2r::Conn::ByOrder(expr) => Conn { port: String::new(), expr: convert_expr(expr) },
                }
            }).collect();
            Some(ModuleItem::Instance(Instance {
                module_name: inst.module_name,
                inst_name: inst.instance_name,
                conns,
            }))
        }
        v2r::ModuleItem::Integer(_) => None,
        v2r::ModuleItem::Initial(_) => None,
        v2r::ModuleItem::GateInst(_) => None,
    }
}

fn convert_stmt(stmt: v2r::Stmt) -> Option<Stmt> {
    match stmt {
        v2r::Stmt::BlockingAssign { lhs, rhs } => Some(Stmt::Blocking {
            target: convert_expr(lhs),
            value: convert_expr(rhs),
        }),
        v2r::Stmt::NonBlockingAssign { lhs, rhs } => Some(Stmt::Nonblocking {
            target: convert_expr(lhs),
            value: convert_expr(rhs),
        }),
        v2r::Stmt::If { cond, then, else_ } => Some(Stmt::If {
            cond: convert_expr(cond),
            then: then.into_iter().filter_map(convert_stmt).collect(),
            else_: if else_.is_empty() { None } else { Some(else_.into_iter().filter_map(convert_stmt).collect()) },
        }),
        v2r::Stmt::For { .. } => None,
        _ => None,
    }
}

fn convert_expr(expr: v2r::Expr) -> Expr {
    match expr {
        v2r::Expr::Number(nl) => {
            let width = nl.width.map(|w| w as u32).unwrap_or(32);
            Expr::Number(nl.value, width)
        }
        v2r::Expr::Ident(name) => Expr::Ident(name),
        v2r::Expr::Binary { op, lhs, rhs } => Expr::Binary(
            convert_binop(op),
            Box::new(convert_expr(*lhs)),
            Box::new(convert_expr(*rhs)),
        ),
        v2r::Expr::Unary { op, expr } => Expr::Unary(
            convert_unaryop(op),
            Box::new(convert_expr(*expr)),
        ),
        v2r::Expr::Concat(exprs) => Expr::Concat(exprs.into_iter().map(convert_expr).collect()),
        v2r::Expr::BitSelect { expr, bit } => Expr::BitSel {
            base: Box::new(convert_expr(*expr)),
            index: Box::new(convert_expr(*bit)),
        },
        v2r::Expr::Select { expr, msb, lsb } => Expr::Range {
            base: Box::new(convert_expr(*expr)),
            msb: Box::new(convert_expr(*msb)),
            lsb: Box::new(convert_expr(*lsb)),
        },
        v2r::Expr::Cond { cond, if_true: _, if_false: _ } => Expr::Binary(
            BinOp::Neq,
            Box::new(convert_expr(*cond)),
            Box::new(Expr::Number(0, 1)),
        ),
        v2r::Expr::Replicate { count, expr } => Expr::Concat(
            (0..count as usize).map(|_| convert_expr(*expr.clone())).collect()
        ),
    }
}

fn convert_binop(op: v2r::BinaryOp) -> BinOp {
    match op {
        v2r::BinaryOp::Add => BinOp::Add,
        v2r::BinaryOp::Sub => BinOp::Sub,
        v2r::BinaryOp::Mul => BinOp::Mul,
        v2r::BinaryOp::Div => BinOp::Div,
        v2r::BinaryOp::Shl => BinOp::Shl,
        v2r::BinaryOp::Shr => BinOp::Shr,
        v2r::BinaryOp::Lt => BinOp::Lt,
        v2r::BinaryOp::Leq => BinOp::Le,
        v2r::BinaryOp::Gt => BinOp::Gt,
        v2r::BinaryOp::Geq => BinOp::Ge,
        v2r::BinaryOp::Eq => BinOp::Eq,
        v2r::BinaryOp::Neq => BinOp::Neq,
        v2r::BinaryOp::BitAnd => BinOp::And,
        v2r::BinaryOp::BitXor => BinOp::Xor,
        v2r::BinaryOp::BitXnor => BinOp::Xor,
        v2r::BinaryOp::BitOr => BinOp::Or,
        _ => BinOp::And,
    }
}

fn convert_unaryop(op: v2r::UnaryOp) -> UnaryOp {
    match op {
        v2r::UnaryOp::Minus => UnaryOp::Neg,
        v2r::UnaryOp::BitNot => UnaryOp::Not,
        v2r::UnaryOp::ReduceAnd => UnaryOp::And,
        v2r::UnaryOp::ReduceOr => UnaryOp::Or,
        v2r::UnaryOp::ReduceXor => UnaryOp::Xor,
        _ => UnaryOp::Not,
    }
}

pub fn eval_const(e: &Expr) -> Option<i64> {
    match e {
        Expr::Number(v, _) => Some(*v as i64),
        Expr::Unary(UnaryOp::Neg, inner) => eval_const(inner).map(|x| -x),
        Expr::Unary(UnaryOp::Not, inner) => eval_const(inner).map(|x| !x),
        _ => None,
    }
}
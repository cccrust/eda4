use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub items: Vec<ModuleItem>,
    pub params: HashMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    pub direction: PortDir,
    pub name: String,
    pub width: Option<Range>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub msb: u64,
    pub lsb: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleItem {
    Wire(VarDecl),
    Reg(VarDecl),
    Integer(String),
    Assign { lhs: Expr, rhs: Expr },
    Always(AlwaysBlock),
    Initial(Vec<Stmt>),
    GateInst(GateInst),
    ModuleInst(ModuleInst),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub name: String,
    pub width: Option<Range>,
    pub length: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateInst {
    pub gate_type: String,
    pub instance_name: String,
    pub outputs: Vec<Expr>,
    pub inputs: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleInst {
    pub module_name: String,
    pub instance_name: String,
    pub connections: Vec<Conn>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Conn {
    ByName { port: String, wire: Expr },
    ByOrder(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysBlock {
    pub sensitivity: Vec<Sensitivity>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sensitivity {
    Posedge(String),
    Negedge(String),
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    BlockingAssign { lhs: Expr, rhs: Expr },
    NonBlockingAssign { lhs: Expr, rhs: Expr },
    If { cond: Expr, then: Vec<Stmt>, else_: Vec<Stmt> },
    Case { expr: Expr, items: Vec<CaseItem> },
    Forever { stmts: Vec<Stmt> },
    For { init: Box<Stmt>, cond: Expr, inc: Box<Stmt>, stmts: Vec<Stmt> },
    SysCall { name: String, args: Vec<Expr> },
    SysFinish,
    DelayStmt { delay: u64, stmt: Option<Box<Stmt>> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseItem {
    pub exprs: Vec<Expr>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(NumberLit),
    Ident(String),
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Concat(Vec<Expr>),
    Replicate { count: u64, expr: Box<Expr> },
    Select { expr: Box<Expr>, msb: Box<Expr>, lsb: Box<Expr> },
    BitSelect { expr: Box<Expr>, bit: Box<Expr> },
    Cond { cond: Box<Expr>, if_true: Box<Expr>, if_false: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLit {
    pub width: Option<u64>,
    pub radix: Radix,
    pub value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Mul, Div, Mod, Add, Sub,
    Shl, Shr, Sshl, Sshr,
    Lt, Leq, Gt, Geq, Eq, Neq,
    BitAnd, BitXor, BitXnor, BitOr,
    LogicalAnd, LogicalOr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus, Minus, BitNot,
    ReduceAnd, ReduceNand, ReduceOr, ReduceNor,
    ReduceXor, ReduceXnor, LogicalNot,
}

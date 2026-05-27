/// Verilog 語法樹節點定義

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: PortDir,
    pub msb: Option<i64>,
    pub lsb: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    Wire { name: String, msb: Option<i64>, lsb: Option<i64> },
    Reg { name: String, msb: Option<i64>, lsb: Option<i64> },
    Assign(Assign),
    Always(Always),
    Instance(Instance),
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub target: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Always {
    pub sensitivity: Vec<SigEvent>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct SigEvent {
    pub edge: Edge,
    pub signal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Posedge,
    Negedge,
    None,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub module_name: String,
    pub inst_name: String,
    pub conns: Vec<Conn>,
}

#[derive(Debug, Clone)]
pub struct Conn {
    pub port: String,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Blocking { target: Expr, value: Expr },
    Nonblocking { target: Expr, value: Expr },
    If { cond: Expr, then: Vec<Stmt>, else_: Option<Vec<Stmt>> },
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(u64, u32),
    Ident(String),
    Range { base: Box<Expr>, msb: Box<Expr>, lsb: Box<Expr> },
    BitSel { base: Box<Expr>, index: Box<Expr> },
    Concat(Vec<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    And, Or, Xor,
    Eq, Neq, Lt, Gt, Le, Ge,
    Shl, Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, Not, And, Or, Xor,
}

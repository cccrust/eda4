#[derive(Debug, Clone)]
pub enum HdlExpr {
    Const(u64, u32),
    Ident(String),
    Index(Box<HdlExpr>, u32),
    Range(Box<HdlExpr>, u32, u32),
    Concat(Vec<HdlExpr>),
    Add(Box<HdlExpr>, Box<HdlExpr>),
    Sub(Box<HdlExpr>, Box<HdlExpr>),
    And(Box<HdlExpr>, Box<HdlExpr>),
    Or(Box<HdlExpr>, Box<HdlExpr>),
    Xor(Box<HdlExpr>, Box<HdlExpr>),
    Not(Box<HdlExpr>),
}

#[derive(Debug, Clone)]
pub enum HdlStmt {
    Assign { target: String, value: HdlExpr },
    Blocking { target: String, value: HdlExpr },
    Nonblocking { target: String, value: HdlExpr },
    DeclReg { name: String, width: u32 },
    DeclWire { name: String, width: u32 },
}

#[derive(Debug, Clone)]
pub struct HdlPort {
    pub name: String,
    pub direction: HdlPortDir,
    pub width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdlPortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone)]
pub struct HdlModule {
    pub name: String,
    pub ports: Vec<HdlPort>,
    pub stmts: Vec<HdlStmt>,
}

impl HdlModule {
    pub fn new(name: &str) -> Self {
        HdlModule { name: name.to_string(), ports: Vec::new(), stmts: Vec::new() }
    }

    pub fn input(mut self, name: &str, width: u32) -> Self {
        self.ports.push(HdlPort { name: name.to_string(), direction: HdlPortDir::Input, width });
        self.stmts.push(HdlStmt::DeclWire { name: name.to_string(), width });
        self
    }

    pub fn output(mut self, name: &str, width: u32) -> Self {
        self.ports.push(HdlPort { name: name.to_string(), direction: HdlPortDir::Output, width });
        self.stmts.push(HdlStmt::DeclWire { name: name.to_string(), width });
        self
    }

    pub fn reg(mut self, name: &str, width: u32) -> Self {
        self.stmts.push(HdlStmt::DeclReg { name: name.to_string(), width });
        self
    }

    pub fn assign(mut self, target: &str, value: HdlExpr) -> Self {
        self.stmts.push(HdlStmt::Assign { target: target.to_string(), value });
        self
    }

    pub fn inout(mut self, name: &str, width: u32) -> Self {
        self.ports.push(HdlPort { name: name.to_string(), direction: HdlPortDir::Inout, width });
        self.stmts.push(HdlStmt::DeclWire { name: name.to_string(), width });
        self
    }

    pub fn blocking(mut self, target: &str, value: HdlExpr) -> Self {
        self.stmts.push(HdlStmt::Blocking { target: target.to_string(), value });
        self
    }

    pub fn dff(mut self, target: &str, value: HdlExpr) -> Self {
        self.stmts.push(HdlStmt::Nonblocking { target: target.to_string(), value });
        self
    }
}

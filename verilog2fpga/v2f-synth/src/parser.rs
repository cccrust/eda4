/// Verilog-2005 子集 Tokenizer + Recursive Descent Parser

use crate::ast::*;

// ── Token ──

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Module, Endmodule, Input, Output, Inout, Wire, Reg, Assign,
    Always, Posedge, Negedge, If, Else, Begin, End, Case, Endcase,
    Integer,
    Number(u64, u32),
    Ident(String),
    LParen, RParen, LBrack, RBrack, LBrace, RBrace,
    Semi, Comma, Dot, Colon, Hash, At,
    Plus, Minus, Star, Slash, Percent,
    Amp, Pipe, Caret, Tilde,
    Eq, EqEq, NotEq, Lt, Gt, Le, Ge,
    Shl, Shr, And, Or, Not,
    AssignOp,
    Eof,
}

fn tokenize(src: &str) -> Vec<(Tok, usize)> {
    let mut toks = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c.is_whitespace() { i += 1; continue; }

        // // 註解
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
            while i < bytes.len() && bytes[i] as char != '\n' { i += 1; }
            continue;
        }
        // # 註解
        if c == '#' && (i == 0 || bytes[i - 1] as char == '\n') {
            while i < bytes.len() && bytes[i] as char != '\n' { i += 1; }
            continue;
        }

        // 數字
        if c.is_digit(10) {
            let start = i;
            if i + 1 < bytes.len() && bytes[i] as char == '\'' {
                // 'd100 格式：無寬度
                i += 1;
                let base = bytes[i] as char;
                i += 1;
                let digits = read_digits(&bytes, &mut i, base);
                let val = parse_radix(&digits, base);
                toks.push((Tok::Number(val, 0), start));
            } else {
                while i < bytes.len() && (bytes[i] as char).is_digit(10) { i += 1; }
                let s = std::str::from_utf8(&bytes[start..i]).unwrap();
                let val: u64 = s.parse().unwrap();
                toks.push((Tok::Number(val, 0), start));
            }
            continue;
        }
        // 基數格式: 4'b1010, 8'hFF, 32'd100
        if c == '\'' {
            let start = i;
            i += 1;
            if i < bytes.len() {
                let base = bytes[i] as char;
                i += 1;
                let mut digits = String::new();
                while i < bytes.len() && (bytes[i] as char).is_alphanumeric() && bytes[i] as char != '\'' {
                    digits.push(bytes[i] as char);
                    i += 1;
                }
                let val = parse_radix(&digits, base);
                toks.push((Tok::Number(val, 0), start));
            }
            continue;
        }

        i += 1;

        match c {
            '(' => toks.push((Tok::LParen, i-1)),
            ')' => toks.push((Tok::RParen, i-1)),
            '[' => toks.push((Tok::LBrack, i-1)),
            ']' => toks.push((Tok::RBrack, i-1)),
            '{' => toks.push((Tok::LBrace, i-1)),
            '}' => toks.push((Tok::RBrace, i-1)),
            ';' => toks.push((Tok::Semi, i-1)),
            ',' => toks.push((Tok::Comma, i-1)),
            '.' => toks.push((Tok::Dot, i-1)),
            ':' => toks.push((Tok::Colon, i-1)),
            '@' => toks.push((Tok::At, i-1)),
            '~' => toks.push((Tok::Tilde, i-1)),
            '+' => toks.push((Tok::Plus, i-1)),
            '-' => toks.push((Tok::Minus, i-1)),
            '*' => toks.push((Tok::Star, i-1)),
            '/' => toks.push((Tok::Slash, i-1)),
            '%' => toks.push((Tok::Percent, i-1)),
            '&' => {
                if i < bytes.len() && bytes[i] as char == '&' { i += 1; toks.push((Tok::And, i-2)); }
                else { toks.push((Tok::Amp, i-1)); }
            }
            '|' => {
                if i < bytes.len() && bytes[i] as char == '|' { i += 1; toks.push((Tok::Or, i-2)); }
                else { toks.push((Tok::Pipe, i-1)); }
            }
            '^' => toks.push((Tok::Caret, i-1)),
            '!' => {
                if i < bytes.len() && bytes[i] as char == '=' { i += 1; toks.push((Tok::NotEq, i-2)); }
                else { toks.push((Tok::Not, i-1)); }
            }
            '=' => {
                if i < bytes.len() && bytes[i] as char == '=' { i += 1; toks.push((Tok::EqEq, i-2)); }
                else if i < bytes.len() && bytes[i] as char == '>' { i += 1; toks.push((Tok::AssignOp, i-2)); }
                else { toks.push((Tok::Eq, i-1)); }
            }
            '<' => {
                if i < bytes.len() && bytes[i] as char == '=' { i += 1; toks.push((Tok::Le, i-2)); }
                else if i < bytes.len() && bytes[i] as char == '<' { i += 1; toks.push((Tok::Shl, i-2)); }
                else { toks.push((Tok::Lt, i-1)); }
            }
            '>' => {
                if i < bytes.len() && bytes[i] as char == '=' { i += 1; toks.push((Tok::Ge, i-2)); }
                else if i < bytes.len() && bytes[i] as char == '>' { i += 1; toks.push((Tok::Shr, i-2)); }
                else { toks.push((Tok::Gt, i-1)); }
            }
            _ if c.is_alphabetic() || c == '_' => {
                i -= 1;
                let start = i;
                while i < bytes.len() && ((bytes[i] as char).is_alphanumeric() || bytes[i] as char == '_') { i += 1; }
                let s = std::str::from_utf8(&bytes[start..i]).unwrap();
                toks.push((match s {
                    "module" => Tok::Module, "endmodule" => Tok::Endmodule,
                    "input" => Tok::Input, "output" => Tok::Output, "inout" => Tok::Inout,
                    "wire" => Tok::Wire, "reg" => Tok::Reg,
                    "assign" => Tok::Assign, "always" => Tok::Always,
                    "posedge" => Tok::Posedge, "negedge" => Tok::Negedge,
                    "if" => Tok::If, "else" => Tok::Else,
                    "begin" => Tok::Begin, "end" => Tok::End,
                    "case" => Tok::Case, "endcase" => Tok::Endcase,
                    "integer" => Tok::Integer,
                    _ => Tok::Ident(s.to_string()),
                }, start));
            }
            _ => panic!("無法解析的字元 '{}' at pos {}", c, i-1),
        }
    }

    toks.push((Tok::Eof, bytes.len()));
    toks
}

fn read_digits(bytes: &[u8], i: &mut usize, base: char) -> String {
    let mut s = String::new();
    while *i < bytes.len() {
        let c = bytes[*i] as char;
        match base {
            'b' | 'B' => if c == '0' || c == '1' || c == '_' || c == 'x' || c == 'z' { s.push(c); *i += 1; } else { break; },
            'h' | 'H' => if c.is_ascii_hexdigit() || c == '_' || c == 'x' || c == 'z' { s.push(c); *i += 1; } else { break; },
            'd' | 'D' => if c.is_digit(10) || c == '_' { s.push(c); *i += 1; } else { break; },
            'o' | 'O' => if ('0'..='7').contains(&c) || c == '_' { s.push(c); *i += 1; } else { break; },
            _ => break,
        }
    }
    s
}

fn parse_radix(s: &str, base: char) -> u64 {
    let s: String = s.chars().filter(|c| *c != '_' && *c != 'x' && *c != 'z').collect();
    if s.is_empty() { return 0; }
    match base {
        'b' | 'B' => u64::from_str_radix(&s, 2).unwrap_or(0),
        'o' | 'O' => u64::from_str_radix(&s, 8).unwrap_or(0),
        'h' | 'H' => u64::from_str_radix(&s, 16).unwrap_or(0),
        'd' | 'D' => s.parse().unwrap_or(0),
        _ => 0,
    }
}

// ── Parser ──

pub struct Parser {
    toks: Vec<(Tok, usize)>,
    pos: usize,
}

impl Parser {
    pub fn new(src: &str) -> Self {
        let toks = tokenize(src);
        Parser { toks, pos: 0 }
    }

    fn peek(&self) -> &Tok { &self.toks[self.pos].0 }
    fn peek_nth(&self, n: usize) -> &Tok { &self.toks[self.pos + n].0 }

    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].0.clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, tok: &Tok) {
        if self.peek() == tok { self.advance(); }
        else { panic!("預期 {:?}，得到 {:?} at {}", tok, self.peek(), self.pos); }
    }

    fn expect_ident(&mut self) -> String {
        match self.advance() {
            Tok::Ident(s) => s,
            t => panic!("預期識別字，得到 {t:?} at {}", self.pos - 1),
        }
    }

    fn maybe_range(&mut self) -> (Option<i64>, Option<i64>) {
        if self.peek() == &Tok::LBrack {
            self.advance();
            let msb = self.parse_expr();
            let (msb_v, lsb_v) = if self.peek() == &Tok::Colon {
                self.advance();
                let lsb = self.parse_expr();
                (eval_const(&msb), eval_const(&lsb))
            } else {
                (eval_const(&msb), Some(0))
            };
            self.expect(&Tok::RBrack);
            (msb_v, lsb_v)
        } else {
            (None, None)
        }
    }

    // width from [msb:lsb]
    #[allow(dead_code)]
    fn range_width(msb: Option<i64>, lsb: Option<i64>) -> u32 {
        match (msb, lsb) {
            (Some(m), Some(l)) if m >= l => (m - l + 1) as u32,
            _ => 1,
        }
    }

    // ── Public entry ──
    pub fn parse_module(&mut self) -> Module {
        self.expect(&Tok::Module);
        let name = self.expect_ident();

        self.expect(&Tok::LParen);
        let mut port_names = Vec::new();
        let mut ports = Vec::new();
        let mut items = Vec::new();
        while self.peek() != &Tok::RParen {
            match self.peek() {
                Tok::Input | Tok::Output | Tok::Inout => {
                    let dir = match self.peek() {
                        Tok::Input => PortDir::Input,
                        Tok::Output => PortDir::Output,
                        Tok::Inout => PortDir::Inout,
                        _ => unreachable!(),
                    };
                    self.advance();
                    if self.peek() == &Tok::Reg || self.peek() == &Tok::Wire {
                        self.advance();
                    }
                    let (msb, lsb) = self.maybe_range();
                    if self.peek() == &Tok::Reg || self.peek() == &Tok::Wire {
                        self.advance();
                    }
                    let name = self.expect_ident();
                    port_names.push(name.clone());
                    ports.push(Port { name: name.clone(), direction: dir, msb, lsb });
                    items.push(ModuleItem::Wire { name, msb, lsb });
                }
                _ => {
                    port_names.push(self.expect_ident());
                }
            }
            if self.peek() == &Tok::Comma { self.advance(); }
        }
        self.expect(&Tok::RParen);
        self.expect(&Tok::Semi);

        loop {
            match self.peek() {
                Tok::Input | Tok::Output | Tok::Inout => {
                    let dir = match self.peek() {
                        Tok::Input => PortDir::Input,
                        Tok::Output => PortDir::Output,
                        Tok::Inout => PortDir::Inout,
                        _ => unreachable!(),
                    };
                    self.advance();
                    let (msb, lsb) = self.maybe_range();
                    let names = self.parse_id_list();
                    for n in names {
                        if !port_names.contains(&n) {
                            ports.push(Port { name: n.clone(), direction: dir, msb, lsb });
                            items.push(ModuleItem::Wire { name: n, msb, lsb });
                        }
                    }
                    self.expect(&Tok::Semi);
                }
                Tok::Wire => {
                    self.advance();
                    let (msb, lsb) = self.maybe_range();
                    let names = self.parse_id_list();
                    for n in names {
                        items.push(ModuleItem::Wire { name: n, msb, lsb });
                    }
                    self.expect(&Tok::Semi);
                }
                Tok::Reg => {
                    self.advance();
                    let (msb, lsb) = self.maybe_range();
                    let names = self.parse_id_list();
                    for n in names {
                        items.push(ModuleItem::Reg { name: n, msb, lsb });
                    }
                    self.expect(&Tok::Semi);
                }
                Tok::Assign => {
                    self.advance();
                    let target = self.parse_expr();
                    self.expect(&Tok::Eq);
                    let value = self.parse_expr();
                    items.push(ModuleItem::Assign(Assign { target, value }));
                    self.expect(&Tok::Semi);
                }
                Tok::Always => {
                    self.advance();
                    self.expect(&Tok::At);
                    self.expect(&Tok::LParen);
                    let mut sensitivity = Vec::new();
                    loop {
                        let edge = match self.peek() {
                            Tok::Posedge => { self.advance(); Edge::Posedge }
                            Tok::Negedge => { self.advance(); Edge::Negedge }
                            _ => Edge::None,
                        };
                        let signal = self.expect_ident();
                        sensitivity.push(SigEvent { edge, signal });
                        if self.peek() == &Tok::Or || self.peek() == &Tok::Comma {
                            self.advance();
                        } else { break; }
                    }
                    self.expect(&Tok::RParen);
                    let stmts = self.parse_stmt_block();
                    items.push(ModuleItem::Always(Always { sensitivity, stmts }));
                }
                Tok::Endmodule => { self.advance(); break; }
                _ => {
                    let name = self.expect_ident();
                    if self.peek() == &Tok::LParen {
                        // module instance: name inst_name ( ... );
                        // 注意：在 Verilog 中 instance 的寫法是 `module_name inst_name (conn, ...);`
                        // 但目前 name 可能是 instance name，後面跟著 ( 也可能是 expression
                        // 更嚴謹的做法要看下一個 token
                        let inst_name = name;
                        self.expect(&Tok::LParen);
                        let mut conns = Vec::new();
                        loop {
                            if self.peek() == &Tok::RParen { break; }
                            if matches!(self.peek(), Tok::Ident(_))
                                && self.peek_nth(1) == &Tok::LParen
                            {
                                let port = self.expect_ident();
                                self.expect(&Tok::LParen);
                                let expr = self.parse_expr();
                                self.expect(&Tok::RParen);
                                conns.push(Conn { port, expr });
                            } else {
                                let expr = self.parse_expr();
                                conns.push(Conn { port: String::new(), expr });
                            }
                            if self.peek() == &Tok::Comma { self.advance(); }
                        }
                        self.expect(&Tok::RParen);
                        // 正確的 instance 需要知道 module name vs instance name
                        // 此實作將第一個 name 視為 module name，用 "inst" 當 instance name
                        // 暫時簡單處理
                        items.push(ModuleItem::Instance(Instance {
                            module_name: inst_name,
                            inst_name: String::from("inst"),
                            conns,
                        }));
                        self.expect(&Tok::Semi);
                    } else {
                        panic!("未知的宣告 '{}' at {}", name, self.pos);
                    }
                }
            }
        }

        // 如果 port 宣告沒有出現在 items 中，補上
        for p in &ports {
            if !items.iter().any(|it| matches!(it, ModuleItem::Wire { name, .. } if name == &p.name)) {
                items.push(ModuleItem::Wire { name: p.name.clone(), msb: p.msb, lsb: p.lsb });
            }
        }

        Module { name, ports, items }
    }

    fn parse_id_list(&mut self) -> Vec<String> {
        let mut v = vec![self.expect_ident()];
        while self.peek() == &Tok::Comma { self.advance(); v.push(self.expect_ident()); }
        v
    }

    fn parse_stmt_block(&mut self) -> Vec<Stmt> {
        if self.peek() == &Tok::Begin {
            self.advance();
            let stmts = self.parse_stmts_until(Tok::End);
            self.expect(&Tok::End);
            stmts
        } else {
            vec![self.parse_stmt()]
        }
    }

    fn parse_stmts_until(&mut self, end: Tok) -> Vec<Stmt> {
        let mut v = Vec::new();
        while self.peek() != &end {
            if matches!(self.peek(), Tok::Endmodule | Tok::Eof) { break; }
            v.push(self.parse_stmt());
        }
        v
    }

    fn parse_stmt(&mut self) -> Stmt {
        if self.peek() == &Tok::If {
            self.advance();
            self.expect(&Tok::LParen);
            let cond = self.parse_expr();
            self.expect(&Tok::RParen);
            let then = self.parse_stmt_block();
            let else_ = if self.peek() == &Tok::Else {
                self.advance();
                Some(self.parse_stmt_block())
            } else { None };
            return Stmt::If { cond, then, else_ };
        }
        if self.peek() == &Tok::Begin {
            self.advance();
            let stmts = self.parse_stmts_until(Tok::End);
            self.expect(&Tok::End);
            return Stmt::Block(stmts);
        }
        let target = self.parse_expr_bp(10);
        match self.peek() {
            Tok::Eq => {
                self.advance();
                let value = self.parse_expr();
                self.expect(&Tok::Semi);
                Stmt::Blocking { target, value }
            }
            Tok::Le => {
                self.advance();
                let value = self.parse_expr();
                self.expect(&Tok::Semi);
                Stmt::Nonblocking { target, value }
            }
            t => panic!("預期 = 或 <=，得到 {t:?} at {}", self.pos),
        }
    }

    // ── Expression parser ──
    fn parse_expr(&mut self) -> Expr { self.parse_expr_bp(0) }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();
        loop {
            let (bp, _) = match self.peek() {
                Tok::Plus => (10, true), Tok::Minus => (10, true),
                Tok::Star => (20, true), Tok::Slash => (20, true), Tok::Percent => (20, true),
                Tok::Amp => (8, true), Tok::Pipe => (6, true), Tok::Caret => (7, true),
                Tok::EqEq => (9, true), Tok::NotEq => (9, true),
                Tok::Lt => (9, true), Tok::Gt => (9, true),
                Tok::Le => (9, true), Tok::Ge => (9, true),
                Tok::Shl => (11, true), Tok::Shr => (11, true),
                Tok::And => (3, true), Tok::Or => (2, true),
                Tok::LBrack => {
                    self.advance();
                    let index = self.parse_expr();
                    if self.peek() == &Tok::Colon {
                        self.advance();
                        let lsb = self.parse_expr();
                        self.expect(&Tok::RBrack);
                        lhs = Expr::Range { base: Box::new(lhs), msb: Box::new(index), lsb: Box::new(lsb) };
                    } else {
                        self.expect(&Tok::RBrack);
                        lhs = Expr::BitSel { base: Box::new(lhs), index: Box::new(index) };
                    }
                    continue;
                }
                _ => break,
            };
            if bp < min_bp { break; }
            let op = match self.advance() {
                Tok::Plus => BinOp::Add, Tok::Minus => BinOp::Sub,
                Tok::Star => BinOp::Mul, Tok::Slash => BinOp::Div, Tok::Percent => { unimplemented!("%") }
                Tok::Amp => BinOp::And, Tok::Pipe => BinOp::Or, Tok::Caret => BinOp::Xor,
                Tok::EqEq => BinOp::Eq, Tok::NotEq => BinOp::Neq,
                Tok::Lt => BinOp::Lt, Tok::Gt => BinOp::Gt,
                Tok::Le => BinOp::Le, Tok::Ge => BinOp::Ge,
                Tok::Shl => BinOp::Shl, Tok::Shr => BinOp::Shr,
                Tok::And => { unimplemented!("&&") }
                Tok::Or => { unimplemented!("||") }
                _ => unreachable!(),
            };
            let rhs = self.parse_expr_bp(bp + 1);
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn parse_prefix(&mut self) -> Expr {
        match self.advance() {
            Tok::Number(v, w) => Expr::Number(v, w),
            Tok::Minus => Expr::Unary(UnaryOp::Neg, Box::new(self.parse_expr_bp(30))),
            Tok::Tilde => Expr::Unary(UnaryOp::Not, Box::new(self.parse_expr_bp(30))),
            Tok::Amp => Expr::Unary(UnaryOp::And, Box::new(self.parse_expr_bp(30))),
            Tok::Pipe => Expr::Unary(UnaryOp::Or, Box::new(self.parse_expr_bp(30))),
            Tok::Caret => Expr::Unary(UnaryOp::Xor, Box::new(self.parse_expr_bp(30))),
            Tok::LBrace => {
                let mut v = Vec::new();
                loop {
                    v.push(self.parse_expr());
                    if matches!(self.peek(), Tok::Comma) { self.advance(); }
                    else { break; }
                }
                self.expect(&Tok::RBrace);
                Expr::Concat(v)
            }
            Tok::LParen => {
                let e = self.parse_expr();
                self.expect(&Tok::RParen);
                e
            }
            Tok::Ident(name) => Expr::Ident(name),
            t => panic!("預期表達式，得到 {t:?} at {}", self.pos),
        }
    }
}

pub fn eval_const(e: &Expr) -> Option<i64> {
    match e {
        Expr::Number(v, _) => Some(*v as i64),
        Expr::Unary(UnaryOp::Neg, e) => eval_const(e).map(|x| -x),
        Expr::Unary(UnaryOp::Not, e) => eval_const(e).map(|x| !x),
        _ => None,
    }
}

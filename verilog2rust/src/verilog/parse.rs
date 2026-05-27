use crate::verilog::ast::*;
use std::collections::HashMap;

// ----- Token -----

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // keywords
    Module, Endmodule, Input, Output, Inout, Wire, Reg, Integer, Parameter,
    Assign, Always, Posedge, Negedge, Begin, End,
    If, Else, Case, Endcase, Default, For, Forever,
    // operators
    Plus, Minus, Star, Slash, Percent,
    EqEq, NotEq, Lt, Leq, Gt, Geq,
    Ampersand, Bar, Caret, Tilde,
    AmpAmp, BarBar, TildeCaret, CaretTilde,
    LShift, RShift,
    Exclaim, Question, Colon,
    // delimiters
    Semicolon, Comma, Dot, LParen, RParen,
    LBracket, RBracket, LBrace, RBrace,
    At, Hash,
    // literals
    Number(NumberLit),
    Ident(String),
    // special
    AssignOp,  // = or <=
    StringLit(String),
    Initial,
}

// ----- Tokenizer -----

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut toks = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == '/' && i + 1 < chars.len() {
            if chars[i+1] == '/' { i += 2; while i < chars.len() && chars[i] != '\n' { i += 1; } continue; }
            if chars[i+1] == '*' { i += 2; while i+1 < chars.len() && !(chars[i]=='*' && chars[i+1]=='/') { i += 1; } i += 2; continue; }
        }
        if c == '\'' || c.is_ascii_digit() {
            if let Some(n) = try_sized_num(&chars, &mut i) { toks.push(Token::Number(n)); continue; }
            if let Some(n) = try_unsized_num(&chars, &mut i) { toks.push(Token::Number(n)); continue; }
            let s = i; while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            let v: u64 = chars[s..i].iter().collect::<String>().parse().unwrap();
            toks.push(Token::Number(NumberLit { width: None, radix: Radix::Decimal, value: v }));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let s = i; while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '$') { i += 1; }
            let w: String = chars[s..i].iter().collect();
            toks.push(match w.as_str() {
                "module" => Token::Module,
                "endmodule" => Token::Endmodule,
                "input" => Token::Input,
                "output" => Token::Output,
                "inout" => Token::Inout,
                "wire" => Token::Wire,
                "reg" => Token::Reg,
                "integer" => Token::Integer,
                "assign" => Token::Assign,
                "always" => Token::Always,
                "posedge" => Token::Posedge,
                "negedge" => Token::Negedge,
                "begin" => Token::Begin,
                "end" => Token::End,
                "if" => Token::If,
                "else" => Token::Else,
                "case" => Token::Case,
                "endcase" => Token::Endcase,
                "default" => Token::Default,
                "for" => Token::For,
                "forever" => Token::Forever,
                "initial" => Token::Initial,
                "parameter" => Token::Parameter,
                _ => Token::Ident(w),
            });
            continue;
        }
        // multi-char operators
        if c == '<' && i+1 < chars.len() && chars[i+1] == '=' { toks.push(Token::Leq); i += 2; continue; }
        if c == '>' && i+1 < chars.len() && chars[i+1] == '=' { toks.push(Token::Geq); i += 2; continue; }
        if c == '=' && i+1 < chars.len() && chars[i+1] == '=' { toks.push(Token::EqEq); i += 2; continue; }
        if c == '!' && i+1 < chars.len() && chars[i+1] == '=' { toks.push(Token::NotEq); i += 2; continue; }
        if c == '<' && i+1 < chars.len() && chars[i+1] == '<' { toks.push(Token::LShift); i += 2; continue; }
        if c == '>' && i+1 < chars.len() && chars[i+1] == '>' { toks.push(Token::RShift); i += 2; continue; }
        if c == '&' && i+1 < chars.len() && chars[i+1] == '&' { toks.push(Token::AmpAmp); i += 2; continue; }
        if c == '|' && i+1 < chars.len() && chars[i+1] == '|' { toks.push(Token::BarBar); i += 2; continue; }
        if c == '~' && i+1 < chars.len() && chars[i+1] == '^' { toks.push(Token::TildeCaret); i += 2; continue; }
        if c == '^' && i+1 < chars.len() && chars[i+1] == '~' { toks.push(Token::CaretTilde); i += 2; continue; }
        if c == '<' && i+1 < chars.len() && chars[i+1] == '-' { toks.push(Token::AssignOp); i += 2; continue; }
        // single-char operators/delimiters
        if c == '+' { toks.push(Token::Plus); i += 1; continue; }
        if c == '-' { toks.push(Token::Minus); i += 1; continue; }
        if c == '*' { toks.push(Token::Star); i += 1; continue; }
        if c == '/' { toks.push(Token::Slash); i += 1; continue; }
        if c == '%' { toks.push(Token::Percent); i += 1; continue; }
        if c == '&' { toks.push(Token::Ampersand); i += 1; continue; }
        if c == '|' { toks.push(Token::Bar); i += 1; continue; }
        if c == '^' { toks.push(Token::Caret); i += 1; continue; }
        if c == '~' { toks.push(Token::Tilde); i += 1; continue; }
        if c == '!' { toks.push(Token::Exclaim); i += 1; continue; }
        if c == '?' { toks.push(Token::Question); i += 1; continue; }
        if c == ':' { toks.push(Token::Colon); i += 1; continue; }
        if c == ';' { toks.push(Token::Semicolon); i += 1; continue; }
        if c == ',' { toks.push(Token::Comma); i += 1; continue; }
        if c == '.' { toks.push(Token::Dot); i += 1; continue; }
        if c == '(' { toks.push(Token::LParen); i += 1; continue; }
        if c == ')' { toks.push(Token::RParen); i += 1; continue; }
        if c == '[' { toks.push(Token::LBracket); i += 1; continue; }
        if c == ']' { toks.push(Token::RBracket); i += 1; continue; }
        if c == '{' { toks.push(Token::LBrace); i += 1; continue; }
        if c == '}' { toks.push(Token::RBrace); i += 1; continue; }
        if c == '@' { toks.push(Token::At); i += 1; continue; }
        if c == '#' { toks.push(Token::Hash); i += 1; continue; }
        if c == '<' { toks.push(Token::Lt); i += 1; continue; }
        if c == '>' { toks.push(Token::Gt); i += 1; continue; }
        if c == '=' { toks.push(Token::AssignOp); i += 1; continue; }
        if c == '"' {
            i += 1;
            let s = i;
            while i < chars.len() && chars[i] != '"' { i += 1; }
            let str: String = chars[s..i].iter().collect();
            toks.push(Token::StringLit(str));
            if i < chars.len() { i += 1; }
            continue;
        }
        panic!("unexpected char '{}' at {}", c, i);
    }
    toks
}

fn try_sized_num(chars: &[char], i: &mut usize) -> Option<NumberLit> {
    let start = *i;
    let mut j = *i;
    while j < chars.len() && chars[j].is_ascii_digit() { j += 1; }
    if j >= chars.len() || chars[j] != '\'' { return None; }
    j += 1;
    if j >= chars.len() { return None; }
    let radix = match chars[j] { 'b'|'B' => Radix::Binary, 'o'|'O' => Radix::Octal, 'd'|'D' => Radix::Decimal, 'h'|'H' => Radix::Hex, _ => return None };
    j += 1;
    let ws: String = chars[start..j-2].iter().collect();
    let width: u64 = ws.parse().ok()?;
    let vs = j;
    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_' || chars[j] == 'x' || chars[j] == 'X' || chars[j] == 'z' || chars[j] == 'Z' || chars[j] == '?') { j += 1; }
    if vs == j { return None; }
    let digs: String = chars[vs..j].iter().filter(|&&c| c != '_').collect();
    *i = j;
    Some(NumberLit { width: Some(width), radix, value: parse_radix(&digs, radix) })
}

fn try_unsized_num(chars: &[char], i: &mut usize) -> Option<NumberLit> {
    if *i >= chars.len() || chars[*i] != '\'' { return None; }
    let mut j = *i + 1;
    if j >= chars.len() { return None; }
    let radix = match chars[j] { 'b'|'B' => Radix::Binary, 'o'|'O' => Radix::Octal, 'd'|'D' => Radix::Decimal, 'h'|'H' => Radix::Hex, _ => return None };
    j += 1;
    let vs = j;
    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_' || chars[j] == 'x' || chars[j] == 'X' || chars[j] == 'z' || chars[j] == 'Z' || chars[j] == '?') { j += 1; }
    if vs == j { return None; }
    let digs: String = chars[vs..j].iter().filter(|&&c| c != '_').collect();
    *i = j;
    Some(NumberLit { width: None, radix, value: parse_radix(&digs, radix) })
}

fn parse_radix(d: &str, r: Radix) -> u64 {
    match r {
        Radix::Binary => d.chars().fold(0, |a, c| match c { '1' => (a<<1)|1, _ => a<<1 }),
        Radix::Octal => d.chars().fold(0, |a, c| if let Some(dd) = c.to_digit(8) { (a<<3)|dd as u64 } else { a<<3 }),
        Radix::Decimal => d.parse().unwrap_or(0),
        Radix::Hex => d.chars().fold(0, |a, c| if let Some(dd) = c.to_digit(16) { (a<<4)|dd as u64 } else { a<<4 }),
    }
}

// ----- Parser -----

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

macro_rules! expect {
    ($self:expr, $pat:pat) => {
        if $self.pos >= $self.toks.len() { panic!("unexpected EOF"); }
        match &$self.toks[$self.pos] {
            $pat => { $self.pos += 1; }
            _ => panic!("expected {} at pos {}, got {:?}", stringify!($pat), $self.pos, &$self.toks[$self.pos]),
        }
    };
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { toks: tokens, pos: 0 } }

    pub fn parse(&mut self) -> Vec<Module> {
        let mut mods = Vec::new();
        while self.pos < self.toks.len() {
            if self.at(Token::Module) { mods.push(self.parse_module()); }
            else { self.pos += 1; }
        }
        mods
    }

    fn parse_module(&mut self) -> Module {
        self.advance(); // skip module
        let name = self.expect_ident();
        let mut params = HashMap::new();
        if self.at(Token::Semicolon) {
            // no ports (e.g. testbench)
            self.pos += 1;
            let (port_decls, items) = self.parse_port_decls_and_body(&mut params);
            let ports: Vec<Port> = port_decls.into_iter().map(|(n, d, w)| Port { direction: d, name: n, width: w }).collect();
            return Module { name, ports, items, params };
        }
        self.expect_lparen();
        let mut port_names = Vec::new();
        if !self.at(Token::RParen) {
            loop { port_names.push(self.expect_ident()); if self.at(Token::Comma) { self.pos += 1; } else { break; } }
        }
        self.expect_rparen();
        self.expect_semi();

        let (port_decls, items) = self.parse_port_decls_and_body(&mut params);

        let ports: Vec<Port> = port_decls.into_iter().map(|(n, d, w)| Port { direction: d, name: n, width: w }).collect();
        Module { name, ports, items, params }
    }

    fn parse_port_decls_and_body(&mut self, params: &mut HashMap<String, u64>) -> (Vec<(String, PortDir, Option<Range>)>, Vec<ModuleItem>) {
        let mut port_decls: Vec<(String, PortDir, Option<Range>)> = Vec::new();
        let mut items: Vec<ModuleItem> = Vec::new();

        // read port direction decls
        loop {
            match self.toks.get(self.pos) {
                Some(Token::Input) => { self.pos += 1; let w = self.parse_range_opt(); for n in self.parse_id_list() { port_decls.push((n, PortDir::Input, w.clone())); } }
                Some(Token::Output) => {
                    self.pos += 1;
                    let is_reg = self.at(Token::Reg);
                    if is_reg { self.pos += 1; }
                    let w = self.parse_range_opt();
                    for n in self.parse_id_list() {
                        port_decls.push((n.clone(), PortDir::Output, w.clone()));
                        if is_reg { items.push(ModuleItem::Reg(VarDecl { name: n, width: w.clone(), length: None })); }
                    }
                }
                Some(Token::Inout) => { self.pos += 1; let w = self.parse_range_opt(); for n in self.parse_id_list() { port_decls.push((n, PortDir::Inout, w.clone())); } }
                _ => break,
            }
        }

        // body
        loop {
            if self.at(Token::Endmodule) || self.pos >= self.toks.len() { self.pos += 1; break; }
            match self.toks.get(self.pos) {
                Some(Token::Parameter) => {
                    self.pos += 1;
                    loop {
                        let pname = self.expect_ident();
                        self.expect_assign();
                        let val = self.parse_expr(0);
                        params.insert(pname, eval_const(&val));
                        if self.at(Token::Comma) { self.pos += 1; } else { break; }
                    }
                    self.expect_semi();
                }
                Some(Token::Wire) => {
                    self.pos += 1;
                    let w = self.parse_range_opt();
                    loop {
                        let name = self.expect_ident();
                        let length = self.parse_array_dim_opt();
                        items.push(ModuleItem::Wire(VarDecl { name, width: w.clone(), length }));
                        if self.at(Token::Comma) { self.pos += 1; } else { break; }
                    }
                    self.expect_semi();
                }
                Some(Token::Reg) => {
                    self.pos += 1;
                    let w = self.parse_range_opt();
                    loop {
                        let name = self.expect_ident();
                        let length = self.parse_array_dim_opt();
                        items.push(ModuleItem::Reg(VarDecl { name, width: w.clone(), length }));
                        if self.at(Token::Comma) { self.pos += 1; } else { break; }
                    }
                    self.expect_semi();
                }
                Some(Token::Integer) => { self.pos += 1; for n in self.parse_id_list() { items.push(ModuleItem::Integer(n)); } }
                Some(Token::Assign) => { self.pos += 1; let lhs = self.parse_expr(0); self.expect_assign(); let rhs = self.parse_expr(0); self.expect_semi(); items.push(ModuleItem::Assign { lhs, rhs }); }
                Some(Token::Always) => { self.pos += 1; items.push(ModuleItem::Always(self.parse_always())); }
                Some(Token::Initial) => { self.pos += 1; items.push(ModuleItem::Initial(self.parse_block())); }
                Some(Token::Ident(_)) => {
                    let saved = self.pos;
                    let name = self.expect_ident();
                    if is_gate_type(&name) {
                        self.pos = saved;
                        self.expect_ident();
                        items.push(ModuleItem::GateInst(self.parse_gate(name)));
                    } else if self.is_module_inst_fast() {
                        self.pos = saved;
                        items.push(self.parse_module_inst());
                    } else if self.at_token(Token::Hash) || self.at_token(Token::LParen) {
                        self.pos = saved;
                        items.push(self.parse_module_inst());
                    } else {
                        self.pos = saved;
                        self.skip_to_semi();
                    }
                }
                _ => { self.pos += 1; }
            }
        }

        (port_decls, items)
    }

    fn is_module_inst_fast(&self) -> bool {
        // after consuming first ident, check if next is ident (inst name) or #
        if self.pos >= self.toks.len() { return false; }
        match &self.toks[self.pos] {
            Token::Hash => return true,
            Token::Ident(_) => {
                // check if followed by LParen or Hash
                if self.pos + 1 < self.toks.len() {
                    matches!(&self.toks[self.pos + 1], Token::Hash | Token::LParen)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn parse_module_inst(&mut self) -> ModuleItem {
        let mod_name = self.expect_ident();
        if self.at_token(Token::Hash) {
            self.pos += 1;
            self.expect_lparen();
            let mut depth = 1;
            while depth > 0 && self.pos < self.toks.len() {
                match &self.toks[self.pos] {
                    Token::LParen => depth += 1,
                    Token::RParen => depth -= 1,
                    _ => {}
                }
                self.pos += 1;
            }
        }
        let inst_name = self.expect_ident();
        self.expect_lparen();
        let mut conns = Vec::new();
        if !self.at(Token::RParen) {
            loop {
                if self.at_token(Token::Dot) {
                    self.pos += 1;
                    let port = self.expect_ident();
                    self.expect_lparen();
                    let wire = self.parse_expr(0);
                    self.expect_rparen();
                    conns.push(Conn::ByName { port, wire });
                } else {
                    conns.push(Conn::ByOrder(self.parse_expr(0)));
                }
                if self.at(Token::Comma) { self.pos += 1; } else { break; }
            }
        }
        self.expect_rparen();
        self.expect_semi();
        ModuleItem::ModuleInst(ModuleInst { module_name: mod_name, instance_name: inst_name, connections: conns })
    }

    fn parse_gate(&mut self, gtype: String) -> GateInst {
        if self.at_token(Token::Hash) {
            self.pos += 1;
            self.expect_lparen();
            let mut depth = 1;
            while depth > 0 && self.pos < self.toks.len() {
                match &self.toks[self.pos] { Token::LParen => depth += 1, Token::RParen => depth -= 1, _ => {} }
                self.pos += 1;
            }
        }
        let inst_name = if self.at_token(Token::LParen) { String::new() } else { self.expect_ident() };
        self.expect_lparen();
        let args: Vec<Expr> = self.parse_comma_list();
        self.expect_rparen();
        self.expect_semi();
        let mut outputs = Vec::new();
        let mut inputs = Vec::new();
        if args.is_empty() { return GateInst { gate_type: gtype, instance_name: inst_name, outputs, inputs }; }
        outputs.push(args[0].clone());
        for e in &args[1..] { inputs.push(e.clone()); }
        GateInst { gate_type: gtype, instance_name: inst_name, outputs, inputs }
    }

    fn parse_comma_list(&mut self) -> Vec<Expr> {
        let mut v = Vec::new();
        if self.at(Token::RParen) { return v; }
        loop { v.push(self.parse_expr(0)); if self.at(Token::Comma) { self.pos += 1; } else { break; } }
        v
    }

    fn parse_always(&mut self) -> AlwaysBlock {
        let mut sens = Vec::new();
        if self.at_token(Token::At) {
            self.pos += 1;
            if self.at_token(Token::Star) {
                self.pos += 1;
                if self.at_token(Token::RParen) { self.pos += 1; }
                sens.push(Sensitivity::All);
            } else if self.at_token(Token::LParen) {
                self.pos += 1;
                loop {
                    if self.at_token(Token::Posedge) { self.pos += 1; sens.push(Sensitivity::Posedge(self.expect_ident())); }
                    else if self.at_token(Token::Negedge) { self.pos += 1; sens.push(Sensitivity::Negedge(self.expect_ident())); }
                    else if self.at(Token::RParen) { break; }
                    else if self.at_token(Token::Star) { self.pos += 1; sens.push(Sensitivity::All); break; }
                    else if let Token::Ident(s) = &self.toks[self.pos] {
                        if s == "or" || s == "," { self.pos += 1; continue; }
                        sens.push(Sensitivity::Posedge(self.expect_ident()));
                    } else { break; }
                }
                if self.at_token(Token::RParen) { self.pos += 1; }
            }
        }
        if sens.is_empty() { sens.push(Sensitivity::All); }

        let stmts = self.parse_block();
        AlwaysBlock { sensitivity: sens, stmts }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        if self.at_token(Token::Begin) {
            self.pos += 1;
            let mut stmts = Vec::new();
            while !self.at(Token::End) && self.pos < self.toks.len() { stmts.push(self.parse_stmt()); }
            if self.at_token(Token::End) { self.pos += 1; }
            stmts
        } else {
            vec![self.parse_stmt()]
        }
    }

    fn parse_stmt(&mut self) -> Stmt {
        // handle #delay prefix
        if self.at_token(Token::Hash) {
            self.pos += 1;
            let delay = if let Token::Number(n) = &self.toks[self.pos] {
                let v = n.value;
                self.pos += 1;
                v
            } else {
                panic!("expected delay value after #");
            };
            if self.at(Token::Semicolon) {
                self.pos += 1;
                return Stmt::DelayStmt { delay, stmt: None };
            }
            let inner = self.parse_stmt();
            return Stmt::DelayStmt { delay, stmt: Some(Box::new(inner)) };
        }
        match self.toks.get(self.pos) {
            Some(Token::If) => self.parse_if(),
            Some(Token::Case) => self.parse_case(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Forever) => { self.pos += 1; let s = self.parse_block(); Stmt::Forever { stmts: s } }
            Some(Token::Ident(s)) if s.starts_with('$') => self.parse_syscall(),
            _ => self.parse_assign_stmt(),
        }
    }

    fn parse_syscall(&mut self) -> Stmt {
        let name = self.expect_ident();
        if name == "$finish" {
            self.expect_semi();
            return Stmt::SysFinish;
        }
        // $display(...) or $monitor(...)
        self.expect_lparen();
        let mut args = Vec::new();
        if !self.at(Token::RParen) {
            loop {
        if matches!(&self.toks[self.pos], Token::StringLit(_)) {
            if let Token::StringLit(s) = &self.toks[self.pos] {
                args.push(Expr::Ident(format!("__str:{}", s)));
                self.pos += 1;
            }
                } else {
                    args.push(self.parse_expr(0));
                }
                if self.at(Token::Comma) { self.pos += 1; } else { break; }
            }
        }
        self.expect_rparen();
        self.expect_semi();
        Stmt::SysCall { name, args }
    }

    fn parse_if(&mut self) -> Stmt {
        self.pos += 1;
        self.expect_lparen();
        let cond = self.parse_expr(0);
        self.expect_rparen();
        let then = self.parse_block();
        let else_ = if self.at_token(Token::Else) { self.pos += 1; self.parse_block() } else { Vec::new() };
        Stmt::If { cond, then, else_ }
    }

    fn parse_case(&mut self) -> Stmt {
        self.pos += 1;
        self.expect_lparen();
        let expr = self.parse_expr(0);
        self.expect_rparen();
        let mut items = Vec::new();
        loop {
            if self.at(Token::Endcase) { break; }
            if self.at_token(Token::Default) {
                self.pos += 1;
                expect!(self, Token::Colon);
                items.push(CaseItem { exprs: Vec::new(), stmts: self.parse_case_stmts() });
            } else {
                let mut es = Vec::new();
                loop { es.push(self.parse_expr(0)); if self.at(Token::Comma) { self.pos += 1; } else { break; } }
                expect!(self, Token::Colon);
                items.push(CaseItem { exprs: es, stmts: self.parse_case_stmts() });
            }
        }
        expect!(self, Token::Endcase);
        Stmt::Case { expr, items }
    }

    fn parse_case_stmts(&mut self) -> Vec<Stmt> {
        let mut ss = Vec::new();
        loop {
            if self.at(Token::Endcase) || self.at_token(Token::Default) { break; }
            // peek ahead: if ident : or number : then it's a new case item
            if self.pos + 1 < self.toks.len() && matches!(&self.toks[self.pos + 1], Token::Colon) {
                if matches!(&self.toks[self.pos], Token::Ident(_) | Token::Number(_)) { break; }
            }
            if self.at_token(Token::End) { break; }
            if self.at_token(Token::Begin) { self.pos += 1; while !self.at(Token::End) && self.pos < self.toks.len() { ss.push(self.parse_stmt()); } expect!(self, Token::End); continue; }
            if self.at_token(Token::If) { ss.push(self.parse_if()); continue; }
            ss.push(self.parse_assign_stmt());
        }
        ss
    }

    fn parse_for(&mut self) -> Stmt {
        self.pos += 1;
        self.expect_lparen();
        let init = Box::new(self.parse_assign_stmt_or_empty());
        let cond = self.parse_expr(0);
        self.expect_semi();
        let inc = Box::new(self.parse_assign_stmt_or_empty());
        self.expect_rparen();
        let stmts = self.parse_block();
        Stmt::For { init, cond, inc, stmts }
    }

    fn parse_assign_stmt_or_empty(&mut self) -> Stmt {
        if self.at(Token::Semicolon) { return Stmt::BlockingAssign { lhs: Expr::Ident("".into()), rhs: Expr::Ident("".into()) }; }
        self.parse_assign_stmt()
    }

    fn parse_assign_stmt(&mut self) -> Stmt {
        let lhs = self.parse_lhs();
        if self.at_token(Token::Leq) { self.pos += 1; let rhs = self.parse_expr(0); self.expect_semi(); Stmt::NonBlockingAssign { lhs, rhs } }
        else if self.at_token(Token::AssignOp) { self.pos += 1; let rhs = self.parse_expr(0); self.expect_semi(); Stmt::BlockingAssign { lhs, rhs } }
        else { self.skip_to_semi(); Stmt::BlockingAssign { lhs: Expr::Ident("__skip__".into()), rhs: Expr::Ident("__skip__".into()) } }
    }

    fn parse_lhs(&mut self) -> Expr {
        // Parse LHS of assignment: ident, ident[bit], ident[msb:lsb], {concat}
        match self.toks.get(self.pos).cloned() {
            Some(Token::Ident(s)) => {
                self.pos += 1;
                if self.at_token(Token::LBracket) {
                    self.pos += 1;
                    let msb = self.parse_expr(0);
                    if self.at_token(Token::Colon) {
                        self.pos += 1;
                        let lsb = self.parse_expr(0);
                        expect!(self, Token::RBracket);
                        Expr::Select { expr: Box::new(Expr::Ident(s)), msb: Box::new(msb), lsb: Box::new(lsb) }
                    } else {
                        expect!(self, Token::RBracket);
                        Expr::BitSelect { expr: Box::new(Expr::Ident(s)), bit: Box::new(msb) }
                    }
                } else {
                    Expr::Ident(s)
                }
            }
            Some(Token::LBrace) => {
                self.pos += 1;
                let mut v = Vec::new();
                loop { v.push(self.parse_lhs()); if self.at(Token::Comma) { self.pos += 1; } else { break; } }
                expect!(self, Token::RBrace);
                Expr::Concat(v)
            }
            _ => self.parse_expr(0),
        }
    }

    fn skip_to_semi(&mut self) {
        while !self.at(Token::Semicolon) && self.pos < self.toks.len() { self.pos += 1; }
        if self.at_token(Token::Semicolon) { self.pos += 1; }
    }

    // ----- expression parsing -----

    fn parse_expr(&mut self, min_prec: u32) -> Expr {
        let mut lhs = self.parse_primary();
        loop {
            let tok = match self.toks.get(self.pos) { Some(t) => t.clone(), None => break };
            let (lbp, rbp) = match self.bp(&tok) { Some(p) => p, None => break };
            if lbp < min_prec { break; }
            self.pos += 1;
            if let Some(op) = self.tok_to_binop(&tok) {
                let rhs = self.parse_expr(rbp);
                lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
            }
        }
        lhs
    }

    fn parse_primary(&mut self) -> Expr {
        match self.toks.get(self.pos).cloned() {
            Some(Token::Number(n)) => { self.pos += 1; Expr::Number(n) }
            Some(Token::Ident(s)) => {
                self.pos += 1;
                if self.at_token(Token::LBracket) {
                    self.pos += 1;
                    let msb = self.parse_expr(0);
                    if self.at_token(Token::Colon) {
                        self.pos += 1;
                        let lsb = self.parse_expr(0);
                        expect!(self, Token::RBracket);
                        Expr::Select { expr: Box::new(Expr::Ident(s)), msb: Box::new(msb), lsb: Box::new(lsb) }
                    } else {
                        expect!(self, Token::RBracket);
                        Expr::BitSelect { expr: Box::new(Expr::Ident(s)), bit: Box::new(msb) }
                    }
                } else {
                    Expr::Ident(s)
                }
            }
            Some(Token::LParen) => { self.pos += 1; let e = self.parse_expr(0); expect!(self, Token::RParen); e }
            Some(Token::LBrace) => {
                self.pos += 1;
                let first = self.parse_expr(0);
                if self.at_token(Token::LBrace) {
                    self.pos += 1;
                    let body = self.parse_expr(0);
                    expect!(self, Token::RBrace);
                    expect!(self, Token::RBrace);
                    let cnt = eval_const(&first);
                    Expr::Replicate { count: cnt, expr: Box::new(body) }
                } else {
                    let mut v = vec![first];
                    loop { if self.at(Token::Comma) { self.pos += 1; v.push(self.parse_expr(0)); } else { break; } }
                    expect!(self, Token::RBrace);
                    Expr::Concat(v)
                }
            }
            Some(Token::Plus) => { self.pos += 1; Expr::Unary { op: UnaryOp::Plus, expr: Box::new(self.parse_primary()) } }
            Some(Token::Minus) => { self.pos += 1; Expr::Unary { op: UnaryOp::Minus, expr: Box::new(self.parse_primary()) } }
            Some(Token::Tilde) => { self.pos += 1; Expr::Unary { op: UnaryOp::BitNot, expr: Box::new(self.parse_primary()) } }
            Some(Token::Exclaim) => { self.pos += 1; Expr::Unary { op: UnaryOp::LogicalNot, expr: Box::new(self.parse_primary()) } }
            Some(Token::Ampersand) => { self.pos += 1; Expr::Unary { op: UnaryOp::ReduceAnd, expr: Box::new(self.parse_primary()) } }
            Some(Token::Bar) => { self.pos += 1; Expr::Unary { op: UnaryOp::ReduceOr, expr: Box::new(self.parse_primary()) } }
            Some(Token::Caret) => { self.pos += 1; Expr::Unary { op: UnaryOp::ReduceXor, expr: Box::new(self.parse_primary()) } }
            Some(Token::TildeCaret) | Some(Token::CaretTilde) => { self.pos += 1; Expr::Unary { op: UnaryOp::ReduceXnor, expr: Box::new(self.parse_primary()) } }
            Some(Token::StringLit(s)) => { let s = s.clone(); self.pos += 1; Expr::Ident(format!("__str:{}", s)) }
            Some(ref t) => panic!("unexpected token in expr at pos {}: {:?}", self.pos, t),
            None => panic!("unexpected EOF in expr"),
        }
    }

    fn bp(&self, tok: &Token) -> Option<(u32, u32)> {
        Some(match tok {
            Token::BarBar => (10, 11),
            Token::AmpAmp => (15, 16),
            Token::Bar => (20, 21),
            Token::Caret | Token::CaretTilde | Token::TildeCaret => (25, 26),
            Token::Ampersand => (30, 31),
            Token::EqEq | Token::NotEq => (35, 36),
            Token::Lt | Token::Leq | Token::Gt | Token::Geq => (40, 41),
            Token::LShift | Token::RShift => (45, 46),
            Token::Plus | Token::Minus => (50, 51),
            Token::Star | Token::Slash | Token::Percent => (55, 56),
            _ => return None,
        })
    }

    fn tok_to_binop(&self, tok: &Token) -> Option<BinaryOp> {
        Some(match tok {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Sub,
            Token::Star => BinaryOp::Mul,
            Token::Slash => BinaryOp::Div,
            Token::Percent => BinaryOp::Mod,
            Token::EqEq => BinaryOp::Eq,
            Token::NotEq => BinaryOp::Neq,
            Token::Lt => BinaryOp::Lt,
            Token::Leq => BinaryOp::Leq,
            Token::Gt => BinaryOp::Gt,
            Token::Geq => BinaryOp::Geq,
            Token::Ampersand => BinaryOp::BitAnd,
            Token::Bar => BinaryOp::BitOr,
            Token::Caret => BinaryOp::BitXor,
            Token::CaretTilde | Token::TildeCaret => BinaryOp::BitXnor,
            Token::AmpAmp => BinaryOp::LogicalAnd,
            Token::BarBar => BinaryOp::LogicalOr,
            Token::LShift => BinaryOp::Shl,
            Token::RShift => BinaryOp::Shr,
            _ => return None,
        })
    }

    fn parse_range_opt(&mut self) -> Option<Range> {
        if self.at_token(Token::LBracket) {
            self.pos += 1;
            let msb = self.parse_expr(0);
            expect!(self, Token::Colon);
            let lsb = self.parse_expr(0);
            expect!(self, Token::RBracket);
            Some(Range { msb: eval_const(&msb), lsb: eval_const(&lsb) })
        } else { None }
    }

    fn parse_array_dim_opt(&mut self) -> Option<u64> {
        if self.at_token(Token::LBracket) {
            self.pos += 1;
            let msb = self.parse_expr(0);
            if self.at_token(Token::Colon) {
                self.pos += 1;
                let lsb = self.parse_expr(0);
                self.expect_rbracket();
                let m = eval_const(&msb);
                let l = eval_const(&lsb);
                Some(if m >= l { m - l + 1 } else { l - m + 1 })
            } else {
                self.expect_rbracket();
                Some(eval_const(&msb) + 1)
            }
        } else { None }
    }

    fn parse_id_list(&mut self) -> Vec<String> {
        let mut v = Vec::new();
        loop { v.push(self.expect_ident()); if self.at(Token::Comma) { self.pos += 1; } else { break; } }
        self.expect_semi();
        v
    }

    // helpers
    fn at(&self, tok: Token) -> bool {
        self.toks.get(self.pos).map_or(false, |t| *t == tok)
    }

    fn at_token(&self, tok: Token) -> bool {
        if self.pos >= self.toks.len() { return false; }
        std::mem::discriminant(&self.toks[self.pos]) == std::mem::discriminant(&tok)
    }

    fn advance(&mut self) { self.pos += 1; }
    fn expect_ident(&mut self) -> String {
        match &self.toks[self.pos] { Token::Ident(s) => { let s = s.clone(); self.pos += 1; s } _ => panic!("expected ident") }
    }
    fn expect_lparen(&mut self) { expect!(self, Token::LParen); }
    fn expect_rparen(&mut self) { expect!(self, Token::RParen); }
    fn expect_semi(&mut self) { expect!(self, Token::Semicolon); }
    fn expect_rbracket(&mut self) { expect!(self, Token::RBracket); }
    fn expect_assign(&mut self) { expect!(self, Token::AssignOp); }
}

fn is_gate_type(s: &str) -> bool {
    matches!(s, "and" | "nand" | "or" | "nor" | "xor" | "xnor" | "not" | "buf" | "bufif0" | "bufif1" | "notif0" | "notif1")
}

pub(crate) fn eval_const(e: &Expr) -> u64 {
    match e {
        Expr::Number(n) => n.value,
        Expr::Binary { op: BinaryOp::Add, lhs, rhs } => eval_const(lhs) + eval_const(rhs),
        Expr::Binary { op: BinaryOp::Sub, lhs, rhs } => eval_const(lhs).wrapping_sub(eval_const(rhs)),
        Expr::Binary { op: BinaryOp::Mul, lhs, rhs } => eval_const(lhs) * eval_const(rhs),
        Expr::Unary { op: UnaryOp::Minus, expr } => (!eval_const(expr)).wrapping_add(1),
        _ => 0,
    }
}

pub fn parse_verilog(input: &str) -> Vec<Module> {
    let toks = tokenize(input);
    let mut p = Parser::new(toks);
    p.parse()
}

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TS, Span};
use quote::quote;

/// `fpga!` 巨集：在 Rust 中直接撰寫硬體描述
///
/// # 語法範例
/// ```ignore
/// let blinky = fpga! {
///     module Blinky {
///         input clk: 1,
///         output led: 4,
///
///         reg [25:0] counter,
///
///         always(posedge clk) {
///             counter <= counter + 1;
///         }
///
///         assign led = counter[25:22];
///     }
/// };
/// ```
#[proc_macro]
pub fn fpga(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    match parse_fpga(&input_str) {
        Ok(tokens) => tokens.into(),
        Err(msg) => {
            let err = syn::Error::new(Span::call_site(), msg);
            err.to_compile_error().into()
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

fn parse_fpga(input: &str) -> Result<TS, String> {
    let mut p = Cursor::new(input);
    p.skip_ws_and_comma();
    p.expect("module")?;
    let name = p.ident()?;
    p.expect("{")?;

    let mut ports: Vec<PortDecl> = Vec::new();
    let mut regs: Vec<RegDecl> = Vec::new();
    let mut always_blocks: Vec<AlwaysBlock> = Vec::new();
    let mut assigns: Vec<Assign> = Vec::new();

    loop {
        p.skip_ws_and_comma();
        if p.at_end() || p.peek() == Some('}') {
            break;
        }
        if p.check("input") || p.check("output") || p.check("inout") {
            ports.push(parse_port_decl(&mut p)?);
        } else if p.check("reg") {
            regs.push(parse_reg_decl(&mut p)?);
        } else if p.check("always") {
            always_blocks.push(parse_always(&mut p)?);
        } else if p.check("assign") {
            assigns.push(parse_assign_stmt(&mut p)?);
        } else {
            return Err(format!("位置 {}: 無效的宣告 '{}'", p.pos, p.chars.iter().skip(p.pos).take(20).collect::<String>()));
        }
    }
    p.expect("}")?;

    Ok(generate_code(&name, &ports, &regs, &always_blocks, &assigns))
}

// -- data types -------------------------------------------------------------

#[allow(dead_code)]
struct PortDecl { dir: String, name: String, width: u32 }
#[allow(dead_code)]
struct RegDecl { name: String, width: u32 }
#[allow(dead_code)]
struct AlwaysBlock { edge: String, sensitivity: String, stmts: Vec<HdlStmt> }
#[allow(dead_code)]
struct Assign { target: String, value: Expr }

enum HdlStmt {
    #[allow(dead_code)]
    Blocking { target: String, value: Expr },
    Nonblocking { target: String, value: Expr },
}

enum Expr {
    Const(u64, u32),
    Ident(String),
    Index(Box<Expr>, u32),
    Range(Box<Expr>, u32, u32),
    #[allow(dead_code)]
    Concat(Vec<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    #[allow(dead_code)]
    And(Box<Expr>, Box<Expr>),
    #[allow(dead_code)]
    Or(Box<Expr>, Box<Expr>),
    #[allow(dead_code)]
    Xor(Box<Expr>, Box<Expr>),
    #[allow(dead_code)]
    Not(Box<Expr>),
}

// -- port -------------------------------------------------------------------

fn parse_port_decl(p: &mut Cursor) -> Result<PortDecl, String> {
    let dir = p.ident()?;
    let name = p.ident()?;
    p.expect(":")?;
    let width = p.number()? as u32;
    Ok(PortDecl { dir, name, width })
}

// -- reg --------------------------------------------------------------------

fn parse_reg_decl(p: &mut Cursor) -> Result<RegDecl, String> {
    p.expect("reg")?;
    let width = if p.check("[") {
        p.expect("[")?;
        let msb = p.number()?;
        p.expect(":")?;
        let lsb = p.number()?;
        p.expect("]")?;
        (msb - lsb + 1) as u32
    } else {
        1
    };
    let name = p.ident()?;
    Ok(RegDecl { name, width })
}

// -- always -----------------------------------------------------------------

fn parse_always(p: &mut Cursor) -> Result<AlwaysBlock, String> {
    p.expect("always")?;
    p.expect("(")?;
    let edge = p.ident()?;
    let sensitivity = p.ident()?;
    p.expect(")")?;
    p.expect("{")?;

    let mut stmts = Vec::new();
    loop {
        p.skip_ws_and_comma();
        if p.peek() == Some('}') { break; }
        stmts.push(parse_stmt(p)?);
    }
    p.expect("}")?;

    Ok(AlwaysBlock { edge, sensitivity, stmts })
}

fn parse_stmt(p: &mut Cursor) -> Result<HdlStmt, String> {
    let target = p.ident()?;
    let stmt = if p.check("<=") {
        p.eat_str("<=");
        let value = parse_expr(p, 0)?;
        HdlStmt::Nonblocking { target, value }
    } else if p.check("=") {
        p.eat_char('=');
        let value = parse_expr(p, 0)?;
        HdlStmt::Blocking { target, value }
    } else {
        return Err(format!("位置 {}: 預期 '=' 或 '<='", p.pos));
    };
    p.eat_char(';');
    Ok(stmt)
}

// -- assign -----------------------------------------------------------------

fn parse_assign_stmt(p: &mut Cursor) -> Result<Assign, String> {
    p.expect("assign")?;
    let target = p.ident()?;
    p.expect("=")?;
    let value = parse_expr(p, 0)?;
    p.eat_char(';');
    Ok(Assign { target, value })
}

// -- expression -------------------------------------------------------------

fn parse_expr(p: &mut Cursor, min_prec: u32) -> Result<Expr, String> {
    let mut lhs = parse_primary(p)?;
    loop {
        p.skip_ws();
        let op = match p.peek() {
            Some('+') if prec("+", p) >= min_prec => { p.eat_char('+'); Op::Add }
            Some('-') if prec("-", p) >= min_prec => { p.eat_char('-'); Op::Sub }
            Some('&') if prec("&", p) >= min_prec => { p.eat_char('&'); Op::And }
            Some('|') if prec("|", p) >= min_prec => { p.eat_char('|'); Op::Or }
            Some('^') if prec("^", p) >= min_prec => { p.eat_char('^'); Op::Xor }
            _ => break,
        };
        let next_min = min_prec + 1;
        let rhs = parse_expr(p, next_min)?;
        lhs = match op {
            Op::Add => Expr::Add(Box::new(lhs), Box::new(rhs)),
            Op::Sub => Expr::Sub(Box::new(lhs), Box::new(rhs)),
            Op::And => Expr::And(Box::new(lhs), Box::new(rhs)),
            Op::Or  => Expr::Or(Box::new(lhs), Box::new(rhs)),
            Op::Xor => Expr::Xor(Box::new(lhs), Box::new(rhs)),
        };
    }
    Ok(lhs)
}

enum Op { Add, Sub, And, Or, Xor }

fn prec(op: &str, _p: &Cursor) -> u32 {
    match op {
        "+" | "-" => 10,
        "&" | "|" | "^" => 5,
        _ => 0,
    }
}

fn parse_primary(p: &mut Cursor) -> Result<Expr, String> {
    p.skip_ws();
    if p.peek() == Some('(') {
        p.eat_char('(');
        let e = parse_expr(p, 0)?;
        p.expect(")")?;
        return Ok(e);
    }
    if p.check("~") {
        p.eat_char('~');
        let inner = parse_primary(p)?;
        return Ok(Expr::Not(Box::new(inner)));
    }
    if p.peek().map_or(false, |c| c.is_ascii_digit()) {
        let n = p.number()?;
        let w = if n == 0 { 1 } else { 64 - n.leading_zeros() };
        return Ok(Expr::Const(n, w.max(1) as u32));
    }
    let name = p.ident()?;
    p.skip_ws();
    if p.peek() == Some('[') {
        p.eat_char('[');
        let idx = p.number()?;
        if p.peek() == Some(':') {
            p.eat_char(':');
            let lsb = p.number()?;
            p.expect("]")?;
            Ok(Expr::Range(Box::new(Expr::Ident(name)), idx as u32, lsb as u32))
        } else {
            p.expect("]")?;
            Ok(Expr::Index(Box::new(Expr::Ident(name)), idx as u32))
        }
    } else {
        Ok(Expr::Ident(name))
    }
}

// -- cursor (string parser) -------------------------------------------------

struct Cursor<'a> {
    chars: Vec<char>,
    pos: usize,
    #[allow(dead_code)]
    _src: &'a str,
}

impl<'a> Cursor<'a> {
    fn new(src: &'a str) -> Self {
        Cursor { chars: src.chars().collect(), pos: 0, _src: src }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_ws_and_comma(&mut self) {
        loop {
            self.skip_ws();
            if self.pos < self.chars.len() && self.chars[self.pos] == ',' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() { Some(self.chars[self.pos]) } else { None }
    }

    fn check(&self, s: &str) -> bool {
        let mut p = self.pos;
        let sc: Vec<char> = s.chars().collect();
        while p < self.chars.len() && self.chars[p].is_ascii_whitespace() { p += 1; }
        if p + sc.len() > self.chars.len() { return false; }
        for (i, &c) in sc.iter().enumerate() {
            if self.chars[p + i] != c { return false; }
        }
        if sc[0].is_ascii_alphabetic() || sc[0] == '_' {
            if p + sc.len() < self.chars.len() {
                let next = self.chars[p + sc.len()];
                if next.is_ascii_alphanumeric() || next == '_' { return false; }
            }
        }
        true
    }

    fn eat_char(&mut self, c: char) -> bool {
        self.skip_ws();
        if self.pos < self.chars.len() && self.chars[self.pos] == c {
            self.pos += 1;
            return true;
        }
        false
    }

    fn eat_str(&mut self, s: &str) -> bool {
        self.skip_ws();
        let sc: Vec<char> = s.chars().collect();
        if self.pos + sc.len() > self.chars.len() {
            return false;
        }
        for (i, &c) in sc.iter().enumerate() {
            if self.chars[self.pos + i] != c { return false; }
        }
        self.pos += sc.len();
        true
    }

    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.eat_str(s) { Ok(()) }
        else { Err(format!("位置 {}: 預期 '{s}'，實際得到 {:?}", self.pos, self.chars.get(self.pos))) }
    }

    fn ident(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.pos >= self.chars.len() || !(self.chars[self.pos].is_ascii_alphabetic() || self.chars[self.pos] == '_') {
            return Err(format!("位置 {}: 預期識別符號，得到 '{:?}'", self.pos, self.chars.get(self.pos)));
        }
        let start = self.pos;
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_') {
            self.pos += 1;
        }
        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn number(&mut self) -> Result<u64, String> {
        self.skip_ws();
        if self.pos >= self.chars.len() || !self.chars[self.pos].is_ascii_digit() {
            return Err(format!("位置 {}: 預期數字", self.pos));
        }
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<u64>().map_err(|_| format!("位置 {}: 無效數字", start))
    }
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn generate_code(
    name: &str,
    ports: &[PortDecl],
    regs: &[RegDecl],
    always_blocks: &[AlwaysBlock],
    assigns: &[Assign],
) -> TS {
    let name_str = name.to_string();
    let mut stmts = Vec::new();

    for p in ports {
        let n = &p.name;
        let w = p.width;
        match p.dir.as_str() {
            "input" => stmts.push(quote! { __m = __m.input(#n, #w); }),
            "output" => stmts.push(quote! { __m = __m.output(#n, #w); }),
            "inout" => stmts.push(quote! { __m = __m.inout(#n, #w); }),
            _ => {}
        }
    }

    for r in regs {
        let n = &r.name;
        let w = r.width;
        stmts.push(quote! { __m = __m.reg(#n, #w); });
    }

    for ab in always_blocks {
        for s in &ab.stmts {
            match s {
                HdlStmt::Nonblocking { target, value } => {
                    let t = target;
                    let e = gen_expr(value);
                    stmts.push(quote! { __m = __m.dff(#t, #e); });
                }
                HdlStmt::Blocking { target, value } => {
                    let t = target;
                    let e = gen_expr(value);
                    stmts.push(quote! { __m = __m.blocking(#t, #e); });
                }
            }
        }
    }

    for a in assigns {
        let t = &a.target;
        let e = gen_expr(&a.value);
        stmts.push(quote! { __m = __m.assign(#t, #e); });
    }

    quote! {{
        let mut __m = v2f_rust::HdlModule::new(#name_str);
        #(#stmts)*
        __m
    }}
}

fn gen_expr(expr: &Expr) -> TS {
    match expr {
        Expr::Const(v, w) => {
            let val = *v;
            let width = *w;
            quote! { v2f_rust::HdlExpr::Const(#val, #width) }
        }
        Expr::Ident(name) => {
            let n = name.as_str();
            quote! { v2f_rust::HdlExpr::Ident(#n.to_string()) }
        }
        Expr::Index(base, idx) => {
            let b = gen_expr(base);
            let i = *idx;
            quote! { v2f_rust::HdlExpr::Index(Box::new(#b), #i) }
        }
        Expr::Range(base, msb, lsb) => {
            let b = gen_expr(base);
            let m = *msb;
            let l = *lsb;
            quote! { v2f_rust::HdlExpr::Range(Box::new(#b), #m, #l) }
        }
        Expr::Concat(items) => {
            let exprs: Vec<TS> = items.iter().map(gen_expr).collect();
            quote! { v2f_rust::HdlExpr::Concat(vec![#(#exprs),*]) }
        }
        Expr::Add(l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            quote! { v2f_rust::HdlExpr::Add(Box::new(#left), Box::new(#right)) }
        }
        Expr::Sub(l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            quote! { v2f_rust::HdlExpr::Sub(Box::new(#left), Box::new(#right)) }
        }
        Expr::And(l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            quote! { v2f_rust::HdlExpr::And(Box::new(#left), Box::new(#right)) }
        }
        Expr::Or(l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            quote! { v2f_rust::HdlExpr::Or(Box::new(#left), Box::new(#right)) }
        }
        Expr::Xor(l, r) => {
            let left = gen_expr(l);
            let right = gen_expr(r);
            quote! { v2f_rust::HdlExpr::Xor(Box::new(#left), Box::new(#right)) }
        }
        Expr::Not(inner) => {
            let i = gen_expr(inner);
            quote! { v2f_rust::HdlExpr::Not(Box::new(#i)) }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blinky() {
        let input = "module Blinky { input clk : 1 , output led : 1 , reg [ 25 : 0 ] counter , always ( posedge clk ) { counter <= counter + 1 ; } assign led = counter [ 25 ] ; }";
        let result = parse_fpga(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_blinky_nice() {
        let input = "module Blinky { input clk: 1, output led: 1, reg [25:0] counter, always(posedge clk) { counter <= counter + 1; } assign led = counter[25]; }";
        let result = parse_fpga(input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }
}

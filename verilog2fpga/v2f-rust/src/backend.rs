use crate::hdl::{HdlExpr, HdlModule, HdlStmt, HdlPortDir};

pub fn to_verilog(module: &HdlModule) -> String {
    let mut s = String::new();
    s.push_str(&format!("module {} (\n", module.name));
    let port_lines: Vec<String> = module.ports.iter().map(|p| {
        let dir = match p.direction {
            HdlPortDir::Input => "input",
            HdlPortDir::Output => "output",
            HdlPortDir::Inout => "inout",
        };
        let width = if p.width > 1 { format!(" [{}:0] ", p.width - 1) } else { " ".to_string() };
        format!("  {}{}{}", dir, width, p.name)
    }).collect();
    s.push_str(&port_lines.join(",\n"));
    s.push_str("\n);\n");

    for stmt in &module.stmts {
        match stmt {
            HdlStmt::DeclReg { name, width } => {
                if *width > 1 { s.push_str(&format!("  reg [{}:0] {};\n", width - 1, name)); }
                else { s.push_str(&format!("  reg {};\n", name)); }
            }
            HdlStmt::DeclWire { name, width } => {
                if *width > 1 { s.push_str(&format!("  wire [{}:0] {};\n", width - 1, name)); }
                else { s.push_str(&format!("  wire {};\n", name)); }
            }
            HdlStmt::Assign { target, value } => {
                s.push_str(&format!("  assign {} = {};\n", target, expr_to_str(value)));
            }
            HdlStmt::Blocking { target, value } => {
                s.push_str(&format!("  {} = {};\n", target, expr_to_str(value)));
            }
            HdlStmt::Nonblocking { target, value } => {
                s.push_str(&format!("  {} <= {};\n", target, expr_to_str(value)));
            }
        }
    }

    s.push_str("endmodule\n");
    s
}

fn expr_to_str(expr: &HdlExpr) -> String {
    match expr {
        HdlExpr::Const(v, _) => format!("{}'d{}", if *v > 0xFFFF { 32 } else if *v > 0xFF { 16 } else { 8 }, v),
        HdlExpr::Ident(name) => name.clone(),
        HdlExpr::Index(base, idx) => format!("{}[{}]", expr_to_str(base), idx),
        HdlExpr::Range(base, msb, lsb) => format!("{}[{}:{}]", expr_to_str(base), msb, lsb),
        HdlExpr::Concat(exprs) => {
            let parts: Vec<String> = exprs.iter().map(expr_to_str).collect();
            format!("{{{}}}", parts.join(", "))
        }
        HdlExpr::Add(a, b) => format!("{} + {}", expr_to_str(a), expr_to_str(b)),
        HdlExpr::Sub(a, b) => format!("{} - {}", expr_to_str(a), expr_to_str(b)),
        HdlExpr::And(a, b) => format!("{} & {}", expr_to_str(a), expr_to_str(b)),
        HdlExpr::Or(a, b) => format!("{} | {}", expr_to_str(a), expr_to_str(b)),
        HdlExpr::Xor(a, b) => format!("{} ^ {}", expr_to_str(a), expr_to_str(b)),
        HdlExpr::Not(a) => format!("~{}", expr_to_str(a)),
    }
}

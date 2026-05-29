use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use verilog_parser::parse::parse_verilog;
use verilog_parser::ast::Module;

pub fn expand_includes(source: &str, base_path: &Path) -> String {
    preprocess(source, base_path, &mut HashSet::new(), &mut HashMap::new())
}

pub fn preprocess_only(source: &str) -> String {
    preprocess(source, Path::new("."), &mut HashSet::new(), &mut HashMap::new())
}

pub fn parse_file(path: &str) -> Vec<Module> {
    let source = fs::read_to_string(path).expect(&format!("Failed to read file: {}", path));
    let base = Path::new(path).parent().unwrap_or(Path::new("."));
    let expanded = preprocess(&source, base, &mut HashSet::new(), &mut HashMap::new());
    parse_verilog(&expanded)
}

fn preprocess(source: &str, base_path: &Path, visited: &mut HashSet<PathBuf>, macros: &mut HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut if_stack: Vec<bool> = Vec::new(); // true = condition was true (we're outputting)

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('`') {
            let rest = rest.trim();
            if rest.starts_with("include ") || rest.starts_with("include\"") {
                let filename = rest.trim_start_matches("include ").trim_start_matches("include\"").trim_matches('"');
                let inc_path = base_path.join(filename);
                let canon = fs::canonicalize(&inc_path).ok();
                if let Some(ref canon_str) = canon {
                    if !visited.insert(canon_str.clone()) {
                        eprintln!("Warning: circular include detected: {}", inc_path.display());
                        result.push('\n');
                        continue;
                    }
                }
                if let Ok(include_source) = fs::read_to_string(&inc_path) {
                    let parent = inc_path.parent().unwrap_or(base_path);
                    let expanded = preprocess(&include_source, parent, visited, macros);
                    result.push_str(&expanded);
                    result.push('\n');
                    if let Some(canon_str) = canon {
                        visited.remove(&canon_str);
                    }
                } else {
                    eprintln!("Warning: cannot open include file: {}", inc_path.display());
                    result.push('\n');
                }
            } else if rest.starts_with("define ") {
                let def = rest.trim_start_matches("define ").trim();
                if let Some(eq_pos) = def.find(|c: char| c.is_whitespace()) {
                    let name = &def[..eq_pos];
                    let value = strip_comment(def[eq_pos..].trim());
                    macros.insert(name.to_string(), value);
                } else {
                    macros.insert(strip_comment(def), String::new());
                }
                result.push('\n');
            } else if rest.starts_with("undef ") {
                let name = rest.trim_start_matches("undef ").trim();
                macros.remove(name);
                result.push('\n');
            } else if rest.starts_with("ifdef ") || rest.starts_with("ifndef ") {
                let is_ifdef = rest.starts_with("ifdef ");
                let name = rest.trim_start_matches("ifdef ").trim_start_matches("ifndef ").trim();
                let defined = macros.contains_key(name);
                let in_skipped = if_stack.iter().any(|&active| !active);
                let cond_true = if is_ifdef { defined } else { !defined };
                let active = !in_skipped && cond_true;
                if_stack.push(active);
                result.push('\n');
            } else if rest.starts_with("else") {
                if let Some(&last) = if_stack.last() {
                    let in_skipped = if_stack[..if_stack.len()-1].iter().any(|&active| !active);
                    if !in_skipped {
                        // flip: if last was active (true), now skip; if last was inactive, reactivate
                        let len = if_stack.len();
                        if_stack[len - 1] = !last;
                    }
                }
                result.push('\n');
            } else if rest.starts_with("endif") {
                if_stack.pop();
                result.push('\n');
            } else {
                result.push('\n');
            }
        } else {
            let skipping = if_stack.iter().any(|&active| !active);
            if !skipping {
                result.push_str(&substitute_macros(line, macros));
            }
            result.push('\n');
        }
    }
    result
}

fn substitute_macros(line: &str, macros: &HashMap<String, String>) -> String {
    let mut result = line.to_string();
    loop {
        let mut changed = false;
        let mut out = String::new();
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '`' {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                if end > start {
                    let name: String = chars[start..end].iter().collect();
                    if let Some(value) = macros.get(&name) {
                        out.push_str(value);
                        changed = true;
                    } else {
                        out.push('`');
                        out.push_str(&name);
                    }
                    i = end;
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        if !changed { break; }
        result = out;
    }
    result
}

fn strip_comment(line: &str) -> String {
    if let Some(pos) = line.find("//") {
        line[..pos].trim_end().to_string()
    } else {
        line.to_string()
    }
}

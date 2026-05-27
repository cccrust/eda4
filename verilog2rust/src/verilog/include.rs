use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use crate::verilog::parse::parse_verilog;
use crate::verilog::ast::Module;

pub fn parse_file(path: &str) -> Vec<Module> {
    let source = fs::read_to_string(path).expect(&format!("Failed to read file: {}", path));
    let base = Path::new(path).parent().unwrap_or(Path::new("."));
    let expanded = expand_includes(&source, base);
    parse_verilog(&expanded)
}

pub fn expand_includes(source: &str, base_path: &Path) -> String {
    expand_includes_rec(source, base_path, &mut HashSet::<PathBuf>::new())
}

fn expand_includes_rec(source: &str, base_path: &Path, visited: &mut HashSet<PathBuf>) -> String {
    let mut result = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('`') {
            if let Some(include_rest) = rest.trim().strip_prefix("include") {
                let filename = include_rest.trim().trim_matches('"');
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
                    result.push_str(&expand_includes_rec(&include_source, parent, visited));
                    result.push('\n');
                    if let Some(canon_str) = canon {
                        visited.remove(&canon_str);
                    }
                } else {
                    eprintln!("Warning: cannot open include file: {}", inc_path.display());
                    result.push('\n');
                }
            } else {
                // skip other backtick directives
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

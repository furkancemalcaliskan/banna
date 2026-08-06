use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub static RX_PLURAL_ES: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(ch|sh|s|x|z)$").unwrap());
pub static RX_PLURAL_IES: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)[^aeiou]y$").unwrap());
static RX_PLACEHOLDER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{([\w\.]+)\}").unwrap());
static RX_PLACEHOLDER_ALT: Lazy<Regex> = Lazy::new(|| Regex::new(r"__([\w\.]+?)__").unwrap());

pub fn to_lower_camel(s: &str) -> String {
    if s.is_empty() {
        return s.into();
    }
    let mut ch = s.chars();
    match ch.next() {
        Some(f) => f.to_lowercase().chain(ch).collect(),
        None => s.into(),
    }
}

pub fn to_kebab(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

pub fn pluralize(w: &str) -> String {
    if RX_PLURAL_ES.is_match(w) {
        format!("{w}es")
    } else if RX_PLURAL_IES.is_match(w) {
        format!("{}ies", &w[..w.len() - 1])
    } else {
        format!("{w}s")
    }
}

#[allow(dead_code)]
pub fn to_upper_camel(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    first.to_uppercase().collect::<String>() + chars.as_str()
}

pub fn read_text(p: &Path) -> Result<String> {
    fs::read_to_string(p).with_context(|| format!("read_text failed: {}", p.display()))
}

pub fn write_text(p: &Path, s: &str) -> Result<()> {
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(p, s)?;
    Ok(())
}

pub fn render_template(src: &str, mapping: &HashMap<String, String>) -> String {
    fn apply_regex(src: &str, rx: &Regex, mapping: &HashMap<String, String>) -> String {
        let mut out = String::with_capacity(src.len());
        let mut last_idx = 0;

        for caps in rx.captures_iter(src) {
            let m = caps.get(0).unwrap();
            let key = caps.get(1).unwrap().as_str();

            out.push_str(&src[last_idx..m.start()]);

            let replacement = mapping
                .get(key)
                .cloned()
                .unwrap_or_else(|| m.as_str().into());

            if replacement.contains('\n') {
                // Preserve indentation of the placeholder line for multi-line insertions.
                let line_start = src[..m.start()].rfind('\n').map_or(0, |i| i + 1);
                let indent = src[line_start..m.start()]
                    .chars()
                    .rev()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();

                let mut parts = replacement.split('\n');
                if let Some(first) = parts.next() {
                    out.push_str(first);
                }
                for part in parts {
                    out.push('\n');
                    out.push_str(&indent);
                    out.push_str(part);
                }
            } else {
                out.push_str(&replacement);
            }

            last_idx = m.end();
        }

        out.push_str(&src[last_idx..]);
        out
    }

    let first = apply_regex(src, &RX_PLACEHOLDER, mapping);
    apply_regex(&first, &RX_PLACEHOLDER_ALT, mapping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_names_to_supported_cases() {
        assert_eq!(to_lower_camel("InvoiceItem"), "invoiceItem");
        assert_eq!(to_upper_camel("invoiceItem"), "InvoiceItem");
        assert_eq!(to_kebab("InvoiceItem"), "invoice-item");
    }

    #[test]
    fn pluralizes_common_suffixes() {
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("invoice"), "invoices");
    }

    #[test]
    fn renders_both_placeholder_styles_and_preserves_indentation() {
        let mapping = HashMap::from([
            ("entity.name".to_owned(), "Invoice".to_owned()),
            ("fields".to_owned(), "Id\nName".to_owned()),
        ]);
        let template = "class ${entity.name} {\n    __fields__\n}";

        assert_eq!(
            render_template(template, &mapping),
            "class Invoice {\n    Id\n    Name\n}"
        );
    }

    #[test]
    fn leaves_unknown_placeholders_unchanged() {
        assert_eq!(
            render_template("${unknown} __also.unknown__", &HashMap::new()),
            "${unknown} __also.unknown__"
        );
    }
}

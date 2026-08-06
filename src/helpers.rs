use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/* ------------------------------ LOGGING ------------------------------- */

pub static VERBOSE: AtomicBool = AtomicBool::new(true);

#[macro_export]
macro_rules! vprintln {
    ($($arg:tt)*) => {{
        if $crate::helpers::VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
            println!($($arg)*);
        }
    }};
}

pub(crate) fn log_verbose(msg: &str) {
    if VERBOSE.load(Ordering::Relaxed) {
        println!("{msg}");
    }
}

/* ------------------------------ HELPERS ------------------------------- */

pub(crate) fn last_segment(qname: &str) -> &str {
    qname.rsplit('.').next().unwrap_or(qname)
}

pub(crate) fn ns_of(qname: &str) -> String {
    let mut parts = qname.split('.').collect::<Vec<_>>();
    if parts.len() > 1 {
        parts.pop();
        parts.join(".")
    } else {
        String::new()
    }
}

pub(crate) fn infer_domain_from_src(src_path: &Path) -> Option<String> {
    const LAYERS: &[&str] = &[
        "Application",
        "Application.Contracts",
        "Domain",
        "Domain.Shared",
        "EntityFrameworkCore",
    ];

    use std::collections::HashMap;
    let mut score: HashMap<String, usize> = HashMap::new();

    for entry in fs::read_dir(src_path).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_dir() {
            continue;
        }
        for &layer in LAYERS {
            if name == layer {
                continue;
            }
            if name.ends_with(layer)
                && let Some((prefix, _)) = name.rsplit_once('.')
            {
                *score.entry(prefix.to_string()).or_default() += 1;
                break;
            }
        }
    }

    score
        .into_iter()
        .max_by(|(a_dom, a_cnt), (b_dom, b_cnt)| {
            a_cnt.cmp(b_cnt).then_with(|| b_dom.len().cmp(&a_dom.len()))
        })
        .map(|(dom, _)| dom)
        .or_else(|| {
            for entry in fs::read_dir(src_path).ok()?.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !entry.path().is_dir() {
                    continue;
                }
                if let Some((prefix, _)) = name.rsplit_once('.')
                    && !prefix.is_empty()
                {
                    return Some(prefix.to_string());
                }
            }
            None
        })
}

pub(crate) fn ns_relative_path(domain: &str, namespace: &str) -> PathBuf {
    if namespace == domain {
        PathBuf::new()
    } else if let Some(rest) = namespace.strip_prefix(&(domain.to_string() + ".")) {
        PathBuf::from_iter(rest.split('.'))
    } else {
        PathBuf::from_iter(namespace.split('.'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_namespace_parts() {
        assert_eq!(last_segment("Acme.Billing.Invoices"), "Invoices");
        assert_eq!(ns_of("Acme.Billing.Invoices"), "Acme.Billing");
        assert_eq!(ns_of("Invoices"), "");
    }

    #[test]
    fn maps_namespaces_to_relative_paths() {
        assert_eq!(ns_relative_path("Acme", "Acme"), PathBuf::new());
        assert_eq!(
            ns_relative_path("Acme", "Acme.Billing.Invoices"),
            PathBuf::from("Billing/Invoices")
        );
    }
}

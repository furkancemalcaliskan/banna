use anyhow::{Result, anyhow};
use rust_embed::RustEmbed;
use std::borrow::Cow;

// Generation assets are compiled into both CLI binaries.
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[derive(RustEmbed)]
#[folder = "projects/"]
struct Projects;

pub fn read_tpl_text(rel: &str) -> Result<String> {
    let path = rel.replace('\\', "/");
    if let Some(file) = Templates::get(&path) {
        let bytes: Cow<'static, [u8]> = file.data;
        Ok(String::from_utf8(bytes.to_vec())?)
    } else {
        Err(anyhow!("embedded template not found: {path}"))
    }
}

pub fn embedded_walk(prefix: &str) -> Vec<String> {
    let norm = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };

    Templates::iter()
        .filter_map(|f| {
            let p = f.as_ref().to_string();

            if !p.starts_with(&norm) || !p.ends_with(".tpl") {
                return None;
            }
            let rest = &p[norm.len()..];
            if rest.contains('/') {
                return None;
            }

            Some(p)
        })
        .collect()
}

pub fn embedded_walk_recursive(prefix: &str) -> Vec<String> {
    let norm = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };

    Templates::iter()
        .map(|f| f.as_ref().to_string())
        .filter(|p| p.starts_with(&norm) && p.ends_with(".tpl"))
        .collect()
}

pub fn embedded_projects_walk(prefix: &str) -> Vec<String> {
    let norm = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };

    Projects::iter()
        .map(|f| f.as_ref().to_string())
        .filter(|p| p.starts_with(&norm))
        .collect()
}

pub fn read_project_file(rel: &str) -> Result<Vec<u8>> {
    let path = rel.replace('\\', "/");
    if let Some(file) = Projects::get(&path) {
        let bytes: Cow<'static, [u8]> = file.data;
        Ok(bytes.to_vec())
    } else {
        Err(anyhow!("embedded project file not found: {path}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{embedded_projects_walk, read_project_file};

    #[test]
    fn react_native_scaffold_embeds_upstream_notice_and_complete_license_set() {
        let files = embedded_projects_walk("Vanilla/react-native");
        for required in [
            "Vanilla/react-native/NOTICE.md",
            "Vanilla/react-native/LICENSE.LGPL-3.0-only.txt",
            "Vanilla/react-native/LICENSE.GPL-3.0-only.txt",
        ] {
            assert!(
                files.iter().any(|file| file == required),
                "missing {required}"
            );
            assert!(!read_project_file(required).unwrap().is_empty());
        }
    }
}

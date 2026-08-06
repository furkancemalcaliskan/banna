mod angular;
mod command;
mod project_meta;
mod razor;
mod react_native;
mod vue;
mod vue_workspace;

#[cfg(test)]
mod test_support;

use self::angular::generate_angular_ui;
pub(crate) use self::project_meta::ProjectMetaChangeRequest;
pub use self::project_meta::apply_project_meta_change;
use self::razor::generate_mvc_razor;
use self::react_native::generate_mobile_react_native;
use self::vue::generate_mvc_vue;
use self::vue_workspace::prepare_vue_workspace;
use crate::helpers::{last_segment, log_verbose, ns_of, ns_relative_path};
use crate::merge::merge_into_existing_files;
use crate::models::{Field, MobileUi, UiTarget};
use crate::ports::{CommandRunner, CommandSpec, ExternalTool};
use crate::templates::{embedded_walk, embedded_walk_recursive, read_tpl_text};
use crate::utils::{
    pluralize, render_template, to_kebab, to_lower_camel, to_upper_camel, write_text,
};
use anyhow::{Context, Result, anyhow};
use itertools::Itertools;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn insert_into_js_block(content: &str, block: &str, entries: &[&str]) -> Result<(String, bool)> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let target = format!("{block}:");
    let mut start_idx = None;
    let mut end_idx = None;
    let mut depth = 0i32;

    for (i, line) in lines.iter().enumerate() {
        if start_idx.is_none() && line.contains(&target) {
            start_idx = Some(i);
        }
        if start_idx.is_some() {
            depth += line.matches('{').count() as i32;
            depth -= line.matches('}').count() as i32;
            if depth == 0 {
                end_idx = Some(i);
                break;
            }
        }
    }

    let (start, end) = match (start_idx, end_idx) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return Ok((content.to_string(), false)),
    };
    let mut end = end;

    let indent = lines
        .get(start)
        .map(|l| {
            l.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_default();
    let child_indent = format!("{indent}  ");

    let mut changed = false;
    for entry in entries {
        if lines[start..=end].iter().any(|l| l.contains(entry)) {
            continue;
        }
        lines.insert(end, format!("{child_indent}{entry}"));
        changed = true;
        end += 1;
    }

    if !changed {
        return Ok((content.to_string(), false));
    }

    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        out.push_str(line);
        if i + 1 != lines.len() {
            out.push('\n');
        }
    }
    Ok((out, true))
}

fn ensure_bundle_line_any(text: &mut String, markers: &[&str], line: &str) -> bool {
    let asset_path = line.split('"').nth(1);
    if text.contains(line) || asset_path.is_some_and(|path| text.contains(path)) {
        return false;
    }
    for marker in markers {
        if let Some(marker_pos) = text.find(marker)
            && let Some(rel_brace) = text[marker_pos..].find('{')
        {
            let brace_pos = marker_pos + rel_brace;
            let line_start = text[..marker_pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let indent: String = text[line_start..marker_pos]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            let insert_indent = format!("{indent}    ");
            text.insert_str(brace_pos + 1, &format!("\n{insert_indent}{line}"));
            return true;
        }
    }
    false
}

pub fn detect_ui(project_root: &Path, domain: &str) -> (bool, bool) {
    let mut has_mvc = false;
    let mut has_angular = false;

    let src_root = project_root.join("src");
    if let Ok(entries) = fs::read_dir(&src_root) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            if name == format!("{}.Web", domain) {
                has_mvc = true;
            }
        }
    }

    if let Ok(entries) = fs::read_dir(project_root) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();

            if name.eq_ignore_ascii_case("angular") {
                has_angular = true;
            }
        }
    }

    (has_mvc, has_angular)
}

pub(crate) struct GenerationRequest<'a> {
    pub project_root: &'a Path,
    pub domain: &'a str,
    pub namespace: &'a str,
    pub entity: &'a str,
    pub fields: &'a [Field],
    pub ui_target: UiTarget,
    pub mobile_ui: MobileUi,
    pub no_merge: bool,
}

pub(super) struct GenerationContext<'a> {
    pub(super) project_root: &'a Path,
    pub(super) out_root: PathBuf,
    pub(super) domain: &'a str,
    pub(super) namespace: &'a str,
    pub(super) entity: &'a str,
    pub(super) fields: &'a [Field],
}

impl<'a> GenerationContext<'a> {
    fn from_request(request: &GenerationRequest<'a>) -> Self {
        Self {
            project_root: request.project_root,
            out_root: request.project_root.join("src"),
            domain: request.domain,
            namespace: request.namespace,
            entity: request.entity,
            fields: request.fields,
        }
    }
}

pub fn run_generate(
    request: &GenerationRequest<'_>,
    logger: Option<&mut dyn FnMut(&str)>,
    runner: &dyn CommandRunner,
) -> Result<()> {
    run_generate_inner(request, logger, runner)
}

fn run_generate_inner(
    request: &GenerationRequest<'_>,
    mut logger: Option<&mut dyn FnMut(&str)>,
    runner: &dyn CommandRunner,
) -> Result<()> {
    let context = GenerationContext::from_request(request);
    let project_root = context.project_root;
    let domain = context.domain;
    let ui_target = request.ui_target;
    let mobile_ui = request.mobile_ui;
    let no_merge = request.no_merge;
    let mut log = |msg: &str| {
        if let Some(cb) = logger.as_mut() {
            cb(msg);
        } else {
            log_verbose(msg);
        }
    };

    if !context.out_root.exists() {
        return Err(anyhow!(
            "Expected DDD root not found: {} (no 'src' under the project root)",
            context.out_root.display()
        ));
    }

    let mut required_tools = Vec::new();
    if mini_excel_package_missing(project_root, domain)? {
        required_tools.push(ExternalTool::DotNet);
    }
    if matches!(ui_target, UiTarget::Vue)
        && vue_workspace::vue_packages_missing(project_root, domain)
    {
        required_tools.extend([ExternalTool::Npm, ExternalTool::Abp]);
    }
    command::preflight_tools(runner, &required_tools, &mut log)?;

    ensure_mini_excel_package(runner, project_root, domain, &mut log)?;
    ensure_wwwroot_files(project_root, domain, &mut log)?;

    let mut mapping: HashMap<String, String> = compile_fragments(&context);
    mapping.insert(
        "ui_target".into(),
        match ui_target {
            UiTarget::None => "none",
            UiTarget::Razor => "razor",
            UiTarget::Vue => "vue",
            UiTarget::Angular => "angular",
        }
        .into(),
    );

    if matches!(ui_target, UiTarget::Razor | UiTarget::Vue) {
        ensure_web_components_and_toolbars(project_root, domain, &mapping, &mut log)?;
        ensure_web_module_toolbars(project_root, domain, &mut log)?;
    }

    generate_from(&context, ui_target, &mapping, &mut log)?;

    if matches!(ui_target, UiTarget::Vue) {
        prepare_vue_workspace(runner, project_root, domain, &mut log)?;
    }

    if matches!(mobile_ui, MobileUi::ReactNative) {
        generate_mobile_react_native(&context, &mapping, &mut log)?;
    }

    if !no_merge {
        merge_into_existing_files(&context.out_root, &mapping, &mut log)?;
    }
    Ok(())
}

/* --------------------------- GENERATION HELPERS ----------------------- */

pub(crate) fn ensure_mini_excel_package(
    runner: &dyn CommandRunner,
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let app_dir = project_root
        .join("src")
        .join(format!("{}.Application", domain));
    let csproj_path = app_dir.join(format!("{}.Application.csproj", domain));

    if !csproj_path.exists() {
        log(&format!(
            "skip: csproj not found at {}",
            csproj_path.display()
        ));
        return Ok(());
    }

    let content = fs::read_to_string(&csproj_path).with_context(|| {
        format!(
            "failed to read Application csproj at {}",
            csproj_path.display()
        )
    })?;

    let has_pkg = content
        .lines()
        .any(|line| line.contains("PackageReference") && line.contains("MiniExcel"));
    if has_pkg {
        log(&format!(
            "MiniExcel already referenced in {}",
            csproj_path.display()
        ));
        return Ok(());
    }

    log(&format!(
        "running 'dotnet add package MiniExcel' in {}",
        app_dir.display()
    ));
    let mut spec = CommandSpec::new(&app_dir, "dotnet", ["add", "package", "MiniExcel"]);
    spec.env.insert("DOTNET_NOLOGO".into(), "1".into());
    let output = runner.run(&spec).with_context(|| {
        format!(
            "failed to execute 'dotnet add package MiniExcel' in {}",
            app_dir.display()
        )
    })?;

    if !output.success {
        return Err(anyhow!(output.failure_message(&spec)));
    }

    Ok(())
}

fn mini_excel_package_missing(project_root: &Path, domain: &str) -> Result<bool> {
    let csproj_path = project_root
        .join("src")
        .join(format!("{}.Application", domain))
        .join(format!("{}.Application.csproj", domain));
    if !csproj_path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(&csproj_path)
        .with_context(|| format!("failed to read {}", csproj_path.display()))?;
    Ok(!content
        .lines()
        .any(|line| line.contains("PackageReference") && line.contains("MiniExcel")))
}

pub(crate) fn ensure_wwwroot_files(
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let web_dir = project_root.join("src").join(format!("{}.Web", domain));
    if !web_dir.exists() {
        log(&format!(
            "skip: web project directory not found at {}",
            web_dir.display()
        ));
        return Ok(());
    }
    let wwwroot = web_dir.join("wwwroot");

    let ensure_file = |name: &str, log: &mut dyn FnMut(&str)| -> Result<()> {
        let target = wwwroot.join(name);
        if target.exists() {
            log(&format!("{} already exists", target.display()));
            return Ok(());
        }
        let tpl_path = format!("Web/MVC/wwwroot/{name}");
        let content = match read_tpl_text(&tpl_path) {
            Ok(c) => c,
            Err(_) => {
                let tpl_with_ext = format!("{tpl_path}.tpl");
                read_tpl_text(&tpl_with_ext)
                    .with_context(|| format!("failed to read embedded template {}", tpl_path))?
            }
        };
        write_text(&target, &content)?;
        log(&format!("added {}", target.display()));
        Ok(())
    };

    ensure_file("global-scripts.js", log)?;
    ensure_file("global-styles.css", log)?;
    ensure_file("app.vuehelper.js", log)?;

    Ok(())
}

fn ensure_web_components_and_toolbars(
    project_root: &Path,
    domain: &str,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let web_dir = project_root.join("src").join(format!("{}.Web", domain));
    if !web_dir.exists() {
        log(&format!(
            "skip: web project directory not found at {}",
            web_dir.display()
        ));
        return Ok(());
    }

    let targets = [
        ("Web/MVC/Components", web_dir.join("Components")),
        ("Web/MVC/Toolbars", web_dir.join("Toolbars")),
    ];

    for (prefix, target_root) in targets {
        let prefix_norm = if prefix.ends_with('/') {
            prefix.to_string()
        } else {
            format!("{prefix}/")
        };
        for rel in embedded_walk_recursive(prefix) {
            let rel_tail = rel.strip_prefix(&prefix_norm).unwrap_or(rel.as_str());
            let out_rel = rel_tail.trim_end_matches(".tpl");
            let out_path = target_root.join(out_rel);
            if out_path.exists() {
                log(&format!("skip: {} already exists", out_path.display()));
                continue;
            }
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = render_template(&read_tpl_text(&rel)?, mapping);
            write_text(&out_path, &content)?;
            log(&format!("added {}", out_path.display()));
        }
    }

    Ok(())
}

fn ensure_web_module_toolbars(
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    fn find_class_block(text: &str) -> Option<(usize, usize, String)> {
        let rx = Regex::new(r"class\s+\w*WebModule").ok()?;
        if let Some(mat) = rx.find(text) {
            let brace_rel = text[mat.end()..].find('{')?;
            let open_idx = mat.end() + brace_rel;
            let mut depth = 0i32;
            for (i, ch) in text[open_idx..].char_indices() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        let line_start =
                            text[..mat.start()].rfind('\n').map(|p| p + 1).unwrap_or(0);
                        let indent: String = text[line_start..]
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect();
                        return Some((open_idx, open_idx + i, indent));
                    }
                }
            }
        }
        None
    }

    fn find_method_block(text: &str, method: &str) -> Option<(usize, usize, String)> {
        let pattern = format!(
            r"(?m)^\s*(?:public\s+|protected\s+|private\s+|internal\s+)?(?:override\s+)?void\s+{}\s*\(",
            regex::escape(method)
        );
        let rx = Regex::new(&pattern).ok()?;
        if let Some(mat) = rx.find(text) {
            let brace_rel = text[mat.end()..].find('{')?;
            let open_idx = mat.end() + brace_rel;
            let mut depth = 0i32;
            for (i, ch) in text[open_idx..].char_indices() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        let line_start =
                            text[..mat.start()].rfind('\n').map(|p| p + 1).unwrap_or(0);
                        let indent: String = text[line_start..]
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect();
                        return Some((open_idx, open_idx + i, indent));
                    }
                }
            }
        }
        None
    }

    let web_dir = project_root.join("src").join(format!("{}.Web", domain));
    if !web_dir.exists() {
        log(&format!(
            "skip: web project directory not found at {}",
            web_dir.display()
        ));
        return Ok(());
    }
    let domain_short = domain.split('.').next_back().unwrap_or(domain);
    let module_candidates = [
        web_dir.join("WebModule.cs"),
        web_dir.join(format!("{domain_short}WebModule.cs")),
    ];
    let web_module = module_candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| module_candidates[0].clone());

    if !web_module.exists() {
        log(&format!(
            "skip: web module not found at {}, toolbar setup not applied",
            web_module.display()
        ));
        return Ok(());
    }

    let mut text =
        fs::read_to_string(&web_module).with_context(|| format!("{}", web_module.display()))?;
    let mut changed = false;

    let toolbar_using = "using Volo.Abp.AspNetCore.Mvc.UI.Theme.Shared.Toolbars;";
    let domain_toolbar_using = format!("using {domain}.Web.Toolbars;");
    for line in [toolbar_using, domain_toolbar_using.as_str()] {
        if !text.contains(line) {
            if let Some(ns_pos) = text.find("\nnamespace") {
                text.insert_str(ns_pos + 1, &format!("{line}\n"));
            } else {
                text.insert_str(0, &format!("{line}\n"));
            }
            changed = true;
        }
    }

    if let Some((svc_open, svc_close, svc_indent)) = find_method_block(&text, "ConfigureServices") {
        let body_start = svc_open + 1;
        let body_text = &text[body_start..svc_close];
        if !body_text.contains("ConfigureToolbars();") {
            let body_indent = format!("{svc_indent}    ");
            let insertion = format!("{body_indent}ConfigureToolbars();\n");
            if let Some(bundle_idx) = body_text.find("ConfigureBundles") {
                let after_bundle = body_start + bundle_idx;
                let insert_pos = text[after_bundle..]
                    .find('\n')
                    .map(|n| after_bundle + n + 1)
                    .unwrap_or(svc_close);
                text.insert_str(insert_pos, &insertion);
            } else {
                let brace_line_start = text[..svc_close]
                    .rfind('\n')
                    .map(|p| p + 1)
                    .unwrap_or(svc_close);
                text.insert_str(brace_line_start, &insertion);
            }
            changed = true;
        }
    } else {
        log("skip: ConfigureServices not found, could not add toolbar call");
    }

    if !text.contains("void ConfigureToolbars(") {
        if let Some((_class_open, class_close, class_indent)) = find_class_block(&text) {
            let insert_pos = text[..class_close]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(class_close);
            let member_indent = format!("{class_indent}    ");
            let snippet = format!(
                "\n{indent}private void ConfigureToolbars()\n{indent}{{\n{indent}    Configure<AbpToolbarOptions>(options =>\n{indent}    {{\n{indent}        options.Contributors.Add(new ChangeThemeToolbarContributor());\n{indent}    }});\n{indent}}}\n",
                indent = member_indent
            );
            text.insert_str(insert_pos, &snippet);
            changed = true;
        } else {
            log("skip: WebModule class block not found, toolbar method not added");
        }
    }

    if changed {
        write_text(&web_module, &text)?;
        log(&format!(
            "patched {} with toolbar configuration",
            web_module.display()
        ));
    } else {
        log("WebModule already contains toolbar configuration");
    }

    Ok(())
}

fn ensure_domain_shared_localization(
    out_root: &Path,
    domain: &str,
    mapping: &HashMap<String, String>,
    fields: &[Field],
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let domain_short = mapping.get("domain_short").cloned().unwrap_or_default();
    let entity_name = mapping.get("entity_name").cloned().unwrap_or_default();
    let entity_plural = mapping.get("entity_plural").cloned().unwrap_or_default();

    let loc_dir = out_root
        .join(format!("{}.Domain.Shared", domain))
        .join("Localization")
        .join(&domain_short);

    fs::create_dir_all(&loc_dir)?;
    let en_json_path = loc_dir.join("en.json");

    let mut root_val: Value = if en_json_path.exists() {
        fs::read_to_string(&en_json_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or(Value::Object(Map::new()))
    } else {
        Value::Object(Map::new())
    };

    if !root_val.is_object() {
        root_val = Value::Object(Map::new());
    }
    {
        let obj = root_val.as_object_mut().unwrap();

        let culture_key = obj
            .keys()
            .find(|k| k.eq_ignore_ascii_case("culture"))
            .cloned()
            .unwrap_or_else(|| "Culture".to_string());
        if obj.get(&culture_key).is_none() {
            obj.insert(culture_key.clone(), Value::String("en".to_string()));
        } else if culture_key != "Culture"
            && let Some(val) = obj.remove(&culture_key)
        {
            obj.insert("Culture".to_string(), val);
        }

        let texts_key = obj
            .keys()
            .find(|k| k.eq_ignore_ascii_case("texts"))
            .cloned()
            .unwrap_or_else(|| "Texts".to_string());
        let need_recreate = match obj.get(&texts_key) {
            Some(v) => !v.is_object(),
            None => true,
        };
        if need_recreate {
            obj.insert(texts_key.clone(), Value::Object(Map::new()));
        }
        if texts_key != "Texts" {
            let val = obj
                .remove(&texts_key)
                .unwrap_or_else(|| Value::Object(Map::new()));
            obj.insert("Texts".to_string(), val);
        }
    }

    let texts_map: &mut Map<String, Value> = {
        let obj = root_val.as_object_mut().unwrap();
        let texts_key = obj
            .keys()
            .find(|k| k.eq_ignore_ascii_case("texts"))
            .cloned()
            .unwrap_or_else(|| "Texts".to_string());
        obj.get_mut(&texts_key).unwrap().as_object_mut().unwrap()
    };

    let mut ensure = |k: &str, v: &str| {
        if !texts_map.contains_key(k) {
            texts_map.insert(k.to_string(), Value::String(v.to_string()));
            log(&format!("l10n+ {k} = {v}"));
        } else {
            log(&format!("l10n= {k} (skip, exists)"));
        }
    };

    ensure(&format!("Permission:{}", entity_plural), &entity_plural);
    ensure(&entity_plural, &entity_plural);
    ensure(
        &format!("New{}", entity_name),
        &format!("New {}", entity_name),
    );
    ensure(&format!("Menu:{}", entity_plural), &entity_plural);
    ensure("Detail", "Detail");
    ensure("Permission:Create", "Create");
    ensure("Permission:Detail", "Detail");
    ensure("Permission:Edit", "Edit");
    ensure("Permission:Delete", "Delete");
    ensure("ExportToExcel", "Export to Excel");
    ensure("Close", "Close");
    ensure("Cancel", "Cancel");
    ensure("Save", "Save");
    ensure("Actions", "Actions");
    ensure("Pagination", "Pagination");
    ensure("PagerShow", "Show");
    ensure("PagerPageSize", "Page size");
    ensure("PagerEntries", "entries per page");
    ensure("PagerInfo", "Showing {0} to {1} of {2} entries");
    ensure("PagerFirst", "First");
    ensure("PagerPrevious", "Previous");
    ensure("PagerNext", "Next");
    ensure("PagerLast", "Last");
    ensure("ClearSelection", "Clear selection");
    ensure(
        "OneItemOnThisPageIsSelected",
        "1 item on this page is selected",
    );
    ensure(
        "NumberOfItemsOnThisPageAreSelected",
        "{0} items on this page are selected",
    );
    ensure("SelectAllItems", "Select all {0} items");
    ensure("AllItemsAreSelected", "All {0} items are selected");
    ensure("DeleteAllRecords", "Delete all records?");
    ensure(
        "DeleteSelectedRecords",
        "Delete the selected {0} record(s)?",
    );
    ensure("SuccessfullySaved", "Successfully Saved");
    ensure("SuccessfullyDeleted", "Successfully Deleted");
    ensure(
        "DeleteConfirmationMessage",
        "Are you sure you want to delete this item?",
    );
    for f in fields {
        let key = format!("{}:{}", entity_name, f.name);
        let value = if f.ftype.eq_ignore_ascii_case("Guid") {
            if let Some(nav_full) = &f.navigation {
                last_segment(nav_full).to_string()
            } else {
                f.name.clone()
            }
        } else {
            f.name.clone()
        };
        ensure(&key, &value);
    }

    let pretty = serde_json::to_string_pretty(&root_val)?;
    fs::write(&en_json_path, pretty)?;
    log(&format!("localized: {}", en_json_path.display()));
    Ok(())
}

/* ------------------------------ C# HELPERS ---------------------------- */

fn csharp_type(t: &str, required: bool) -> String {
    match t {
        "int" | "long" | "bool" | "Guid" | "DateTime" | "decimal" | "DateOnly" | "TimeOnly" => {
            if required {
                t.into()
            } else {
                format!("{t}?")
            }
        }
        "textarea" | "string" => {
            if required {
                "string".into()
            } else {
                "string?".into()
            }
        }
        _ => t.into(),
    }
}

fn backend_type_for_field(f: &Field) -> String {
    if f.ftype.eq_ignore_ascii_case("textarea") {
        "string".into()
    } else if f.ftype == "enum" {
        if let Some(nav) = &f.navigation {
            last_segment(split_nav(nav).0).to_string()
        } else {
            "Enum".into()
        }
    } else {
        f.ftype.clone()
    }
}

fn dto_type_for_field(f: &Field) -> String {
    let backend_ft = backend_type_for_field(f);
    if f.ftype == "enum" {
        if f.required {
            backend_ft
        } else {
            format!("{backend_ft}?")
        }
    } else {
        csharp_type(&backend_ft, f.required)
    }
}

fn split_nav(nav: &str) -> (&str, Option<&str>) {
    if let Some(idx) = nav.rfind('#') {
        (&nav[..idx], Some(&nav[idx + 1..]))
    } else {
        (nav, None)
    }
}

fn nav_parts(f: &Field) -> Option<(String, Option<String>)> {
    f.navigation.as_ref().map(|nav| {
        let (base, disp) = split_nav(nav);
        (
            base.to_string(),
            f.navigation_display
                .clone()
                .or_else(|| disp.filter(|s| !s.is_empty()).map(|s| s.to_string())),
        )
    })
}

fn attr_required_and_length(f: &Field, entity: &str) -> String {
    if (f.ftype == "string" || f.ftype == "textarea") && f.required {
        let mut s = String::from("[Required]\n");
        if let Some(_max) = f.max_length {
            s.push_str(&format!(
                "[StringLength({}Consts.{}MaxLength)]\n",
                entity, f.name
            ));
        }
        return s;
    }
    if (f.ftype == "string" || f.ftype == "textarea")
        && let Some(_max) = f.max_length
    {
        return format!("[StringLength({}Consts.{}MaxLength)]\n", entity, f.name);
    }
    String::new()
}

/* --------------------------- COMPILE FRAGMENTS ------------------------- */

#[allow(clippy::too_many_lines)]
pub(crate) fn compile_fragments(context: &GenerationContext<'_>) -> HashMap<String, String> {
    let out_root = &context.out_root;
    let domain = context.domain;
    let namespace = context.namespace;
    let entity = context.entity;
    let fields = context.fields;
    let domain_short = domain.split('.').next_back().unwrap_or(domain).to_string();
    let entity_lower = to_lower_camel(entity);
    let entity_plural = pluralize(entity);
    let entity_plural_lower = to_lower_camel(&entity_plural);
    let entity_plural_kebab = to_kebab(&entity_plural);

    let mut fields_decl = String::new();
    let mut fields_assignments = String::new();
    let mut constructor_params = String::new();
    let mut manager_params = String::new();
    let mut manager_checks = String::new();
    let mut manager_assignments = String::new();

    let mut repo_filter_list: Vec<String> = vec!["string? filterText = null".into()];
    let mut filter_method_list: Vec<String> = vec!["string? filterText = null".into()];
    let mut filter_name_list: Vec<String> = vec!["filterText".into()];

    let mut navigation_decls = String::new();
    let mut navigation_joins = String::new();
    let mut nav_selects_list = String::new();
    let mut nav_selects_single = String::new();

    let mut repository_nav_filters = String::new();
    let mut repository_entity_filters = String::new();
    let mut const_decls = String::new();

    let mut navigation_fk_configs = String::new();
    let mut mobile_form_validations = String::new();
    let mut mobile_form_initial_values = String::new();
    let mut mobile_form_inputs = String::new();
    let mut mobile_form_state = String::new();
    let mut mobile_form_modals = String::new();
    let mobile_form_refs = String::new();
    let mut mobile_form_extra_imports = String::new();
    let mut mobile_form_external_imports = String::new();
    let mut mobile_form_props = String::new();
    let mut mobile_form_prop_types = String::new();
    let mut mobile_list_fields = String::new();
    let mut mobile_lookup_methods = String::new();
    let mut mobile_screen_extra_imports = String::new();
    let mut mobile_form_styles = String::new();
    let mut mobile_screen_state = String::new();
    let mut mobile_screen_lookup_loaders = String::new();
    let mut mobile_screen_lookup_calls = String::new();
    let mut mobile_screen_effects = String::new();
    let mut mobile_screen_form_props = String::new();
    let mut mobile_has_checkbox = false;
    let mut mobile_has_datepicker = false;
    let mut mobile_has_combobox = false;
    let mut _mobile_has_textarea = false;

    for f in fields {
        let name = &f.name;
        let _ui_type = f.ftype.as_str();
        let backend_type = if f.ftype.eq_ignore_ascii_case("textarea") {
            "string".to_string()
        } else if f.ftype == "enum" {
            if let Some(nav) = &f.navigation {
                last_segment(nav).to_string()
            } else {
                "Enum".to_string()
            }
        } else {
            f.ftype.clone()
        };
        let required = f.required;
        let name_lower = to_lower_camel(name);

        let type_decl = if required {
            backend_type.to_string()
        } else {
            format!("{backend_type}?")
        };
        fields_decl.push_str(&format!(
            "public virtual {type_decl} {name} {{ get; set; }}\n"
        ));

        let ctor_type = if required {
            backend_type.to_string()
        } else {
            format!("{backend_type}?")
        };
        constructor_params.push_str(&format!("{ctor_type} {name_lower}, "));
        if required {
            if backend_type == "string" {
                fields_assignments.push_str(&format!(
                    "Check.NotNullOrWhiteSpace({name_lower}, nameof({name_lower}));\n"
                ));
            } else {
                fields_assignments.push_str(&format!(
                    "Check.NotNull({name_lower}, nameof({name_lower}));\n"
                ));
            }
        }
        if backend_type == "string"
            && let Some(max) = f.max_length
        {
            const_decls.push_str(&format!("public const int {}MaxLength = {};\n", name, max));
            fields_assignments.push_str(&format!(
                    "Check.Length({name_lower}, nameof({name_lower}), {entity}Consts.{name}MaxLength);\n"
                ));
        }
        fields_assignments.push_str(&format!("{name} = {name_lower};\n"));

        let mtype = if required {
            backend_type.to_string()
        } else {
            format!("{backend_type}?")
        };
        manager_params.push_str(&format!("{mtype} {name_lower}, "));
        if required {
            if backend_type == "string" {
                manager_checks.push_str(&format!(
                    "Check.NotNullOrWhiteSpace({name_lower}, nameof({name_lower}));\n"
                ));
                if f.max_length.is_some() {
                    manager_checks.push_str(&format!(
                        "Check.Length({name_lower}, nameof({name_lower}), {entity}Consts.{name}MaxLength);\n"
                    ));
                }
            } else {
                manager_checks.push_str(&format!(
                    "Check.NotNull({name_lower}, nameof({name_lower}));\n"
                ));
            }
        }
        manager_assignments.push_str(&format!("{entity_lower}.{name} = {name_lower};\n"));

        if let Some((nav_q, _nav_display_override)) = nav_parts(f) {
            let nav_short = last_segment(&nav_q).to_string();

            if f.ftype == "Guid" {
                if f.filterable {
                    repo_filter_list.push(format!("Guid? {name_lower} = null"));
                    filter_method_list.push(format!("Guid? {name_lower} = null"));
                    filter_name_list.push(name_lower.clone());
                }

                navigation_decls.push_str(&format!(
                    "public {nav_short}? {nav_short} {{ get; set; }} = null!;\n"
                ));

                let nav_var = to_lower_camel(&nav_short);
                let nav_plural = pluralize(&nav_var);
                navigation_joins.push_str(&format!(
                    "join {nav_var} in dbContext.Set<{nav_short}>() on {entity_lower}.{name} equals {nav_var}.Id into {nav_plural}\nfrom {nav_var} in {nav_plural}.DefaultIfEmpty()\n"
                ));
                nav_selects_list.push_str(&format!("{nav_short} = {nav_var},\n"));
                nav_selects_single.push_str(&format!(
                    "{nav_short} = dbContext.Set<{nav_short}>().FirstOrDefault({nav_var} => {nav_var}.Id == x.{name}),\n"
                ));

                let is_req = if required { ".IsRequired()" } else { "" };
                let delete_behavior = if required {
                    "DeleteBehavior.Cascade"
                } else {
                    "DeleteBehavior.SetNull"
                };
                navigation_fk_configs.push_str(&format!(
                    "b.HasOne<{ns}>().WithMany(){req}.HasForeignKey(x => x.{fk}).OnDelete({del});\n",
                    ns = nav_short,
                    req = is_req,
                    fk = f.name,
                    del = delete_behavior
                ));
            }
            if f.filterable && f.ftype != "Guid" {
                let filt_type = if backend_type == "string" {
                    "string"
                } else {
                    &backend_type
                };
                repo_filter_list.push(format!("{}? {} = null", filt_type, name_lower));
                filter_method_list.push(format!("{}? {} = null", filt_type, name_lower));
                filter_name_list.push(name_lower.clone());
            }
        } else if f.filterable {
            let filt_type = if backend_type == "string" {
                "string"
            } else {
                &backend_type
            };
            repo_filter_list.push(format!("{}? {} = null", filt_type, name_lower));
            filter_method_list.push(format!("{}? {} = null", filt_type, name_lower));
            filter_name_list.push(name_lower.clone());
        }
    }

    let mut first_string_field_visible: Option<String> = None;
    let mut first_string_field_any: Option<String> = None;
    for f in fields {
        if f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea") {
            if first_string_field_any.is_none() {
                first_string_field_any = Some(f.name.clone());
            }
            if f.show_in_ui && first_string_field_visible.is_none() {
                first_string_field_visible = Some(f.name.clone());
            }
        }
    }
    let first_string_field = first_string_field_visible.or(first_string_field_any);

    if let Some(ref name_field) = first_string_field {
        repository_nav_filters.push_str(&format!(
        ".WhereIf(!string.IsNullOrWhiteSpace(filterText), x => EF.Functions.Like(x.{entity}.{f}.Trim(), $\"%{{filterText.Trim()}}%\"))\n",
        entity = entity,
        f = name_field
    ));
        repository_entity_filters.push_str(&format!(
        ".WhereIf(!string.IsNullOrWhiteSpace(filterText), x => EF.Functions.Like(x.{f}.Trim(), $\"%{{filterText.Trim()}}%\"))\n",
        f = name_field
    ));
    } else {
        repository_nav_filters
            .push_str(".WhereIf(!string.IsNullOrWhiteSpace(filterText), x => true)\n");
        repository_entity_filters
            .push_str(".WhereIf(!string.IsNullOrWhiteSpace(filterText), x => true)\n");
    }

    let format_mobile_value = |ftype: &str, base: &str, field: &str| -> String {
        match ftype.to_ascii_lowercase().as_str() {
            "bool" => format!("{base}.{field} ? t('AbpUi::Yes') : t('AbpUi::No')"),
            "datetime" | "dateonly" | "timeonly" => {
                format!("{base}.{field} ? new Date({base}.{field}).toLocaleDateString() : ''")
            }
            "int" | "long" | "decimal" => format!("{base}.{field} ?? ''"),
            _ => format!("{base}.{field} || ''"),
        }
    };

    let mut title_field = first_string_field
        .clone()
        .unwrap_or_else(|| "Id".to_string());
    if title_field.eq_ignore_ascii_case("id") {
        title_field = "Id".into();
    }
    let title_field_lower = to_lower_camel(&title_field);

    for f in fields {
        if !f.show_in_ui && f.navigation.is_none() {
            continue;
        }

        if f.name.eq_ignore_ascii_case("id") {
            continue;
        }

        let fname = &f.name;
        let field = to_lower_camel(fname);
        let ftype = f.ftype.as_str();
        let is_bool = ftype.eq_ignore_ascii_case("bool");
        let is_numeric = ftype.eq_ignore_ascii_case("int")
            || ftype.eq_ignore_ascii_case("long")
            || ftype.eq_ignore_ascii_case("decimal");
        let is_datetime = ftype.eq_ignore_ascii_case("datetime")
            || ftype.eq_ignore_ascii_case("dateonly")
            || ftype.eq_ignore_ascii_case("timeonly");
        let is_textarea = ftype.eq_ignore_ascii_case("textarea");
        let is_nav = f.navigation.is_some() && ftype.eq_ignore_ascii_case("guid");
        let is_concurrency_stamp = fname.eq_ignore_ascii_case("concurrencyStamp");

        if is_concurrency_stamp {
            // Hidden field: keep in payload but don't render input (initial value is added upfront)
            continue;
        }

        if is_bool {
            mobile_form_initial_values.push_str(&format!(
                "      {field}: {entity_lower}?.{field} ?? false,\n",
                entity_lower = entity_lower
            ));
        }

        let rule = if is_bool {
            "Yup.boolean()".to_string()
        } else if is_numeric {
            "Yup.number()".to_string()
        } else {
            "Yup.string()".to_string()
        };

        let rule = if f.required {
            format!("{rule}.required('AbpValidation::ThisFieldIsRequired.')")
        } else {
            format!("{rule}.nullable()")
        };

        mobile_form_validations.push_str(&format!("  {field}: {rule},\n"));

        if is_nav {
            let nav_full = f.navigation.as_ref().unwrap();
            let nav_short = last_segment(nav_full).to_string();
            let nav_var = to_lower_camel(&nav_short);
            let field_kebab = to_kebab(&nav_short);
            let lookup_endpoint = format!(
                "api/app/{epk}/{fk}-lookup",
                epk = entity_plural_kebab,
                fk = field_kebab
            );
            let lookup_fn = format!("get{nav}Lookup", nav = nav_short);

            if !mobile_lookup_methods.contains(&lookup_fn) {
                mobile_lookup_methods.push_str(&format!(
                    "export const {fn_name} = (filter, maxResultCount = 100, skipCount = 0) =>\n  api\n    .get('/{endpoint}', {{\n      params: {{\n        filter,\n        maxResultCount,\n        skipCount,\n      }},\n    }})\n    .then(({{ data }}) => data);\n\n",
                    fn_name = lookup_fn,
                    endpoint = lookup_endpoint
                ));
            }

            if !mobile_screen_extra_imports.contains(&lookup_fn) {
                mobile_screen_extra_imports.push_str(&format!(
                    "import {{ {fn_name} }} from '../../../api/{entity}API';\n",
                    fn_name = lookup_fn,
                    entity = entity
                ));
            }

            if !mobile_screen_state.contains(&nav_var) {
                mobile_screen_state.push_str(&format!(
                    "  const [{var}Options, set{nav}Options] = useState([]);\n",
                    var = nav_var,
                    nav = nav_short
                ));
                let load_fn = format!("load{nav}Options", nav = nav_short);
                if !mobile_screen_lookup_loaders.contains(&load_fn) {
                    mobile_screen_lookup_loaders.push_str(&format!(
                        "  const {load_fn} = () =>\n    {fn_name}().then(({{ items }} = {{}}) => set{nav}Options(items || []));\n\n",
                        load_fn = load_fn,
                        fn_name = lookup_fn,
                        nav = nav_short
                    ));
                    mobile_screen_effects.push_str(&format!(
                        "  useEffect(() => {{\n    {load_fn}();\n  }}, []);\n\n",
                        load_fn = load_fn
                    ));
                    if !mobile_screen_lookup_calls.contains(&load_fn) {
                        mobile_screen_lookup_calls
                            .push_str(&format!("      {load_fn}(),\n", load_fn = load_fn));
                    }
                }
                mobile_screen_form_props.push_str(&format!(
                    "      {var}Options={{{var}Options}}\n",
                    var = nav_var
                ));
            }

            if !mobile_form_props.contains(&nav_var) {
                mobile_form_props.push_str(&format!("  {var}Options: [];\n", var = nav_var));
                mobile_form_prop_types.push_str(&format!(
                    "  {var}Options: PropTypes.array,\n",
                    var = nav_var
                ));
            }

            mobile_has_combobox = true;
            if !mobile_form_external_imports.contains("ComboBox") {
                mobile_form_external_imports
                    .push_str("import { ComboBox } from '../../../components/ComboBox';\n");
            }
            mobile_form_inputs.push_str(&format!(
                "            <FormControl{required_prop} isInvalid={{isInvalidControl('{field}')}}>\n              <FormControlLabel>\n                <FormControlLabelText>{{i18n.t('{domain_short}::{entity}:{label}')}}</FormControlLabelText>\n              </FormControlLabel>\n              <ComboBox\n                value={{form.values.{field} || ''}}\n                onValueChange={{(val) => form.setFieldValue('{field}', val, true)}}\n                placeholder={{i18n.t('AbpUi::Select')}}\n                options={{{var}Options.map((opt) => ({{ label: opt.displayName, value: opt.id }}))}}\n              />\n              {{isInvalidControl('{field}') && (\n                <ValidationMessage>{{form.errors.{field}}}</ValidationMessage>\n              )}}\n            </FormControl>\n\n",
                field = field,
                label = fname,
                domain_short = domain_short,
                entity = entity,
                required_prop = if f.required { " isRequired" } else { "" },
                var = nav_var
            ));
        } else if is_datetime {
            mobile_has_datepicker = true;
            let field_upper = to_upper_camel(&field);
            mobile_form_state.push_str(&format!(
                "  const [{field}PickerVisible, set{field_upper}PickerVisible] = useState(false);\n",
                field = field,
                field_upper = field_upper
            ));
            mobile_form_modals.push_str(&format!(
                "      {{{field}PickerVisible && (\n        <DateTimePicker\n          value={{form.values.{field} ? new Date(form.values.{field}) : new Date()}}\n          mode=\"date\"\n          is24Hour={{true}}\n          onChange={{(event, selectedDate) => {{\n            set{field_upper}PickerVisible(false);\n            if (event && event.type === 'dismissed') {{\n              return;\n            }}\n            if (selectedDate) {{\n              form.setFieldValue('{field}', selectedDate.toISOString(), true);\n            }}\n          }}}}\n        />\n      )}}\n",
                field = field,
                field_upper = field_upper
            ));
            mobile_form_inputs.push_str(&format!(
                "            <FormControl{required_prop} isInvalid={{isInvalidControl('{field}')}}>\n              <FormControlLabel>\n                <FormControlLabelText>{{i18n.t('{domain_short}::{entity}:{label}')}}</FormControlLabelText>\n              </FormControlLabel>\n              <Input\n                bg={{inputBg}}\n                borderWidth={{1}}\n                borderColor={{inputBorder}}\n                borderRadius=\"$lg\"\n                _dark={{{{ bg: inputBgDark, borderColor: inputBorderDark }}}}\n              >\n                <InputField\n                  value={{form.values.{field} ? new Date(form.values.{field}).toLocaleDateString() : ''}}\n                  editable={{false}}\n                  placeholder={{i18n.t('AbpUi::Select')}}\n                />\n                <InputSlot pr=\"$3\" onPress={{() => set{field_upper}PickerVisible(true)}}>\n                  <Icon as={{Ionicons}} name=\"calendar-outline\" size=\"md\" color=\"$secondary500\" _dark={{{{ color: '$textDarkMuted' }}}} />\n                </InputSlot>\n              </Input>\n              {{isInvalidControl('{field}') && (\n                <ValidationMessage>{{form.errors.{field}}}</ValidationMessage>\n              )}}\n            </FormControl>\n\n",
                field = field,
                field_upper = field_upper,
                label = fname,
                domain_short = domain_short,
                entity = entity,
                required_prop = if f.required { " isRequired" } else { "" }
            ));
        } else if is_textarea {
            _mobile_has_textarea = true;
            if !mobile_form_styles.contains("textAreaInput") {
                mobile_form_styles.push_str(
                    "  textAreaInput: {\n    minHeight: 140,\n    textAlignVertical: 'top',\n    padding: 0,\n    margin: 0,\n    width: '100%',\n  },\n",
                );
            }
            mobile_form_inputs.push_str(&format!(
                "            <FormControl{required_prop} isInvalid={{isInvalidControl('{field}')}}>\n              <FormControlLabel>\n                <FormControlLabelText>{{i18n.t('{domain_short}::{entity}:{label}')}}</FormControlLabelText>\n              </FormControlLabel>\n              <Box\n                borderWidth={{1}}\n                borderColor={{inputBorder}}\n                borderRadius=\"$lg\"\n                bg={{inputBg}}\n                px=\"$3\"\n                py=\"$3\"\n                _dark={{{{ bg: inputBgDark, borderColor: inputBorderDark }}}}\n              >\n                <TextInput\n                  onChangeText={{form.handleChange('{field}')}}\n                  onBlur={{form.handleBlur('{field}')}}\n                  value={{form.values.{field}}}\n                  autoCapitalize=\"none\"\n                  keyboardType=\"default\"\n                  multiline\n                  placeholder=\"\"\n                  style={{styles.textAreaInput}}\n                />\n              </Box>\n              {{isInvalidControl('{field}') && (\n                <ValidationMessage>{{form.errors.{field}}}</ValidationMessage>\n              )}}\n            </FormControl>\n\n",
                field = field,
                label = fname,
                domain_short = domain_short,
                entity = entity,
                required_prop = if f.required { " isRequired" } else { "" }
            ));
        } else if is_bool {
            mobile_has_checkbox = true;
            mobile_form_inputs.push_str(&format!(
                "            <FormControl isInvalid={{isInvalidControl('{field}')}} my=\"$2\">\n              <Checkbox\n                value=\"{field}\"\n                isChecked={{!!form.values.{field}}}\n                onChange={{(value) => form.setFieldValue('{field}', value, true)}}\n              >\n                <CheckboxIndicator>\n                  <CheckboxIcon as={{CheckIcon}} />\n                </CheckboxIndicator>\n                <CheckboxLabel> {{i18n.t('{domain_short}::{entity}:{label}')}} </CheckboxLabel>\n              </Checkbox>\n              {{isInvalidControl('{field}') && (\n                <ValidationMessage>{{form.errors.{field}}}</ValidationMessage>\n              )}}\n            </FormControl>\n\n",
                field = field,
                label = fname,
                domain_short = domain_short,
                entity = entity
            ));
        } else {
            let required_prop = if f.required { " isRequired" } else { "" };
            let keyboard_type = if is_numeric { "numeric" } else { "default" };
            mobile_form_inputs.push_str(&format!(
                "            <FormControl{required_prop} isInvalid={{isInvalidControl('{field}')}}>\n              <FormControlLabel>\n                <FormControlLabelText>{{i18n.t('{domain_short}::{entity}:{label}')}}</FormControlLabelText>\n              </FormControlLabel>\n              <Input\n                bg={{inputBg}}\n                borderWidth={{1}}\n                borderColor={{inputBorder}}\n                borderRadius=\"$lg\"\n                _dark={{{{ bg: inputBgDark, borderColor: inputBorderDark }}}}\n              >\n                <InputField\n                  onChangeText={{form.handleChange('{field}')}}\n                  onBlur={{form.handleBlur('{field}')}}\n                  value={{form.values.{field}}}\n                  autoCapitalize=\"none\"\n                  keyboardType=\"{keyboard_type}\"\n                />\n              </Input>\n              {{isInvalidControl('{field}') && (\n                <ValidationMessage>{{form.errors.{field}}}</ValidationMessage>\n              )}}\n            </FormControl>\n\n",
                field = field,
                label = fname,
                domain_short = domain_short,
                entity = entity,
                required_prop = required_prop,
                keyboard_type = keyboard_type
            ));
        }

        if f.show_in_ui && field != title_field_lower {
            let value_expr = if let Some(navigation) = &f.navigation {
                let nav_lower = to_lower_camel(last_segment(navigation));
                format!(
                    "item.{nav}?.name || entityItem.{field} || ''",
                    nav = nav_lower,
                    field = field
                )
            } else {
                format_mobile_value(ftype, "entityItem", &field)
            };
            mobile_list_fields.push_str(&format!(
                "                  <Text color=\"$secondary500\">{{t('{domain_short}::{entity}:{label}')}}: {{ {value_expr} }}</Text>\n",
                label = fname,
                value_expr = value_expr,
                domain_short = domain_short,
                entity = entity
            ));
        }
    }

    mobile_form_initial_values.push_str(&format!(
        "      ...{entity_lower},\n",
        entity_lower = entity_lower
    ));

    if mobile_has_checkbox {
        mobile_form_extra_imports.push_str(
            "  Checkbox,\n  CheckboxIndicator,\n  CheckboxIcon,\n  CheckboxLabel,\n  CheckIcon,\n",
        );
    }

    if mobile_has_combobox && !mobile_form_external_imports.contains("ComboBox") {
        mobile_form_external_imports
            .push_str("import { ComboBox } from '../../../components/ComboBox';\n");
    }

    if mobile_has_datepicker {
        if !mobile_form_extra_imports.contains("InputSlot") {
            mobile_form_extra_imports.push_str("  InputSlot,\n");
        }
        if !mobile_form_extra_imports.contains("  Icon,") {
            mobile_form_extra_imports.push_str("  Icon,\n");
        }
        if !mobile_form_external_imports.contains("DateTimePicker") {
            mobile_form_external_imports
                .push_str("import DateTimePicker from '@react-native-community/datetimepicker';\n");
        }
        if !mobile_form_external_imports.contains("@expo/vector-icons") {
            mobile_form_external_imports
                .push_str("import { Ionicons } from '@expo/vector-icons';\n");
        }
    }

    for f in fields {
        let field_name = &f.name;
        let field_param = to_lower_camel(field_name);
        let field_entity = entity;

        if f.navigation.is_none() && f.filterable {
            match f.ftype.as_str() {
                "string" | "textarea" | "Textarea" => {
                    repository_nav_filters.push_str(&format!(
                    ".WhereIf(!string.IsNullOrWhiteSpace({p}), x => EF.Functions.Like(x.{entity}.{f}.Trim(), $\"%{{{p}.Trim()}}%\"))\n",
                    p = field_param,
                    entity = field_entity,
                    f = field_name
                ));
                    repository_entity_filters.push_str(&format!(
                    ".WhereIf(!string.IsNullOrWhiteSpace({p}), x => EF.Functions.Like(x.{f}.Trim(), $\"%{{{p}.Trim()}}%\"))\n",
                    p = field_param,
                    f = field_name
                ));
                }
                "int" | "long" | "bool" | "DateTime" | "Guid" | "decimal" => {
                    repository_nav_filters.push_str(&format!(
                        ".WhereIf({p}.HasValue, x => x.{entity}.{f} == {p}.Value)\n",
                        p = field_param,
                        entity = field_entity,
                        f = field_name
                    ));
                    repository_entity_filters.push_str(&format!(
                        ".WhereIf({p}.HasValue, x => x.{f} == {p}.Value)\n",
                        p = field_param,
                        f = field_name
                    ));
                }
                _ => {}
            }
        }

        if let Some(navigation) = f
            .navigation
            .as_deref()
            .filter(|_| f.ftype == "Guid" && f.filterable)
        {
            let nav_prop = last_segment(navigation);
            let nav_param = to_lower_camel(&f.name);

            repository_nav_filters.push_str(&format!(
            ".WhereIf({p} != null && {p} != Guid.Empty, x => x.{nav} != null && x.{nav}.Id == {p})\n",
            p = nav_param,
            nav = nav_prop
        ));
            repository_entity_filters.push_str(&format!(
                ".WhereIf({p} != null && {p} != Guid.Empty, x => x.{f} == {p})\n",
                p = nav_param,
                f = field_name
            ));
        }
    }

    let repository_nav_filters = repository_nav_filters.trim_end().to_string();
    let repository_entity_filters = repository_entity_filters.trim_end().to_string();

    let filter_method_params = filter_method_list.join(", ");
    let repository_filters = repo_filter_list
        .iter()
        .map(|l| format!("{},\n", l))
        .collect::<String>();

    let constructor_params = constructor_params.trim_end_matches(", ").to_string();
    let manager_params_str = manager_params.trim_end_matches(", ").to_string();
    let manager_param_names_source = Regex::new(r"\s*\b[\w\?]+\s+")
        .unwrap()
        .replace_all(&manager_params_str, "");
    let manager_param_names = manager_param_names_source
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .join(", ");
    let constructor_params = if constructor_params.is_empty() {
        String::new()
    } else {
        format!(", {constructor_params}")
    };

    let mut create_dto_props = String::new();
    for f in fields {
        create_dto_props.push_str(&attr_required_and_length(f, entity));
        create_dto_props.push_str(&format!(
            "public {} {} {{ get; set; }}\n",
            dto_type_for_field(f),
            f.name
        ));
    }

    let mut update_dto_props = String::new();
    for f in fields {
        update_dto_props.push_str(&attr_required_and_length(f, entity));
        update_dto_props.push_str(&format!(
            "public {} {} {{ get; set; }}\n",
            dto_type_for_field(f),
            f.name
        ));
    }
    update_dto_props.push_str("public string ConcurrencyStamp { get; set; } = null!;\n");

    let mut dto_props = String::new();
    for f in fields {
        dto_props.push_str(&format!(
            "public {} {} {{ get; set; }}\n",
            dto_type_for_field(f),
            f.name
        ));
    }
    dto_props.push_str("public string ConcurrencyStamp { get; set; } = null!;\n");

    let with_nav_dto_entity_prop =
        format!("public {entity}Dto {entity} {{ get; set; }} = null!;\n");

    let mut with_nav_dto_props = String::new();
    for f in fields {
        if f.ftype == "Guid"
            && let Some(nav) = &f.navigation
        {
            let nav_short = last_segment(nav);
            with_nav_dto_props.push_str(&format!(
                "public {nav_short}Dto? {nav_short} {{ get; set; }} = null!;\n"
            ));
        }
    }

    let mut get_input_props = String::from("public string? FilterText { get; set; }\n\n");
    for f in fields {
        if f.filterable {
            get_input_props.push_str(&format!(
                "public {} {} {{ get; set; }}\n",
                f.nullable_filter_type(),
                f.name
            ));
        }
    }

    let mut excel_download_props = String::from(
        "public string DownloadToken { get; set; } = null!;\n\npublic string? FilterText { get; set; }\n\n",
    );
    for f in fields {
        if f.filterable {
            excel_download_props.push_str(&format!(
                "public {} {} {{ get; set; }}\n",
                f.nullable_filter_type(),
                f.name
            ));
        }
    }

    let mut excel_dto_props = String::new();
    for f in fields {
        excel_dto_props.push_str(&format!(
            "public {} {} {{ get; set; }}\n",
            dto_type_for_field(f),
            f.name
        ));
    }

    let mut input_filter_call_args = "input.FilterText".to_string();
    let chosen: Vec<String> = fields
        .iter()
        .filter(|f| f.filterable)
        .map(|f| format!("input.{}", f.name))
        .collect();
    if !chosen.is_empty() {
        input_filter_call_args.push_str(", ");
        input_filter_call_args.push_str(&chosen.join(", "));
    }
    let appservice_create_args = fields
        .iter()
        .map(|f| format!("input.{}", f.name))
        .join(", ");
    let appservice_update_args = format!("{}, input.ConcurrencyStamp", appservice_create_args);

    let mut required_guid_checks = String::new();
    for f in fields {
        if f.ftype == "Guid" && f.required {
            let label = f
                .navigation
                .as_ref()
                .map(|s| last_segment(s).to_string())
                .unwrap_or_else(|| f.name.clone());
            required_guid_checks.push_str(&format!(
                "if (input.{n} == default)\n    throw new UserFriendlyException(L[\"The {{0}} field is required.\", L[\"{label}\"]].Value);\n\n",
                n = f.name
            ));
        }
    }

    let mut nav_repo_fields = String::new();
    let mut nav_repo_ctor_params = String::new();
    let mut nav_repo_ctor_assignments = String::new();
    let mut nav_lookup_methods = String::new();
    let mut nav_lookup_interface_methods = String::new();
    let mut nav_lookup_profile_maps = String::new();
    let mut nav_lookup_mapperly_blocks = String::new();
    let mut excel_export_pairs = String::new();
    let mut nav_usings_set = Vec::<String>::new();
    let mut nav_lookup_mapperly_navs = std::collections::HashMap::<String, String>::new();

    for f in fields {
        if f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea") {
            excel_export_pairs.push_str(&format!(
                "{{ \"{n}\", item.{e}.{n} }},\n",
                n = f.name,
                e = entity
            ));
        }

        if f.ftype == "Guid"
            && let Some((nav_full, nav_display_override)) = nav_parts(f)
        {
            let nav_short = last_segment(&nav_full);
            let nav_var = to_lower_camel(nav_short);

            let first_for_nav = !nav_lookup_mapperly_navs.contains_key(nav_short);
            let display_prop = nav_display_override
                .map(|s| s.to_string())
                .unwrap_or_else(|| first_string_prop_of(out_root, domain, &nav_full));

            nav_lookup_mapperly_navs
                .entry(nav_short.to_string())
                .or_insert_with(|| display_prop.clone());

            if first_for_nav {
                nav_repo_fields.push_str(&format!(
                    "protected IRepository<{ns}, Guid> _{v}Repository;\n",
                    ns = nav_short,
                    v = nav_var
                ));
                if nav_repo_ctor_params.is_empty() {
                    nav_repo_ctor_params = format!(
                        ",\n            IRepository<{ns}, Guid> {v}Repository",
                        ns = nav_short,
                        v = nav_var
                    );
                } else {
                    nav_repo_ctor_params.push_str(&format!(
                        ",\n            IRepository<{ns}, Guid> {v}Repository",
                        ns = nav_short,
                        v = nav_var
                    ));
                }
                nav_repo_ctor_assignments
                    .push_str(&format!("_{v}Repository = {v}Repository;\n", v = nav_var));

                nav_lookup_profile_maps.push_str(&format!(
                        "\n        CreateMap<{ns}, LookupDto<Guid>>()\
                .ForMember(dest => dest.DisplayName, opt => opt.MapFrom(src => src.{prop} ?? src.Name));\n",
                        ns = nav_short,
                        prop = display_prop
                    ));

                let display_is_string = prop_is_string(out_root, domain, &nav_full, &display_prop);
                let filter_clause = if display_is_string {
                    format!(
                        "x.{prop} != null && x.{prop}.Contains(input.Filter)",
                        prop = display_prop
                    )
                } else {
                    format!(
                        "x.{prop}.ToString().Contains(input.Filter)",
                        prop = display_prop
                    )
                };
                nav_lookup_methods.push_str(&format!(
                        "public virtual async Task<PagedResultDto<LookupDto<Guid>>> Get{ns}LookupAsync(LookupRequestDto input)\n{{\n    var query = await _{v}Repository.GetQueryableAsync();\n\n    if (!string.IsNullOrWhiteSpace(input.Filter))\n    {{\n        query = query.Where(x => {filter});\n    }}\n\n    query = query.OrderBy(x => x.{prop});\n\n    var lookupData = await query.PageBy(input.SkipCount, input.MaxResultCount).ToDynamicListAsync<{ns}>();\n    var totalCount = query.Count();\n\n    return new PagedResultDto<LookupDto<Guid>>\n    {{\n        TotalCount = totalCount,\n        Items = ObjectMapper.Map<List<{ns}>, List<LookupDto<Guid>>>(lookupData)\n    }};\n}}\n\n",
                        ns = nav_short,
                        v = nav_var,
                        prop = display_prop,
                        filter = filter_clause
                    ));

                nav_lookup_interface_methods.push_str(&format!(
                        "Task<PagedResultDto<LookupDto<Guid>>> Get{ns}LookupAsync(LookupRequestDto input);\n",
                        ns = nav_short
                    ));

                let nav_prop_is_string = prop_is_string(out_root, domain, &nav_full, &display_prop);
                let nav_value = if nav_prop_is_string {
                    format!("item.{nav}?.{prop}", nav = nav_short, prop = display_prop)
                } else {
                    format!(
                        "item.{nav}?.{prop}.ToString()",
                        nav = nav_short,
                        prop = display_prop
                    )
                };
                excel_export_pairs.push_str(&format!(
                    "{{ \"{ns}\", {val} }},\n",
                    ns = nav_short,
                    val = nav_value
                ));
            }

            let explicit_ns = ns_of(&nav_full);
            if !explicit_ns.is_empty() {
                let u = format!("using {};", explicit_ns);
                if !nav_usings_set.contains(&u) {
                    nav_usings_set.push(u);
                }
            } else {
                let guessed_ns = format!("{}.{}", domain, pluralize(nav_short));
                let u = format!("using {};", guessed_ns);
                if !nav_usings_set.contains(&u) {
                    nav_usings_set.push(u);
                }
            }
        }
    }

    if !nav_lookup_mapperly_navs.is_empty() {
        let mut blocks: Vec<String> = nav_lookup_mapperly_navs
            .iter()
            .map(|(ns, prop)| {
                format!(
                    "\n// <auto-nav-lookup-{ns}>\n[Mapper]\npublic partial class {ns}ToLookupDtoMapper : MapperBase<{ns}, LookupDto<Guid>>\n{{\n    [MapProperty(nameof({ns}.{prop}), nameof(LookupDto<Guid>.DisplayName))]\n    public override partial LookupDto<Guid> Map({ns} source);\n    [MapProperty(nameof({ns}.{prop}), nameof(LookupDto<Guid>.DisplayName))]\n    public override partial void Map({ns} source, LookupDto<Guid> destination);\n}}\n// </auto-nav-lookup-{ns}>\n"
                )
            })
            .collect();
        blocks.sort();
        nav_lookup_mapperly_blocks = blocks.join("\n");
    }

    if excel_export_pairs.is_empty() {
        excel_export_pairs.push_str(&format!(
            "{{ \"Id\", item.{e}.Id.ToString() }},\n",
            e = entity
        ));
    }

    let entity_ns = namespace.to_string();
    let entity_display_prop =
        first_string_prop_of(out_root, domain, &format!("{}.{}", entity_ns, entity));

    let nav_usings = nav_usings_set.iter().sorted().cloned().join("\n");
    let with_nav_entity_prop = format!("public {e} {e} {{ get; set; }} = null!;\n", e = entity);

    let mut efcore_property_configs = String::new();
    for f in fields {
        efcore_property_configs.push_str(&format!(
            "b.Property(x => x.{name}).HasColumnName(nameof({entity}.{name}))",
            name = f.name,
            entity = entity
        ));

        if (f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea")) && f.required {
            efcore_property_configs.push_str(".IsRequired()");
        }

        if (f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea"))
            && let Some(_max) = f.max_length
        {
            efcore_property_configs.push_str(&format!(
                ".HasMaxLength({entity}Consts.{n}MaxLength)",
                n = f.name
            ));
        }

        efcore_property_configs.push_str(";\n");
    }

    let mut efcore_len_configs = String::new();
    for f in fields {
        if (f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea"))
            && f.max_length.is_some()
        {
            efcore_len_configs.push_str(&format!(
                "b.Property(x => x.{n}).HasMaxLength({e}Consts.{n}MaxLength);\n",
                n = f.name,
                e = entity
            ));
        }
    }

    // ---------------------- INDEX (Web) fragments ----------------------

    let mut web_index_filter_fields = String::new();

    let mut index_cs_nav_props = String::new();
    let mut index_cs_onget_nav_fill = String::new();

    let mut web_index_column_fields = String::new();
    let mut web_datatable_columns = String::new();
    let mut index_js_nav_filters = String::new();
    let mut index_js_nav_query_pairs = String::new();
    let mut index_js_nav_clear_filters = String::new();
    let mut index_js_string_filters = String::new();
    let mut index_js_string_query_pairs = String::new();
    let mut index_js_string_clear_filters = String::new();
    let mut index_js_select2_nav_filters = String::new();

    for f in fields {
        if (f.ftype == "string" || f.ftype.eq_ignore_ascii_case("textarea")) && f.filterable {
            let fname = &f.name;
            let fname_lc = to_lower_camel(fname);

            index_cs_nav_props.push_str(&format!(
                "        public string? {n}Filter {{ get; set; }}\n\n",
                n = fname
            ));

            web_index_filter_fields.push_str(&format!(
                r#"<abp-column size="_3">
  <abp-input asp-for="{n}Filter" label="@L["{e}:{n}"].Value" />
</abp-column>
"#,
                e = entity,
                n = fname
            ));

            let input_id = format!("{}Filter", fname);
            index_js_string_filters.push_str(&format!(
                "      {k}: $(\"#{id}\").val(),\n",
                k = fname_lc,
                id = input_id
            ));
            index_js_string_query_pairs.push_str(&format!(
                "          {{ name: '{k}', value: input.{k} }},\n",
                k = fname_lc
            ));
            index_js_string_clear_filters
                .push_str(&format!("    $(\"#{id}\").val(\"\");\n", id = input_id));
        }

        if let Some(nav_full) = f
            .navigation
            .as_deref()
            .filter(|_| f.ftype == "Guid" && f.filterable)
        {
            let nav_short = last_segment(nav_full).to_string();
            let nav_lower = to_lower_camel(&nav_short);

            index_cs_nav_props.push_str(&format!(
r#"
        [SelectItems(nameof({n}LookupList))]
        public Guid {n}IdFilter {{ get; set; }}
        public List<SelectListItem> {n}LookupList {{ get; set; }} = new List<SelectListItem> {{ new SelectListItem(string.Empty, "") }};
"#,
            n = nav_short
        ));

            index_cs_onget_nav_fill.push_str(&format!(
r#"            {n}LookupList.AddRange((
                await _{el}AppService.Get{n}LookupAsync(
                    new LookupRequestDto {{ MaxResultCount = LimitedResultRequestDto.MaxMaxResultCount }}
                )).Items
                .Select(t => new SelectListItem(t.DisplayName, t.Id.ToString()))
                .ToList()
            );
"#,
            n = nav_short,
            el = entity_lower
        ));

            web_index_filter_fields.push_str(&format!(
                r#"<abp-column size="_3">
  <div class="mb-3">
    <label class="form-label">@L["{e}:{field}"].Value</label>
    <select id="{n}IdFilter" name="{n}IdFilter" class="form-control select2">
    </select>
  </div>
</abp-column>
"#,
                e = entity,
                n = nav_short,
                field = f.name
            ));

            let field_kebab = to_kebab(&nav_short);
            let lookup_endpoint = format!(
                "api/app/{epk}/{fk}-lookup",
                epk = entity_plural_kebab,
                fk = field_kebab
            );

            index_js_select2_nav_filters.push_str(&format!(
r#"  $('#{n}IdFilter').select2({{
    ajax: {{
      url: abp.appPath + '{endpoint}',
      type: 'GET',
      data: function (params) {{ return {{ filter: params.term, maxResultCount: 10 }}; }},
      processResults: function (data) {{
        var mapped = _.map(data.items, function (item) {{ return {{ id: item.id, text: item.displayName }}; }});
        return {{ results: [{{ id: '', text: '' }}].concat(mapped) }};
      }}
    }}
  }});
"#,
                n = nav_short,
                endpoint = lookup_endpoint
            ));

            index_js_nav_filters.push_str(&format!(
                "      {nl}Id: $('#{n}IdFilter').val(),\n",
                nl = nav_lower,
                n = nav_short
            ));
            index_js_nav_query_pairs.push_str(&format!(
                "          {{ name: '{nl}Id', value: input.{nl}Id }},\n",
                nl = nav_lower
            ));
            index_js_nav_clear_filters.push_str(&format!(
            "    (function(){{ const $el=$('#{n}IdFilter'); if($el.length) $el.val('').trigger('change'); }})();\n",
            n = nav_short
        ));
        }

        if f.show_in_ui {
            let fname = &f.name;
            let fname_l = to_lower_camel(fname);

            if let Some(nav_full) = f.navigation.as_deref().filter(|_| f.ftype == "Guid") {
                let nav_name = last_segment(nav_full);
                let nav_lc = to_lower_camel(nav_name);
                let disp_field = first_string_prop_of(out_root, domain, nav_full);
                let disp_lc = to_lower_camel(&disp_field);

                web_index_column_fields.push_str(&format!(
                    "          <th>@L[\"{e}:{field}\"].Value</th>\n",
                    e = entity,
                    field = f.name
                ));
                web_datatable_columns.push_str(&format!(
                    "    {{ data: \"{nav_lc}.{disp_lc}\", defaultContent: \"\", render: DataTable.render.text() }},\n",
                    nav_lc = nav_lc,
                    disp_lc = disp_lc
                ));
            } else {
                web_index_column_fields.push_str(&format!(
                    "          <th>@L[\"{e}:{n}\"].Value</th>\n",
                    e = entity,
                    n = fname
                ));
                web_datatable_columns.push_str(&format!(
                    "    {{ data: \"{el}.{fl}\", defaultContent: \"\", render: DataTable.render.text() }},\n",
                    el = entity_lower,
                    fl = fname_l
                ));
            }
        }
    }

    let web_datatable_columns = web_datatable_columns
        .trim_end()
        .trim_end_matches(',')
        .to_string();
    let index_js_nav_filters = index_js_nav_filters.trim_end().to_string();
    let index_js_nav_query_pairs = index_js_nav_query_pairs.trim_end().to_string();
    let index_js_nav_clear_filters = index_js_nav_clear_filters.trim_end().to_string();

    // ---------------------- WEB FORM & JS PLACEHOLDERS ----------------------

    let mut web_form_fields_create = String::new();
    let mut web_form_fields_edit = String::new();
    let mut web_form_fields_detail = String::new();
    let mut web_js_select2_create = String::new();
    let mut web_js_select2_edit = String::new();
    let mut web_js_select2_detail = String::new();

    let mut nav_dtos = String::new();
    let mut nav_dto_mappings = String::new();

    for f in fields {
        let name = &f.name;
        let is_nav = f.ftype == "Guid" && f.navigation.is_some();
        let label_text = if f.required {
            format!(
                "@L[\"{entity}:{field}\"].Value *",
                entity = entity,
                field = name
            )
        } else {
            format!(
                "@L[\"{entity}:{field}\"].Value",
                entity = entity,
                field = name
            )
        };

        if is_nav {
            let nav_short = last_segment(f.navigation.as_ref().unwrap());
            let star = if f.required { "*" } else { "" };

            web_form_fields_create.push_str(&format!(
                r#"<div class="mb-3">
  <label for="{e}_{n}" class="form-label">@L["{e}:{n}"].Value {star}</label>
  <div class="form-group">
    <select id="{e}_{n}"
            class="custom-select form-control"
            name="{e}.{n}"
            data-value="@Model.{e}.{n}" {req}>
    </select>
    <span asp-validation-for="{e}.{n}" class="text-danger"></span>
  </div>
</div>
"#,
                e = entity,
                n = name,
                star = star,
                req = if f.required { "required" } else { "" }
            ));

            web_form_fields_edit.push_str(&format!(
                r#"<div class="mb-3">
  <div class="form-group">
    <label class="form-label" for="{e}_{n}">@L["{e}:{n}"].Value {star}</label>
    <select id="{e}_{n}"
            name="{e}.{n}"
            class="custom-select form-control"
            data-value="@Model.{e}.{n}" {req}>
      @if (Model.{ns} != null)
      {{
        <option value="@Model.{e}.{n}">@Model.{ns}.Name</option>
      }}
    </select>
    <span asp-validation-for="{e}.{n}" class="text-danger"></span>
  </div>
</div>
"#,
                e = entity,
                n = name,
                ns = nav_short,
                star = star,
                req = if f.required { "required" } else { "" }
            ));

            web_form_fields_detail.push_str(&format!(
                r#"<div class="mb-3">
  <div class="form-group">
    <label class="form-label" for="{e}_{n}">@L["{e}:{n}"].Value {star}</label>
    <select id="{e}_{n}"
            name="{e}.{n}"
            class="custom-select form-control"
            data-value="@Model.{e}.{n}"
            disabled="true">
      @if (Model.{ns} != null)
      {{
        <option value="@Model.{e}.{n}">@Model.{ns}.Name</option>
      }}
    </select>
  </div>
</div>
"#,
                e = entity,
                n = name,
                ns = nav_short,
                star = star
            ));

            let field_kebab = to_kebab(nav_short);
            let lookup_endpoint = format!(
                "api/app/{epk}/{fk}-lookup",
                epk = entity_plural_kebab,
                fk = field_kebab
            );

            web_js_select2_create.push_str(&format!(
r#"$('#{e}_{n}').select2({{
  dropdownParent: $('#{e}CreateModal'),
  ajax: {{
    url: abp.appPath + '{endpoint}',
    type: 'GET',
    data: function (params) {{ return {{ filter: params.term, maxResultCount: 10 }}; }},
    processResults: function (data) {{
      var mapped = _.map(data.items, function (item) {{ return {{ id: item.id, text: item.displayName }}; }});
      return {{ results: mapped }};
    }}
  }}
}});
"#,
                e = entity,
                n = name,
                endpoint = lookup_endpoint
            ));

            web_js_select2_edit.push_str(&format!(
r#"$('#{e}_{n}').select2({{
  dropdownParent: $('#{e}EditModal'),
  ajax: {{
    url: abp.appPath + '{endpoint}',
    type: 'GET',
    data: function (params) {{ return {{ filter: params.term, maxResultCount: 10 }}; }},
    processResults: function (data) {{
      var mapped = _.map(data.items, function (item) {{ return {{ id: item.id, text: item.displayName }}; }});
      return {{ results: mapped }};
    }}
  }}
}});
"#,
                e = entity,
                n = name,
                endpoint = lookup_endpoint
            ));

            web_js_select2_detail.push_str(&format!(
r#"$('#{e}_{n}').select2({{
  dropdownParent: $('#{e}DetailModal'),
  ajax: {{
    url: abp.appPath + '{endpoint}',
    type: 'GET',
    data: function (params) {{ return {{ filter: params.term, maxResultCount: 10 }}; }},
    processResults: function (data) {{
      var mapped = _.map(data.items, function (item) {{ return {{ id: item.id, text: item.displayName }}; }});
      return {{ results: mapped }};
    }}
  }}
}}).prop('disabled', true);
"#,
                e = entity,
                n = name,
                endpoint = lookup_endpoint
            ));

            nav_dtos.push_str(&format!(
                "public {ns}Dto {ns} {{ get; set; }}\n",
                ns = nav_short
            ));
            nav_dto_mappings.push_str(&format!(
                "{ns} = {el}WithNavigationPropertiesDto.{ns};\n",
                ns = nav_short,
                el = entity_lower
            ));
        } else {
            match f.ftype.as_str() {
                "textarea" | "Textarea" => {
                    web_form_fields_create.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" text-area label=\"{label}\" />",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                    web_form_fields_create.push('\n');

                    web_form_fields_edit.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" text-area label=\"{label}\" />",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                    web_form_fields_edit.push('\n');

                    web_form_fields_detail.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" text-area label=\"{label}\" disabled=\"true\" />",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                    web_form_fields_detail.push('\n');
                }
                "DateTime" | "datetime" => {
                    web_form_fields_create.push_str(&format!(
                "<abp-date-picker asp-for=\"{e}.{n}\" label=\"{label}\" single-open-and-clear-button=\"false\" week-numbers=\"Iso\" time-picker=\"true\" />\n",
                e = entity,
                n = name,
                label = label_text
            ));
                    web_form_fields_edit.push_str(&format!(
                "<abp-date-picker asp-for=\"{e}.{n}\" label=\"{label}\" single-open-and-clear-button=\"false\" week-numbers=\"Iso\" time-picker=\"true\" />\n",
                e = entity,
                n = name,
                label = label_text
            ));
                    web_form_fields_detail.push_str(&format!(
                "<abp-date-picker asp-for=\"{e}.{n}\" label=\"{label}\" single-open-and-clear-button=\"false\" week-numbers=\"Iso\" time-picker=\"true\" disabled=\"true\" />\n",
                e = entity,
                n = name,
                label = label_text
            ));
                }
                _ => {
                    web_form_fields_create.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" label=\"{label}\" />\n",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                    web_form_fields_edit.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" label=\"{label}\" />\n",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                    web_form_fields_detail.push_str(&format!(
                        "<abp-input asp-for=\"{e}.{n}\" label=\"{label}\" disabled=\"true\" />\n",
                        e = entity,
                        n = name,
                        label = label_text
                    ));
                }
            }
        }
    }

    let entity_camel = to_lower_camel(entity);
    let entity_plural_camel = to_lower_camel(&entity_plural);
    let localization_resource = format!("{}Resource", domain_short);
    let rel_path = ns_relative_path(domain, namespace);

    let mut map = HashMap::new();
    map.insert("ns_rel_path".into(), rel_path.to_string_lossy().into());
    map.insert("domain_name".into(), domain.into());
    map.insert("domain_short".into(), domain_short.clone());
    map.insert("namespace".into(), namespace.into());
    map.insert("entity".into(), entity.to_string());
    map.insert("entity_lower".into(), entity_lower.clone());
    map.insert("entity_name".into(), entity.into());
    map.insert("entity_name_lower".into(), entity_lower.clone());
    map.insert("entity_plural".into(), entity_plural.clone());
    map.insert("entity_plural_lower".into(), entity_plural_lower);
    map.insert("entity_plural_kebab".into(), entity_plural_kebab);

    map.insert("fields".into(), fields_decl.trim().into());
    map.insert("constructor_params".into(), constructor_params);
    map.insert(
        "fields_assignments".into(),
        fields_assignments.trim().into(),
    );
    map.insert("manager_params".into(), manager_params_str);
    map.insert("manager_param_names".into(), manager_param_names);
    map.insert("manager_checks".into(), manager_checks.trim().into());
    map.insert(
        "manager_assignments".into(),
        manager_assignments.trim().into(),
    );

    map.insert(
        "repository_filters".into(),
        repository_filters.trim().into(),
    );
    map.insert("filter_method_params".into(), filter_method_params);
    map.insert("filter_param_names".into(), filter_name_list.join(", "));

    map.insert(
        "navigation_properties".into(),
        navigation_decls.trim().into(),
    );
    map.insert("with_nav_properties".into(), navigation_decls.trim().into());
    map.insert("navigation_joins".into(), navigation_joins.trim().into());
    map.insert(
        "navigation_selects_list".into(),
        nav_selects_list.trim().into(),
    );
    map.insert(
        "navigation_selects_single".into(),
        nav_selects_single.trim().into(),
    );
    map.insert("consts".into(), const_decls.trim().into());

    map.insert(
        "repository_nav_filters".into(),
        repository_nav_filters.trim().into(),
    );
    map.insert(
        "repository_entity_filters".into(),
        repository_entity_filters.trim().into(),
    );

    map.insert("create_dto_props".into(), create_dto_props.trim().into());
    map.insert("update_dto_props".into(), update_dto_props.trim().into());
    map.insert("dto_props".into(), dto_props.trim().into());

    map.insert(
        "with_nav_dto_entity_prop".into(),
        with_nav_dto_entity_prop.trim().into(),
    );
    map.insert(
        "with_nav_dto_props".into(),
        with_nav_dto_props.trim().into(),
    );
    map.insert("get_input_props".into(), get_input_props.trim().into());
    map.insert(
        "excel_download_props".into(),
        excel_download_props.trim().into(),
    );
    map.insert("excel_dto_props".into(), excel_dto_props.trim().into());

    map.insert("input_filter_call_args".into(), input_filter_call_args);
    map.insert("appservice_create_args".into(), appservice_create_args);
    map.insert("appservice_update_args".into(), appservice_update_args);
    map.insert(
        "required_guid_checks".into(),
        required_guid_checks.trim().into(),
    );

    map.insert("nav_repo_fields".into(), nav_repo_fields.trim().into());
    map.insert("nav_repo_ctor_params".into(), nav_repo_ctor_params);
    map.insert(
        "nav_repo_ctor_assignments".into(),
        nav_repo_ctor_assignments.trim().into(),
    );
    map.insert(
        "entity_lookup_display_prop".into(),
        entity_display_prop.clone(),
    );
    map.insert(
        "nav_lookup_methods".into(),
        nav_lookup_methods.trim().into(),
    );
    map.insert(
        "nav_lookup_interface_methods".into(),
        nav_lookup_interface_methods.trim().into(),
    );
    map.insert(
        "nav_lookup_profile_maps".into(),
        nav_lookup_profile_maps.trim().into(),
    );
    map.insert(
        "nav_lookup_mapperly_blocks".into(),
        nav_lookup_mapperly_blocks.trim().into(),
    );

    map.insert(
        "excel_export_pairs".into(),
        excel_export_pairs.trim().into(),
    );
    map.insert("nav_usings".into(), nav_usings);

    map.insert(
        "with_nav_entity_prop".into(),
        with_nav_entity_prop.trim().into(),
    );

    map.insert(
        "navigation_fk_configs".into(),
        navigation_fk_configs.trim().into(),
    );
    map.insert(
        "efcore_property_configs".into(),
        efcore_property_configs.trim().into(),
    );
    map.insert(
        "efcore_len_configs".into(),
        efcore_len_configs.trim().into(),
    );

    map.insert(
        "web_index_filter_fields".into(),
        web_index_filter_fields.trim().into(),
    );
    map.insert(
        "web_index_column_fields".into(),
        web_index_column_fields.trim().into(),
    );
    map.insert(
        "index_cs_nav_props".into(),
        index_cs_nav_props.trim().into(),
    );
    map.insert(
        "index_cs_onget_nav_fill".into(),
        index_cs_onget_nav_fill.trim().into(),
    );
    map.insert("index_js_nav_filters".into(), index_js_nav_filters);
    map.insert("web_datatable_columns".into(), web_datatable_columns);
    map.insert("index_js_nav_query_pairs".into(), index_js_nav_query_pairs);
    map.insert(
        "index_js_nav_clear_filters".into(),
        index_js_nav_clear_filters,
    );
    map.insert(
        "index_js_select2_nav_filters".into(),
        index_js_select2_nav_filters,
    );
    map.insert(
        "index_js_string_filters".into(),
        index_js_string_filters.trim().into(),
    );
    map.insert(
        "index_js_string_query_pairs".into(),
        index_js_string_query_pairs.trim().into(),
    );
    map.insert(
        "index_js_string_clear_filters".into(),
        index_js_string_clear_filters.trim().into(),
    );
    map.insert(
        "web_form_fields_create".into(),
        web_form_fields_create.trim().into(),
    );
    map.insert(
        "web_form_fields_edit".into(),
        web_form_fields_edit.trim().into(),
    );
    map.insert(
        "web_form_fields_detail".into(),
        web_form_fields_detail.trim().into(),
    );
    map.insert(
        "web_js_select2_create".into(),
        web_js_select2_create.trim().into(),
    );
    map.insert(
        "web_js_select2_edit".into(),
        web_js_select2_edit.trim().into(),
    );
    map.insert(
        "web_js_select2_detail".into(),
        web_js_select2_detail.trim().into(),
    );
    map.insert("nav_dtos".into(), nav_dtos.trim().into());
    map.insert("nav_dto_mappings".into(), nav_dto_mappings.trim().into());
    map.insert("localization_resource".into(), localization_resource);
    map.insert("entity_camel".into(), entity_camel);
    map.insert("entity_plural_camel".into(), entity_plural_camel);

    let web_ns = format!("{}.Web.Pages.{}", domain, entity_plural);
    map.insert("web_namespace".into(), web_ns);

    let namespace_lower = namespace
        .split('.')
        .map(|seg| {
            if seg.is_empty() {
                "".to_string()
            } else {
                let mut chs = seg.chars();
                match chs.next() {
                    Some(f) => f.to_lowercase().collect::<String>() + chs.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(".");
    map.insert("namespace_lower".into(), namespace_lower);

    // Mobile-specific placeholders (React Native)
    map.insert(
        "mobile_lookup_methods".into(),
        mobile_lookup_methods.trim().into(),
    );
    map.insert("mobile_tab_scenes".into(), String::new());
    map.insert("mobile_tab_route_builder".into(), String::new());
    // RN placeholders and defaults
    map.insert(
        "mobile_screen_extra_imports".into(),
        mobile_screen_extra_imports.trim().into(),
    );
    map.insert("mobile_screen_props".into(), String::new());
    map.insert(
        "mobile_screen_state".into(),
        mobile_screen_state.trim().into(),
    );
    map.insert(
        "mobile_screen_lookup_loaders".into(),
        mobile_screen_lookup_loaders.trim().into(),
    );
    map.insert(
        "mobile_screen_lookup_calls".into(),
        mobile_screen_lookup_calls.trim().into(),
    );
    map.insert(
        "mobile_screen_effects".into(),
        mobile_screen_effects.trim().into(),
    );
    map.insert(
        "mobile_screen_form_props".into(),
        mobile_screen_form_props.trim().into(),
    );
    map.insert(
        "mobile_form_validations".into(),
        mobile_form_validations.trim().into(),
    );
    map.insert("mobile_form_props".into(), mobile_form_props.trim().into());
    map.insert("mobile_form_state".into(), mobile_form_state.trim().into());
    map.insert("mobile_form_refs".into(), mobile_form_refs.trim().into());
    map.insert(
        "mobile_form_initial_values".into(),
        mobile_form_initial_values.trim().into(),
    );
    map.insert(
        "mobile_form_modals".into(),
        mobile_form_modals.trim().into(),
    );
    map.insert(
        "mobile_form_inputs".into(),
        mobile_form_inputs.trim().into(),
    );
    map.insert("mobile_form_styles".into(), String::new());
    map.insert(
        "mobile_form_prop_types".into(),
        mobile_form_prop_types.trim().into(),
    );
    map.insert(
        "mobile_form_extra_imports".into(),
        mobile_form_extra_imports.trim_end().into(),
    );
    map.insert(
        "mobile_form_external_imports".into(),
        mobile_form_external_imports.trim_end().into(),
    );
    // Basic mobile list fallbacks
    let title_field = first_string_field
        .clone()
        .unwrap_or_else(|| "Id".to_string());
    let title_expr = to_lower_camel(&title_field);
    map.insert(
        "mobile_list_title".into(),
        format!("entityItem.{title_expr}"),
    );
    // Use a JS literal to avoid invalid output when no description field exists.
    map.insert("mobile_list_description".into(), "null".to_string());
    map.insert(
        "mobile_list_fields".into(),
        mobile_list_fields.trim().into(),
    );

    // RN placeholders that map to existing values
    map.insert("entity_name_lower".into(), entity_lower.clone());
    map.insert("entity_name_kebab".into(), to_kebab(entity));
    map.insert("entity_plural_camel".into(), to_lower_camel(&entity_plural));
    map.insert("domain_short".into(), domain_short.clone());

    // RN route placeholders
    map.insert(
        "mobile_tab_scenes".into(),
        String::new(), // can be extended when multi-tab needed
    );
    map.insert("mobile_tab_route_builder".into(), String::new());

    map
}

/* ----------------------------- GENERATORS ----------------------------- */

pub(crate) fn generate_from(
    context: &GenerationContext<'_>,
    ui_target: UiTarget,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let out_root = &context.out_root;
    let domain = context.domain;
    let entity = context.entity;
    let fields = context.fields;
    let entity_name = mapping
        .get("entity")
        .cloned()
        .unwrap_or_else(|| entity.to_string());
    let entity_plural = mapping
        .get("entity_plural")
        .cloned()
        .unwrap_or_else(|| pluralize(&entity_name));

    fn process_embedded(
        prefix: &str,
        out_root: &Path,
        domain: &str,
        suffix: &str,
        mapping: &HashMap<String, String>,
        rename: &HashMap<&str, String>,
        log: &mut dyn FnMut(&str),
    ) -> Result<()> {
        let rel = mapping.get("ns_rel_path").cloned().unwrap_or_default();
        let base = out_root.join(format!("{}.{}", domain, suffix));
        let out_dir = if rel.is_empty() { base } else { base.join(rel) };
        fs::create_dir_all(&out_dir)?;
        let files = embedded_walk(prefix);
        for rel in files {
            let path = Path::new(&rel);
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let parent = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str());
            let target_dir = if parent == Some("Shared") {
                let dir = out_dir.join("Shared");
                fs::create_dir_all(&dir)?;
                dir
            } else {
                out_dir.clone()
            };

            let out_name = rename
                .get(stem)
                .cloned()
                .unwrap_or_else(|| format!("{stem}.cs"));

            let content = render_template(&read_tpl_text(&rel)?, mapping);
            let out_path = target_dir.join(out_name);
            write_text(&out_path, &content)?;
            log(&format!("created: {}", out_path.display()));
        }
        Ok(())
    }

    let mut ren = HashMap::from([
        ("Entity", format!("{entity}.cs")),
        ("IRepository", format!("I{entity}Repository.cs")),
        (
            "WithNavigationProperties",
            format!("{entity}WithNavigationProperties.cs"),
        ),
        ("Manager", format!("{entity}Manager.cs")),
    ]);
    process_embedded("Domain", out_root, domain, "Domain", mapping, &ren, log)?;

    ren = HashMap::from([("Consts", format!("{entity}Consts.cs"))]);
    process_embedded(
        "Domain.Shared",
        out_root,
        domain,
        "Domain.Shared",
        mapping,
        &ren,
        log,
    )?;
    ensure_domain_shared_localization(out_root, domain, mapping, fields, log)?;

    let shared_files = embedded_walk("Application.Contracts/Shared");
    if !shared_files.is_empty() {
        let shared_out_dir = out_root
            .join(format!("{}.Application.Contracts", domain))
            .join("Shared");
        fs::create_dir_all(&shared_out_dir)?;
        for rel in shared_files {
            let stem = Path::new(&rel)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let out_path = shared_out_dir.join(format!("{stem}.cs"));
            if !out_path.exists() {
                let content = render_template(&read_tpl_text(&rel)?, mapping);
                write_text(&out_path, &content)?;
                log(&format!("created: {}", out_path.display()));
            } else {
                log(&format!("skip: {}", out_path.display()));
            }
        }
    }

    ren = HashMap::from([
        ("EfCoreRepository", format!("EfCore{entity}Repository.cs")),
        (
            "EfCoreModelCreatingExtension",
            format!("EfCore{entity}ModelCreatingExtension.cs"),
        ),
    ]);
    process_embedded(
        "EntityFrameworkCore",
        out_root,
        domain,
        "EntityFrameworkCore",
        mapping,
        &ren,
        log,
    )?;

    ren = HashMap::from([
        ("AppService", format!("{}AppService.cs", entity_plural)),
        (
            "AppServiceInterface",
            format!("I{}AppService.cs", entity_plural),
        ),
        ("Dto", format!("{entity_name}Dto.cs")),
        ("CreateDto", format!("{entity_name}CreateDto.cs")),
        ("UpdateDto", format!("{entity_name}UpdateDto.cs")),
        ("ExcelDto", format!("{entity_name}ExcelDto.cs")),
        ("FilterDto", format!("{entity_name}FilterDto.cs")),
        ("LookupDto", format!("{entity_name}LookupDto.cs")),
        (
            "WithNavigationPropertiesDto",
            format!("{entity_name}WithNavigationPropertiesDto.cs"),
        ),
        ("Profile", format!("{entity_name}AutoMapperProfile.cs")),
        ("ExcelExporter", format!("{entity_name}ExcelExporter.cs")),
        (
            "DownloadTokenCacheItem",
            format!("{entity_name}DownloadTokenCacheItem.cs"),
        ),
    ]);
    process_embedded(
        "Application",
        out_root,
        domain,
        "Application",
        mapping,
        &ren,
        log,
    )?;

    ren = HashMap::from([
        ("Permissions", format!("{}Permissions.cs", entity_plural)),
        ("Localization", format!("{}Localization.cs", entity_plural)),
        ("Menus", format!("{}Menus.cs", entity_plural)),
        ("Dto", format!("{entity_name}Dto.cs")),
        ("CreateDto", format!("{entity_name}CreateDto.cs")),
        ("UpdateDto", format!("{entity_name}UpdateDto.cs")),
        (
            "WithNavigationPropertiesDto",
            format!("{entity_name}WithNavigationPropertiesDto.cs"),
        ),
        ("ExcelDto", format!("{entity_name}ExcelDto.cs")),
        (
            "ExcelDownloadDto",
            format!("{entity_name}ExcelDownloadDto.cs"),
        ),
        ("GetInput", format!("Get{entity_name}Input.cs")),
        ("IAppService", format!("I{entity_name}AppService.cs")),
        ("LookupDto", format!("{entity_name}LookupDto.cs")),
        (
            "DownloadTokenResultDto",
            format!("{entity_name}DownloadTokenResultDto.cs"),
        ),
        ("LookupRequestDto", "LookupRequestDto.cs".into()),
    ]);
    process_embedded(
        "Application.Contracts",
        out_root,
        domain,
        "Application.Contracts",
        mapping,
        &ren,
        log,
    )?;

    let is_mvc = matches!(ui_target, UiTarget::Razor | UiTarget::Vue);
    if is_mvc {
        generate_mvc_razor(context, mapping)?;
        if matches!(ui_target, UiTarget::Vue) {
            generate_mvc_vue(context, mapping)?;
        }
    }
    if matches!(ui_target, UiTarget::Angular) {
        generate_angular_ui(context, mapping, log)?;
    }

    Ok(())
}

/* ----------------------------- SMALL HELPERS -------------------------- */

fn first_string_prop_of(out_root: &Path, domain: &str, nav_full: &str) -> String {
    let ns = ns_of(nav_full);
    let class_name = last_segment(nav_full);

    let rel = ns_relative_path(domain, &ns);
    let entity_path = out_root
        .join(format!("{}.Domain", domain))
        .join(rel)
        .join(format!("{class}.cs", class = class_name));

    let text = fs::read_to_string(&entity_path).unwrap_or_default();

    let rx = Regex::new(
        r#"(?m)^\s*(?:public|protected|internal)?(?:\s+\w+)*\s+string\??\s+([A-Za-z_]\w*)\s*\{\s*get\s*;\s*(?:set|init)\s*;\s*\}"#,
    )
    .unwrap();

    for cap in rx.captures_iter(&text) {
        let p = cap.get(1).map(|m| m.as_str()).unwrap_or("Id");
        if p.eq_ignore_ascii_case("id") || p.is_empty() {
            continue;
        }
        return p.into();
    }

    "Id".into()
}

fn prop_type_of(out_root: &Path, domain: &str, nav_full: &str, prop: &str) -> Option<String> {
    let ns = ns_of(nav_full);
    let class_name = last_segment(nav_full);

    let rel = ns_relative_path(domain, &ns);
    let entity_path = out_root
        .join(format!("{}.Domain", domain))
        .join(rel)
        .join(format!("{class}.cs", class = class_name));

    let text = fs::read_to_string(&entity_path).ok()?;
    let rx = Regex::new(&format!(
        r#"(?m)^\s*(?:public|protected|internal|private)?(?:\s+\w+)*\s+([A-Za-z_][\w\<>\?]*)\s+{}\s*\{{\s*get\s*;\s*(?:set|init)\s*;\s*\}}"#,
        regex::escape(prop)
    ))
    .ok()?;

    rx.captures(&text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn prop_is_string(out_root: &Path, domain: &str, nav_full: &str, prop: &str) -> bool {
    prop_type_of(out_root, domain, nav_full, prop)
        .map(|t| t.trim_end_matches('?').eq_ignore_ascii_case("string"))
        .unwrap_or(false)
}

use crate::application::history::load_history;
use crate::helpers::infer_domain_from_src;
use crate::models::{AbpTheme, BootstrapOverride, MobileUi};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ProjectInspection {
    pub project_root: PathBuf,
    pub domain: Option<String>,
    pub has_mvc: bool,
    pub has_angular: bool,
    pub has_history: bool,
    pub entity_count: usize,
    pub theme: Option<AbpTheme>,
    pub bootstrap_override: Option<BootstrapOverride>,
    pub mobile_ui: Option<MobileUi>,
}

pub(crate) fn inspect_project(project_root: &Path) -> Result<ProjectInspection> {
    if !project_root.join("src").is_dir() {
        bail!(
            "project root must contain a src directory: {}",
            project_root.display()
        );
    }

    let project_root = project_root.canonicalize()?;
    let src_root = project_root.join("src");
    let domain = infer_domain_from_src(&src_root);
    let has_mvc = domain
        .as_ref()
        .is_some_and(|domain| src_root.join(format!("{domain}.Web")).is_dir());
    let has_angular = contains_directory_named(&project_root, "angular");
    let history = load_history(&project_root)?;
    let detected_theme = detect_abp_theme(&project_root)?;

    Ok(ProjectInspection {
        project_root,
        domain,
        has_mvc,
        has_angular,
        has_history: history.is_some(),
        entity_count: history.as_ref().map_or(0, |record| record.entities.len()),
        theme: detected_theme.or_else(|| history.as_ref().map(|record| record.theme)),
        bootstrap_override: history.as_ref().map(|record| record.bootstrap_override),
        mobile_ui: history.as_ref().map(|record| record.mobile_ui),
    })
}

pub(crate) fn detect_abp_theme(project_root: &Path) -> Result<Option<AbpTheme>> {
    let src_root = project_root.join("src");
    if !src_root.is_dir() {
        return Ok(None);
    }

    let mut web_directories = Vec::new();
    for entry in
        fs::read_dir(&src_root).with_context(|| format!("failed to read {}", src_root.display()))?
    {
        let path = entry
            .with_context(|| format!("failed to inspect an entry in {}", src_root.display()))?
            .path();
        if path.is_dir()
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".Web"))
        {
            web_directories.push(path);
        }
    }
    web_directories.sort();

    for web_directory in &web_directories {
        if let Some(theme) = detect_theme_in_files(web_directory, |path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with("WebModule.cs"))
        })? {
            return Ok(Some(theme));
        }
    }
    for web_directory in &web_directories {
        if let Some(theme) = detect_theme_in_files(web_directory, |path| {
            path.extension()
                .is_some_and(|extension| extension == "csproj")
        })? {
            return Ok(Some(theme));
        }
    }

    if let Some(angular_directory) = find_directory_named(project_root, "angular")? {
        let app_directory = angular_directory.join("src").join("app");
        if app_directory.is_dir()
            && let Some(theme) = detect_theme_in_tree(&app_directory, |path| {
                path.extension().is_some_and(|extension| extension == "ts")
            })?
        {
            return Ok(Some(theme));
        }

        for manifest in ["package.json", "angular.json"] {
            let path = angular_directory.join(manifest);
            if !path.is_file() {
                continue;
            }
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            if let Some(theme) = detect_theme_in_text(&content) {
                return Ok(Some(theme));
            }
        }
    }

    Ok(None)
}

fn find_directory_named(root: &Path, expected: &str) -> Result<Option<PathBuf>> {
    let mut matches = fs::read_dir(root)
        .with_context(|| format!("failed to read {}", root.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(expected))
        })
        .collect::<Vec<_>>();
    matches.sort();
    Ok(matches.into_iter().next())
}

fn detect_theme_in_files(
    directory: &Path,
    matches: impl Fn(&Path) -> bool,
) -> Result<Option<AbpTheme>> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if !path.is_file() || !matches(&path) {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if let Some(theme) = detect_theme_in_text(&content) {
            return Ok(Some(theme));
        }
    }
    Ok(None)
}

fn detect_theme_in_tree(
    directory: &Path,
    matches: impl Fn(&Path) -> bool + Copy,
) -> Result<Option<AbpTheme>> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if let Some(theme) = detect_theme_in_tree(&path, matches)? {
                return Ok(Some(theme));
            }
            continue;
        }
        if !matches(&path) {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if let Some(theme) = detect_theme_in_text(&content) {
            return Ok(Some(theme));
        }
    }
    Ok(None)
}

fn detect_theme_in_text(content: &str) -> Option<AbpTheme> {
    let has_lepton_x = [
        "AbpAspNetCoreMvcUiLeptonXThemeModule",
        "AbpAspNetCoreMvcUiLeptonXLiteThemeModule",
        "LeptonXThemeBundles",
        "LeptonXLiteThemeBundles",
        "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonX",
        "@abp/ng.theme.lepton-x",
        "@volo/ngx-lepton-x",
        "provideThemeLeptonX",
        "ThemeLeptonXModule",
        "eThemeLeptonX",
    ]
    .iter()
    .any(|marker| content.contains(marker));
    let has_basic = [
        "AbpAspNetCoreMvcUiBasicThemeModule",
        "BasicThemeBundles",
        "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic",
        "@abp/ng.theme.basic",
        "provideThemeBasicConfig",
        "ThemeBasicModule",
        "eThemeBasic",
    ]
    .iter()
    .any(|marker| content.contains(marker));

    match (has_basic, has_lepton_x) {
        (true, false) => Some(AbpTheme::Basic),
        (false, true) => Some(AbpTheme::LeptonX),
        _ => None,
    }
}

fn contains_directory_named(root: &Path, expected: &str) -> bool {
    fs::read_dir(root).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry.path().is_dir()
                && entry
                    .file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(expected)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspects_detected_hosts_without_history() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Billing.Domain"))
            .expect("domain should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Billing.Web")).expect("web should exist");
        fs::create_dir(fixture.path().join("Angular")).expect("angular should exist");

        let inspection = inspect_project(fixture.path()).expect("inspection should work");
        assert_eq!(inspection.domain.as_deref(), Some("Acme.Billing"));
        assert!(inspection.has_mvc);
        assert!(inspection.has_angular);
        assert!(!inspection.has_history);
        assert_eq!(inspection.entity_count, 0);
    }

    #[test]
    fn detects_lepton_x_from_the_web_module_without_history() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let web = fixture.path().join("src/Acme.Billing.Web");
        fs::create_dir_all(&web).expect("web should exist");
        fs::write(
            web.join("BillingWebModule.cs"),
            "[DependsOn(typeof(AbpAspNetCoreMvcUiLeptonXLiteThemeModule))]",
        )
        .expect("module should exist");

        let inspection = inspect_project(fixture.path()).expect("inspection should work");
        assert_eq!(inspection.theme, Some(AbpTheme::LeptonX));
    }

    #[test]
    fn detects_basic_from_the_web_project_package_reference() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let web = fixture.path().join("src/Acme.Billing.Web");
        fs::create_dir_all(&web).expect("web should exist");
        fs::write(
            web.join("Acme.Billing.Web.csproj"),
            r#"<PackageReference Include="Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic" />"#,
        )
        .expect("project should exist");

        assert_eq!(
            detect_abp_theme(fixture.path()).expect("detection should work"),
            Some(AbpTheme::Basic)
        );
    }

    #[test]
    fn detects_lepton_x_from_an_angular_workspace_without_an_mvc_host() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Billing.Domain"))
            .expect("domain should exist");
        fs::create_dir_all(fixture.path().join("Angular/src/app"))
            .expect("Angular app should exist");
        fs::write(
            fixture.path().join("Angular/package.json"),
            r#"{"dependencies":{"@abp/ng.theme.lepton-x":"~10.6.0"}}"#,
        )
        .expect("package manifest should exist");

        let inspection = inspect_project(fixture.path()).expect("inspection should work");
        assert!(inspection.has_angular);
        assert_eq!(inspection.theme, Some(AbpTheme::LeptonX));
    }

    #[test]
    fn active_angular_basic_theme_wins_over_a_stale_lepton_x_dependency() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Billing.Domain"))
            .expect("domain should exist");
        let angular_app = fixture.path().join("angular/src/app");
        fs::create_dir_all(&angular_app).expect("Angular app should exist");
        fs::write(
            angular_app.join("app.config.ts"),
            "import { provideThemeBasicConfig } from '@abp/ng.theme.basic';\nprovideThemeBasicConfig();",
        )
        .expect("application configuration should exist");
        fs::write(
            fixture.path().join("angular/package.json"),
            r#"{"dependencies":{"@abp/ng.theme.basic":"latest","@abp/ng.theme.lepton-x":"latest"}}"#,
        )
        .expect("package manifest should exist");

        assert_eq!(
            detect_abp_theme(fixture.path()).expect("detection should work"),
            Some(AbpTheme::Basic)
        );
    }
}

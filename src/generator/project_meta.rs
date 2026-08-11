use super::command::{preflight_tools, run_command_with_output};
use super::ensure_bundle_line_any;
use crate::helpers::infer_domain_from_src;
use crate::models::{AbpTheme, BootstrapOverride, MobileUi};
use crate::ports::{CommandRunner, ExternalTool};
use crate::templates::{embedded_projects_walk, read_project_file, read_tpl_text};
use crate::utils::write_text;
use anyhow::{Context, Result, anyhow};
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};

fn update_web_module_theme(web_module: &Path, theme: AbpTheme) -> Result<bool> {
    if !web_module.exists() {
        return Ok(false);
    }

    let mut text = fs::read_to_string(web_module)
        .with_context(|| format!("failed to read {}", web_module.display()))?;

    let before = text.clone();

    match theme {
        AbpTheme::LeptonX => {
            let replacements = [
                (
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic.Bundling",
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite.Bundling",
                ),
                (
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic",
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite",
                ),
                ("BasicThemeBundles", "LeptonXLiteThemeBundles"),
                (
                    "AbpAspNetCoreMvcUiBasicThemeModule",
                    "AbpAspNetCoreMvcUiLeptonXLiteThemeModule",
                ),
            ];
            for (old, new_) in replacements {
                text = text.replace(old, new_);
            }
        }
        AbpTheme::Basic => {
            let replacements = [
                (
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite.Bundling",
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic.Bundling",
                ),
                (
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite",
                    "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic",
                ),
                ("LeptonXLiteThemeBundles", "BasicThemeBundles"),
                (
                    "AbpAspNetCoreMvcUiLeptonXLiteThemeModule",
                    "AbpAspNetCoreMvcUiBasicThemeModule",
                ),
            ];
            for (old, new_) in replacements {
                text = text.replace(old, new_);
            }
        }
    }

    if text != before {
        fs::write(web_module, text)?;
        return Ok(true);
    }

    Ok(false)
}

fn strip_theme_reference(csproj: &Path, needle: &str) -> Result<bool> {
    if !csproj.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(csproj)
        .with_context(|| format!("failed to read {}", csproj.display()))?;
    let filtered: Vec<_> = content.lines().filter(|l| !l.contains(needle)).collect();
    let new_content = filtered.join("\n");
    if new_content != content {
        fs::write(csproj, new_content)?;
        return Ok(true);
    }
    Ok(false)
}

fn find_angular_workspace(project_root: &Path) -> Result<Option<PathBuf>> {
    let mut matches = fs::read_dir(project_root)
        .with_context(|| format!("failed to read {}", project_root.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    matches.sort_by_key(|entry| entry.path());
    Ok(matches.into_iter().map(|entry| entry.path()).find(|path| {
        path.is_dir()
            && path.join("angular.json").is_file()
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("angular"))
    }))
}

fn ensure_import(text: &mut String, import: &str) -> Result<()> {
    if text.contains(import) {
        return Ok(());
    }
    let insert_at = text
        .find("import ")
        .ok_or_else(|| anyhow!("Angular startup file contains no import declarations"))?;
    text.insert_str(insert_at, &format!("{import}\n"));
    Ok(())
}

fn update_standalone_angular_theme(path: &Path, theme: AbpTheme) -> Result<bool> {
    let original =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if original.contains("@volosoft/abp.ng.theme.lepton-x") {
        return Err(anyhow!(
            "commercial Angular LeptonX is not interchangeable with LeptonX Lite; refusing to rewrite {}",
            path.display()
        ));
    }

    let mut text = original
        .lines()
        .filter(|line| {
            !line.contains("from '@abp/ng.theme.basic'")
                && !line.contains("from '@abp/ng.theme.lepton-x'")
                && !line.contains("from '@abp/ng.theme.lepton-x/layouts'")
                && !line.contains("from './footer/footer.config'")
                && !line.trim_start().starts_with("FOOTER_PROVIDER,")
        })
        .collect::<Vec<_>>()
        .join("\n");
    text.push('\n');

    match theme {
        AbpTheme::Basic => {
            text = text.replace("provideThemeLeptonX(),", "provideThemeBasicConfig(),");
            text = text
                .lines()
                .filter(|line| {
                    !line.contains("provideSideMenuLayout()")
                        && !line.contains("provideTopMenuLayout()")
                })
                .collect::<Vec<_>>()
                .join("\n");
            text.push('\n');
            ensure_import(
                &mut text,
                "import { provideThemeBasicConfig } from '@abp/ng.theme.basic';",
            )?;
            if !text.contains("provideThemeBasicConfig(),") {
                let marker = "    provideAbpThemeShared(),";
                let insert_at = text.find(marker).ok_or_else(|| {
                    anyhow!(
                        "could not find provideAbpThemeShared() in {}",
                        path.display()
                    )
                })?;
                text.insert_str(insert_at, "    provideThemeBasicConfig(),\n");
            }
        }
        AbpTheme::LeptonX => {
            text = text.replace(
                "provideThemeBasicConfig(),",
                "provideThemeLeptonX(),\n    provideSideMenuLayout(),",
            );
            ensure_import(
                &mut text,
                "import { provideThemeLeptonX } from '@abp/ng.theme.lepton-x';",
            )?;
            ensure_import(
                &mut text,
                "import { provideSideMenuLayout } from '@abp/ng.theme.lepton-x/layouts';",
            )?;
            if !text.contains("provideThemeLeptonX(),") {
                let marker = "    provideAbpThemeShared(),";
                let insert_at = text.find(marker).ok_or_else(|| {
                    anyhow!(
                        "could not find provideAbpThemeShared() in {}",
                        path.display()
                    )
                })?;
                text.insert_str(
                    insert_at,
                    "    provideThemeLeptonX(),\n    provideSideMenuLayout(),\n",
                );
            }
            let has_footer = path
                .parent()
                .is_some_and(|app| app.join("footer/footer.config.ts").exists());
            if has_footer {
                ensure_import(
                    &mut text,
                    "import { FOOTER_PROVIDER } from './footer/footer.config';",
                )?;
                if !text.contains("    FOOTER_PROVIDER,") {
                    let marker = "    APP_ROUTE_PROVIDER,";
                    let insert_at = text.find(marker).ok_or_else(|| {
                        anyhow!("could not find APP_ROUTE_PROVIDER in {}", path.display())
                    })? + marker.len();
                    text.insert_str(insert_at, "\n    FOOTER_PROVIDER,");
                }
            }
        }
    }

    if text != original {
        fs::write(path, text)?;
        return Ok(true);
    }
    Ok(false)
}

fn update_legacy_angular_theme(path: &Path, theme: AbpTheme) -> Result<bool> {
    let original =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut text = original.clone();
    match theme {
        AbpTheme::Basic => {
            text = text.replace("@abp/ng.theme.lepton-x", "@abp/ng.theme.basic");
            text = text.replace("ThemeLeptonXModule", "ThemeBasicModule");
            text = text.replace("provideThemeLeptonX", "provideThemeBasicConfig");
            text = text
                .lines()
                .filter(|line| !line.contains("SideMenuLayoutModule"))
                .collect::<Vec<_>>()
                .join("\n");
            text.push('\n');
        }
        AbpTheme::LeptonX => {
            text = text.replace("@abp/ng.theme.basic", "@abp/ng.theme.lepton-x");
            text = text.replace("ThemeBasicModule", "ThemeLeptonXModule");
            text = text.replace("provideThemeBasicConfig", "provideThemeLeptonX");
            ensure_import(
                &mut text,
                "import { SideMenuLayoutModule } from '@abp/ng.theme.lepton-x/layouts';",
            )?;
            if !text.contains("SideMenuLayoutModule,") {
                let marker = "    ThemeLeptonXModule,";
                let insert_at = text.find(marker).ok_or_else(|| {
                    anyhow!("could not find ThemeLeptonXModule in {}", path.display())
                })? + marker.len();
                text.insert_str(insert_at, "\n    SideMenuLayoutModule,");
            }
        }
    }
    if text != original {
        fs::write(path, text)?;
        return Ok(true);
    }
    Ok(false)
}

fn update_angular_footer(angular_root: &Path, theme: AbpTheme) -> Result<bool> {
    let footer_config = angular_root.join("src/app/footer/footer.config.ts");
    if !footer_config.exists() {
        return Ok(false);
    }
    let content = match theme {
        AbpTheme::Basic => "export const FOOTER_PROVIDER = [];\n".to_string(),
        AbpTheme::LeptonX => r#"import { provideAppInitializer, inject } from '@angular/core';
import { ReplaceableComponentsService } from '@abp/ng.core';
import { eThemeLeptonXComponents } from '@abp/ng.theme.lepton-x';
import { FooterComponent } from './footer.component';

function initFooter() {
  const replaceableComponents = inject(ReplaceableComponentsService);
  replaceableComponents.add({
    key: eThemeLeptonXComponents.Footer,
    component: FooterComponent,
  });
}

export const FOOTER_PROVIDER = [
  provideAppInitializer(() => {
    initFooter();
  }),
];
"#
        .to_string(),
    };
    let original = fs::read_to_string(&footer_config)
        .with_context(|| format!("failed to read {}", footer_config.display()))?;
    if original == content {
        return Ok(false);
    }
    fs::write(footer_config, content)?;
    Ok(true)
}

fn angular_theme_styles(theme: AbpTheme) -> Vec<Value> {
    let values: &[(&str, bool, &str)] = match theme {
        AbpTheme::Basic => &[
            (
                "node_modules/bootstrap/dist/css/bootstrap.rtl.min.css",
                false,
                "bootstrap-rtl.min",
            ),
            (
                "node_modules/bootstrap/dist/css/bootstrap.min.css",
                true,
                "bootstrap-ltr.min",
            ),
        ],
        AbpTheme::LeptonX => &[
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/bootstrap-dim.css",
                false,
                "bootstrap-dim",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/ng-bundle.css",
                false,
                "ng-bundle",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/side-menu/layout-bundle.css",
                false,
                "layout-bundle",
            ),
            (
                "node_modules/@abp/ng.theme.lepton-x/assets/css/abp-bundle.css",
                false,
                "abp-bundle",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/bootstrap-dim.rtl.css",
                false,
                "bootstrap-dim.rtl",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/font-bundle.css",
                false,
                "font-bundle",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/font-bundle.rtl.css",
                false,
                "font-bundle.rtl",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/ng-bundle.rtl.css",
                false,
                "ng-bundle.rtl",
            ),
            (
                "node_modules/@volo/ngx-lepton-x.lite/assets/css/side-menu/layout-bundle.rtl.css",
                false,
                "layout-bundle.rtl",
            ),
            (
                "node_modules/@abp/ng.theme.lepton-x/assets/css/abp-bundle.rtl.css",
                false,
                "abp-bundle.rtl",
            ),
            (
                "node_modules/bootstrap-icons/font/bootstrap-icons.css",
                true,
                "bootstrap-icons",
            ),
        ],
    };
    values
        .iter()
        .map(|(input, inject, bundle_name)| {
            json!({ "input": input, "inject": inject, "bundleName": bundle_name })
        })
        .collect()
}

fn is_angular_theme_style(value: &Value) -> bool {
    let input = value
        .as_object()
        .and_then(|value| value.get("input"))
        .and_then(Value::as_str)
        .or_else(|| value.as_str())
        .unwrap_or_default();
    input.contains("ngx-lepton-x")
        || input.contains("ng.theme.lepton-x")
        || input.contains("bootstrap-icons/font/bootstrap-icons.css")
        || input.contains("bootstrap/dist/css/bootstrap.rtl.min.css")
        || input.contains("bootstrap/dist/css/bootstrap.min.css")
}

fn update_angular_json(path: &Path, theme: AbpTheme) -> Result<bool> {
    let original =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut root: Value = serde_json::from_str(&original)
        .with_context(|| format!("failed to parse {} as JSON", path.display()))?;
    let projects = root
        .get_mut("projects")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("{} has no projects object", path.display()))?;

    let mut changed = false;
    for project in projects.values_mut() {
        for target_key in ["architect", "targets"] {
            let Some(styles) = project
                .get_mut(target_key)
                .and_then(|value| value.get_mut("build"))
                .and_then(|value| value.get_mut("options"))
                .and_then(|value| value.get_mut("styles"))
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            let before = styles.clone();
            styles.retain(|style| !is_angular_theme_style(style));
            let insert_at = styles
                .iter()
                .position(|style| style.as_str() == Some("src/styles.scss"))
                .unwrap_or(styles.len());
            styles.splice(insert_at..insert_at, angular_theme_styles(theme));
            changed |= *styles != before;
        }
    }

    if changed {
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))?;
    }
    Ok(changed)
}

fn patch_angular_theme_files(
    angular_root: &Path,
    theme: AbpTheme,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let app = angular_root.join("src/app");
    let standalone = app.join("app.config.ts");
    let legacy = app.join("app.module.ts");
    let startup_changed = if standalone.exists() {
        update_standalone_angular_theme(&standalone, theme)?
    } else if legacy.exists() {
        update_legacy_angular_theme(&legacy, theme)?
    } else {
        return Err(anyhow!(
            "Angular startup file not found; expected {} or {}",
            standalone.display(),
            legacy.display()
        ));
    };
    if startup_changed {
        log("Patched Angular startup theme providers");
    }

    if update_angular_footer(angular_root, theme)? {
        log("Aligned the optional Angular footer provider with the selected theme");
    }
    let angular_json = angular_root.join("angular.json");
    if !angular_json.exists() {
        return Err(anyhow!(
            "Angular workspace config not found: {}",
            angular_json.display()
        ));
    }
    if update_angular_json(&angular_json, theme)? {
        log("Patched Angular theme style bundles");
    }
    Ok(())
}

pub fn apply_theme_change(
    runner: &dyn CommandRunner,
    project_root: &Path,
    theme: AbpTheme,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let src_root = project_root.join("src");
    let domain = infer_domain_from_src(&src_root);
    let web = domain.as_ref().and_then(|domain| {
        let directory = src_root.join(format!("{domain}.Web"));
        directory.exists().then_some((domain.as_str(), directory))
    });
    let angular = find_angular_workspace(project_root)?;

    if web.is_none() && angular.is_none() {
        return Err(anyhow!(
            "theme change requires an MVC Web host or Angular workspace under {}",
            project_root.display()
        ));
    }

    let mut tools = Vec::new();
    if web.is_some() {
        tools.extend([ExternalTool::DotNet, ExternalTool::Npm, ExternalTool::Abp]);
    } else if angular.is_some() {
        tools.push(ExternalTool::Npm);
    }
    preflight_tools(runner, &tools, log)?;

    if let Some((domain, web_dir)) = web {
        let csproj = web_dir.join(format!("{domain}.Web.csproj"));
        match theme {
            AbpTheme::LeptonX => {
                strip_theme_reference(&csproj, "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic")?;
                run_command_with_output(
                    runner,
                    &web_dir,
                    "dotnet",
                    &[
                        "add",
                        "package",
                        "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite",
                    ],
                    log,
                )?;
            }
            AbpTheme::Basic => {
                strip_theme_reference(&csproj, "Volo.Abp.AspNetCore.Mvc.UI.Theme.LeptonXLite")?;
                run_command_with_output(
                    runner,
                    &web_dir,
                    "dotnet",
                    &["add", "package", "Volo.Abp.AspNetCore.Mvc.UI.Theme.Basic"],
                    log,
                )?;
                run_command_with_output(
                    runner,
                    &web_dir,
                    "abp",
                    &["add-package", "@abp/aspnetcore.mvc.ui.theme.basic"],
                    log,
                )?;
            }
        }

        let domain_short = domain.split('.').next_back().unwrap_or(domain);
        let module_candidates = [
            web_dir.join("WebModule.cs"),
            web_dir.join(format!("{domain_short}WebModule.cs")),
        ];
        let web_module = module_candidates
            .iter()
            .find(|path| path.exists())
            .cloned()
            .unwrap_or_else(|| module_candidates[0].clone());
        if update_web_module_theme(&web_module, theme)? {
            log(&format!(
                "Patched {} for {:?} theme",
                web_module.display(),
                theme
            ));
        } else {
            log("WebModule.cs already aligned with selected theme");
        }
        run_command_with_output(runner, &web_dir, "abp", &["install-libs"], log)?;
    }

    if let Some(angular_root) = angular {
        let app = angular_root.join("src/app");
        let startup = [app.join("app.config.ts"), app.join("app.module.ts")]
            .into_iter()
            .find(|path| path.exists())
            .ok_or_else(|| anyhow!("Angular startup file not found under {}", app.display()))?;
        let startup_text = fs::read_to_string(&startup)
            .with_context(|| format!("failed to read {}", startup.display()))?;
        if startup_text.contains("@volosoft/abp.ng.theme.lepton-x") {
            return Err(anyhow!(
                "commercial Angular LeptonX conversion is not supported; no files were changed"
            ));
        }

        let (target_package, old_package) = match theme {
            AbpTheme::Basic => ("@abp/ng.theme.basic", "@abp/ng.theme.lepton-x"),
            AbpTheme::LeptonX => ("@abp/ng.theme.lepton-x", "@abp/ng.theme.basic"),
        };
        run_command_with_output(
            runner,
            &angular_root,
            "npm",
            &["install", "--save", "--legacy-peer-deps", target_package],
            log,
        )?;
        run_command_with_output(
            runner,
            &angular_root,
            "npm",
            &["uninstall", "--legacy-peer-deps", old_package],
            log,
        )?;
        patch_angular_theme_files(&angular_root, theme, log)?;
        run_command_with_output(runner, &angular_root, "npm", &["run", "build"], log)?;
    }

    Ok(())
}

fn update_angular_override_bundle(path: &Path, enabled: bool) -> Result<bool> {
    const OVERRIDE_STYLE: &str = "src/banna-theme.scss";
    let original =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut root: Value = serde_json::from_str(&original)
        .with_context(|| format!("failed to parse {} as JSON", path.display()))?;
    let projects = root
        .get_mut("projects")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("{} has no projects object", path.display()))?;

    let mut changed = false;
    for project in projects.values_mut() {
        for target_key in ["architect", "targets"] {
            let Some(styles) = project
                .get_mut(target_key)
                .and_then(|value| value.get_mut("build"))
                .and_then(|value| value.get_mut("options"))
                .and_then(|value| value.get_mut("styles"))
                .and_then(Value::as_array_mut)
            else {
                continue;
            };
            let before = styles.clone();
            styles.retain(|style| {
                style.as_str() != Some(OVERRIDE_STYLE)
                    && style
                        .as_object()
                        .and_then(|style| style.get("input"))
                        .and_then(Value::as_str)
                        != Some(OVERRIDE_STYLE)
            });
            if enabled {
                let insert_at = styles
                    .iter()
                    .position(|style| style.as_str() == Some("src/styles.scss"))
                    .map_or(styles.len(), |index| index + 1);
                styles.insert(insert_at, Value::String(OVERRIDE_STYLE.into()));
            }
            changed |= *styles != before;
        }
    }

    if changed {
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))?;
    }
    Ok(changed)
}

fn ensure_bootstrap_override_files(
    project_root: &Path,
    domain: Option<&str>,
    override_css: BootstrapOverride,
    log: &mut dyn FnMut(&str),
) -> Result<bool> {
    let mut has_angular = false;
    if let Some(domain) = domain {
        let web_dir = project_root.join("src").join(format!("{}.Web", domain));
        if web_dir.is_dir() {
            let wwwroot = web_dir.join("wwwroot");
            fs::create_dir_all(&wwwroot)?;
            let target = wwwroot.join("global-styles.css");
            let old_target = wwwroot.join("bootstrap-overrides.css");
            if old_target.exists() {
                fs::remove_file(&old_target)?;
                log(&format!("removed legacy {}", old_target.display()));
            }

            let tpl_candidates = match override_css {
                BootstrapOverride::Modern => &["Themes/Modern/MVC/global-styles.css"][..],
                BootstrapOverride::Nord => &["Themes/Nord/MVC/global-styles.css"][..],
                BootstrapOverride::Solarized => &["Themes/Solarized/MVC/global-styles.css"][..],
                BootstrapOverride::None => &[
                    "Web/MVC/wwwroot/global-styles.css",
                    "Web/wwwroot/global-styles.css",
                ][..],
            };
            let content = tpl_candidates
                .iter()
                .find_map(|template| read_tpl_text(template).ok())
                .ok_or_else(|| anyhow!("no template found for bootstrap override styles"))?;
            write_text(&target, &content)?;
            log(&format!("wrote {}", target.display()));
        }
    }

    if let Some(angular_root) = find_angular_workspace(project_root)? {
        has_angular = true;
        let angular_styles = angular_root.join("src/banna-theme.scss");
        let enabled = override_css != BootstrapOverride::None;
        if enabled {
            let template = match override_css {
                BootstrapOverride::Modern => "Themes/Modern/Angular/styles.scss",
                BootstrapOverride::Nord => "Themes/Nord/Angular/styles.scss",
                BootstrapOverride::Solarized => "Themes/Solarized/Angular/styles.scss",
                BootstrapOverride::None => unreachable!(),
            };
            write_text(&angular_styles, &read_tpl_text(template)?)?;
            log(&format!("wrote {}", angular_styles.display()));
        } else if angular_styles.exists() {
            fs::remove_file(&angular_styles)?;
            log(&format!("removed {}", angular_styles.display()));
        }
        let angular_json = angular_root.join("angular.json");
        if update_angular_override_bundle(&angular_json, enabled)? {
            log(&format!(
                "updated Bootstrap override bundle in {}",
                angular_json.display()
            ));
        }
    }

    if matches!(
        override_css,
        BootstrapOverride::Modern | BootstrapOverride::Nord | BootstrapOverride::Solarized
    ) {
        // React Native theme override
        let rn_theme = project_root
            .join("react-native")
            .join("src")
            .join("theme")
            .join("index.ts");
        if rn_theme.exists() {
            let tpl = match override_css {
                BootstrapOverride::Modern => "Themes/Modern/Mobile/theme/index.ts",
                BootstrapOverride::Nord => "Themes/Nord/Mobile/theme/index.ts",
                BootstrapOverride::Solarized => "Themes/Solarized/Mobile/theme/index.ts",
                BootstrapOverride::None => unreachable!(),
            };
            if let Ok(c) = read_tpl_text(tpl) {
                write_text(&rn_theme, &c)?;
                log(&format!("wrote {}", rn_theme.display()));
            } else {
                log("React Native theme template missing");
            }
        } else {
            log("React Native app not found; skipping mobile theme override");
        }
    }

    Ok(has_angular)
}

fn ensure_override_bundle(web_module: &Path, log: &mut dyn FnMut(&str)) -> Result<()> {
    if !web_module.exists() {
        log(&format!(
            "skip: WebModule not found at {}, cannot bundle override",
            web_module.display()
        ));
        return Ok(());
    }

    let mut text = fs::read_to_string(web_module)
        .with_context(|| format!("failed to read {}", web_module.display()))?;
    let mut changed = false;

    // Remove legacy bootstrap-overrides.css if present
    if text.contains("/bootstrap-overrides.css") {
        text = text.replace(r#"bundle.AddFiles("/bootstrap-overrides.css");"#, "");
        text = text.replace(r#"bundle.AddFiles("/bootstrap-overrides.css")"#, "");
        changed = true;
    }

    let style_markers = [
        "BasicThemeBundles.Styles.Global",
        "StandardBundles.Styles.Global",
        "LeptonXLiteThemeBundles.Styles.Global",
    ];

    let global_added = ensure_bundle_line_any(
        &mut text,
        &style_markers,
        r#"bundle.AddFiles("/global-styles.css");"#,
    );
    changed |= global_added;

    let has_style_bundle = text.contains("BasicThemeBundles.Styles.Global")
        || text.contains("StandardBundles.Styles.Global")
        || text.contains("LeptonXLiteThemeBundles.Styles.Global");

    if !global_added && !has_style_bundle {
        let bundle_key = if text.contains("LeptonXLiteThemeBundles") {
            "LeptonXLiteThemeBundles"
        } else if text.contains("StandardBundles") {
            "StandardBundles"
        } else {
            "BasicThemeBundles"
        };
        let fallback = r#"
        Configure<AbpBundlingOptions>(options =>
        {
            options.StyleBundles.Configure(
                BUNDLE_PLACEHOLDER.Styles.Global,
                bundle =>
                {
                    bundle.AddFiles("/global-styles.css");
                });
        });
"#;
        let fallback = fallback.replace("BUNDLE_PLACEHOLDER", bundle_key);
        text.push_str(&fallback);
        changed = true;
    }

    if changed {
        fs::write(web_module, text)?;
        log(&format!(
            "patched {} for global styles bundling",
            web_module.display()
        ));
    } else {
        log("Global styles bundle entry already present");
    }

    Ok(())
}

pub(crate) struct ProjectMetaChangeRequest<'a> {
    pub project_root: &'a Path,
    pub theme_changed: bool,
    pub theme: AbpTheme,
    pub override_changed: bool,
    pub override_css: BootstrapOverride,
    pub mobile_changed: bool,
    pub mobile_ui: MobileUi,
}

pub fn apply_project_meta_change(
    runner: &dyn CommandRunner,
    request: &ProjectMetaChangeRequest<'_>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let project_root = request.project_root;
    let theme_changed = request.theme_changed;
    let theme = request.theme;
    let override_changed = request.override_changed;
    let override_css = request.override_css;
    let mobile_changed = request.mobile_changed;
    let mobile_ui = request.mobile_ui;
    let src_root = project_root.join("src");
    let domain = infer_domain_from_src(&src_root);
    if domain.is_none() {
        log("Could not infer domain from src, some project meta steps may be skipped");
    }
    let domain_short = domain
        .as_ref()
        .map(|d| d.split('.').next_back().unwrap_or(d).to_string());
    let web_module = domain.as_ref().map(|d| {
        let web_dir = src_root.join(format!("{}.Web", d));
        let domain_short = d.split('.').next_back().unwrap_or(d);
        let module_candidates = [
            web_dir.join("WebModule.cs"),
            web_dir.join(format!("{domain_short}WebModule.cs")),
        ];
        module_candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| module_candidates[0].clone())
    });

    if theme_changed {
        log(&format!("Applying theme change to {:?}", theme));
        apply_theme_change(runner, project_root, theme, log)?;
    } else {
        log("Theme unchanged, skipping theme operations");
    }

    if override_changed {
        let has_angular =
            ensure_bootstrap_override_files(project_root, domain.as_deref(), override_css, log)?;
        if let Some(web_module) = web_module.as_ref().filter(|path| path.is_file()) {
            ensure_override_bundle(web_module, log)?;
        }
        if has_angular {
            preflight_tools(runner, &[ExternalTool::Npm], log)?;
            let angular_root = find_angular_workspace(project_root)?.ok_or_else(|| {
                anyhow!("Angular workspace disappeared while applying Bootstrap override")
            })?;
            run_command_with_output(runner, &angular_root, "npm", &["run", "build"], log)?;
        }
    } else {
        log("Bootstrap override unchanged, skipping override operations");
    }

    if mobile_changed {
        match mobile_ui {
            MobileUi::ReactNative => {
                copy_react_native_template(
                    project_root,
                    domain.as_deref(),
                    domain_short.as_deref(),
                    log,
                )?;
                if let (Some(d), Some(d_short)) = (domain.as_ref(), domain_short.as_ref()) {
                    ensure_mobile_openiddict_app(project_root, d, d_short, log)?;
                    ensure_mobile_openiddict_data_seed(project_root, d, d_short, log)?;
                } else {
                    log("Could not infer domain; skipping mobile OpenIddict app");
                }
                log("Skipping other mobile appsettings and module changes (not required anymore)");
            }
            MobileUi::None => log("Mobile UI set to None, no mobile assets applied"),
        }
    } else {
        log("Mobile UI unchanged, skipping mobile scaffold copy");
    }

    Ok(())
}

fn copy_react_native_template(
    project_root: &Path,
    domain: Option<&str>,
    domain_short: Option<&str>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let prefix = "Vanilla/react-native";
    let files = embedded_projects_walk(prefix);

    if files.is_empty() {
        log("React Native project template not found in embedded assets");
        return Ok(());
    }

    let norm_prefix = format!("{prefix}/");
    let dest_root = project_root.join("react-native");

    if dest_root.exists() {
        log(&format!(
            "React Native directory already exists at {}; skipping template copy to avoid overwriting",
            dest_root.display()
        ));
        return Ok(());
    }

    for rel in files {
        if !rel.starts_with(&norm_prefix) {
            continue;
        }
        let inner_path = &rel[norm_prefix.len()..];
        if inner_path.is_empty() {
            continue;
        }
        let dest = dest_root.join(inner_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let bytes = read_project_file(&rel)?;
        fs::write(&dest, bytes)?;
        log(&format!("Wrote {}", dest.display()));
    }

    log(&format!(
        "React Native template copied to {}",
        dest_root.display()
    ));

    if let (Some(domain), Some(domain_short)) = (domain, domain_short) {
        let company_name = domain
            .split('.')
            .next()
            .filter(|seg| !seg.is_empty())
            .unwrap_or(domain_short);
        apply_react_native_placeholders(&dest_root, domain_short, company_name, log)?;
    }

    Ok(())
}

fn apply_react_native_placeholders(
    dest_root: &Path,
    domain_short: &str,
    company_name: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let replacements = [
        ("MyProjectName", domain_short),
        ("MyCompanyName", company_name),
    ];

    fn replace_all(text: &str, replacements: &[(&str, &str)]) -> String {
        let mut out = text.to_string();
        for (needle, value) in replacements {
            out = out.replace(needle, value);
        }
        out
    }

    fn visit_dir(
        path: &Path,
        replacements: &[(&str, &str)],
        content_updates: &mut usize,
        rename_updates: &mut usize,
    ) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                visit_dir(&entry_path, replacements, content_updates, rename_updates)?;
            } else if let Ok(text) = fs::read_to_string(&entry_path) {
                let replaced = replace_all(&text, replacements);
                if replaced != text {
                    fs::write(&entry_path, replaced)?;
                    *content_updates += 1;
                }
            }

            // Rename files or directories whose names still carry placeholders
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                let new_name = replace_all(name, replacements);
                if new_name != name
                    && let Some(parent) = entry_path.parent()
                {
                    let new_path = parent.join(&new_name);
                    fs::rename(&entry_path, &new_path)?;
                    *rename_updates += 1;
                }
            }
        }
        Ok(())
    }

    let mut content_updates = 0usize;
    let mut rename_updates = 0usize;
    visit_dir(
        dest_root,
        &replacements,
        &mut content_updates,
        &mut rename_updates,
    )?;
    if content_updates > 0 || rename_updates > 0 {
        log(&format!(
            "React Native placeholders updated in {} files; {} paths renamed",
            content_updates, rename_updates
        ));
    }
    Ok(())
}

fn ensure_mobile_openiddict_app(
    project_root: &Path,
    domain: &str,
    domain_short: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let migrator_appsettings = project_root
        .join("src")
        .join(format!("{}.DbMigrator", domain))
        .join("appsettings.json");

    if !migrator_appsettings.exists() {
        log(&format!(
            "DbMigrator appsettings not found at {}, skipping mobile OpenIddict application",
            migrator_appsettings.display()
        ));
        return Ok(());
    }

    let text = fs::read_to_string(&migrator_appsettings)?;
    let mut root: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            log(&format!(
                "Could not parse {} as JSON: {e}",
                migrator_appsettings.display()
            ));
            return Ok(());
        }
    };

    let root_obj = match root.as_object_mut() {
        Some(o) => o,
        None => {
            log("Unexpected appsettings.json format; expected JSON object");
            return Ok(());
        }
    };

    let openid = root_obj
        .entry("OpenIddict")
        .or_insert_with(|| Value::Object(Map::new()));
    let apps = openid
        .as_object_mut()
        .expect("OpenIddict should be object")
        .entry("Applications")
        .or_insert_with(|| Value::Object(Map::new()));
    let apps_map = apps
        .as_object_mut()
        .expect("OpenIddict.Applications should be object");

    let mobile_key = format!("{domain_short}_Mobile");
    if apps_map.contains_key(&mobile_key) {
        log("Mobile OpenIddict application already present in DbMigrator appsettings");
        return Ok(());
    }

    let mut mobile_entry = Map::new();
    mobile_entry.insert("ClientId".into(), Value::String(mobile_key.clone()));
    mobile_entry.insert(
        "RootUrl".into(),
        Value::String("exp://0.0.0.0:19000".into()),
    );
    apps_map.insert(mobile_key.clone(), Value::Object(mobile_entry));

    fs::write(&migrator_appsettings, serde_json::to_string_pretty(&root)?)?;
    log(&format!(
        "Added OpenIddict mobile application '{}' to {}",
        mobile_key,
        migrator_appsettings.display()
    ));

    Ok(())
}

fn ensure_mobile_openiddict_data_seed(
    project_root: &Path,
    domain: &str,
    domain_short: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let data_seed = project_root
        .join("src")
        .join(format!("{domain}.Domain",))
        .join("OpenIddict")
        .join("OpenIddictDataSeedContributor.cs");

    if !data_seed.exists() {
        log(&format!(
            "OpenIddictDataSeedContributor not found at {}, skipping mobile client registration",
            data_seed.display()
        ));
        return Ok(());
    }

    let mut content = fs::read_to_string(&data_seed)?;
    if content.contains(&format!("{domain_short}_Mobile")) {
        log("Mobile OpenIddict application already present in data seed");
        return Ok(());
    }

    let marker = "// Swagger Client";
    let insert_pos = if let Some(pos) = content.find(marker) {
        pos
    } else {
        log("Could not find Swagger client marker to insert mobile client; skipping");
        return Ok(());
    };

    let mobile_block = format!(
        r#"
        // Mobile Client
        var mobileClientId = configurationSection["{domain_short}_Mobile:ClientId"];
        if (!mobileClientId.IsNullOrWhiteSpace())
        {{
            var mobileRootUrl = configurationSection["{domain_short}_Mobile:RootUrl"]?.TrimEnd('/');

            await CreateApplicationAsync(
                name: mobileClientId!,
                type: OpenIddictConstants.ClientTypes.Public,
                consentType: OpenIddictConstants.ConsentTypes.Explicit,
                displayName: "Mobile Application",
                secret: null,
                grantTypes: new List<string> {{
                    OpenIddictConstants.GrantTypes.Password, OpenIddictConstants.GrantTypes.RefreshToken
                }},
                scopes: commonScopes,
                clientUri: mobileRootUrl
            );
        }}

"#
    );

    content.insert_str(insert_pos, &mobile_block);
    fs::write(&data_seed, content)?;
    log(&format!(
        "Added mobile OpenIddict application seeding to {}",
        data_seed.display()
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{CommandOutput, CommandSpec};
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingRunner(Mutex<Vec<CommandSpec>>);

    impl CommandRunner for RecordingRunner {
        fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
            self.0.lock().expect("recording lock").push(spec.clone());
            Ok(CommandOutput {
                status_code: Some(0),
                success: true,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }
    }

    fn angular_fixture() -> tempfile::TempDir {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Demo.Domain"))
            .expect("domain should exist");
        let app = fixture.path().join("angular/src/app");
        fs::create_dir_all(app.join("footer")).expect("Angular app should exist");
        fs::write(
            app.join("app.config.ts"),
            r#"import { provideAbpThemeShared } from '@abp/ng.theme.shared';
import { provideThemeLeptonX } from '@abp/ng.theme.lepton-x';
import { provideSideMenuLayout } from '@abp/ng.theme.lepton-x/layouts';
import { ApplicationConfig } from '@angular/core';
import { APP_ROUTE_PROVIDER } from './route.provider';
import { FOOTER_PROVIDER } from './footer/footer.config';

export const appConfig: ApplicationConfig = {
  providers: [
    APP_ROUTE_PROVIDER,
    FOOTER_PROVIDER,
    provideThemeLeptonX(),
    provideSideMenuLayout(),
    provideAbpThemeShared(),
  ],
};
"#,
        )
        .expect("startup config should exist");
        fs::write(
            app.join("footer/footer.component.ts"),
            "export class FooterComponent {}\n",
        )
        .expect("footer component should exist");
        fs::write(
            app.join("footer/footer.config.ts"),
            "import { eThemeLeptonXComponents } from '@abp/ng.theme.lepton-x';\nexport const FOOTER_PROVIDER = [eThemeLeptonXComponents.Footer];\n",
        )
        .expect("footer config should exist");
        fs::write(
            fixture.path().join("angular/angular.json"),
            r#"{
  "projects": {
    "Demo": {
      "architect": {
        "build": {
          "options": {
            "styles": [
              "node_modules/@fortawesome/fontawesome-free/css/all.min.css",
              {"input":"node_modules/@volo/ngx-lepton-x.lite/assets/css/ng-bundle.css","inject":false,"bundleName":"ng-bundle"},
              {"input":"node_modules/bootstrap-icons/font/bootstrap-icons.css","inject":true,"bundleName":"bootstrap-icons"},
              "src/styles.scss"
            ]
          }
        }
      }
    }
  }
}"#,
        )
        .expect("Angular workspace config should exist");
        fixture
    }

    #[test]
    fn generated_react_native_project_preserves_legal_payload() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.BookStore.Domain"))
            .expect("domain should exist");

        apply_project_meta_change(
            &RecordingRunner::default(),
            &ProjectMetaChangeRequest {
                project_root: fixture.path(),
                theme_changed: false,
                theme: AbpTheme::Basic,
                override_changed: false,
                override_css: BootstrapOverride::None,
                mobile_changed: true,
                mobile_ui: MobileUi::ReactNative,
            },
            &mut |_| {},
        )
        .expect("React Native scaffold should be generated");

        let generated = fixture.path().join("react-native");
        let notice =
            fs::read_to_string(generated.join("NOTICE.md")).expect("generated notice should exist");
        assert!(notice.contains("abpframework/abp/tree/rel-8.3"));
        assert!(notice.contains("LGPL-3.0-only"));
        assert!(notice.contains("authors and contributors"));
        assert!(notice.contains("modified derivative"));

        for file in ["LICENSE.LGPL-3.0-only.txt", "LICENSE.GPL-3.0-only.txt"] {
            let actual = fs::read(generated.join(file)).expect("generated license should exist");
            let expected = read_project_file(&format!("Vanilla/react-native/{file}"))
                .expect("embedded license should exist");
            assert_eq!(actual, expected, "generated {file} must remain complete");
        }

        let lgpl = fs::read_to_string(generated.join("LICENSE.LGPL-3.0-only.txt"))
            .expect("generated LGPL text should be readable");
        assert!(lgpl.contains("GNU LESSER GENERAL PUBLIC LICENSE"));
        let gpl = fs::read_to_string(generated.join("LICENSE.GPL-3.0-only.txt"))
            .expect("generated GPL text should be readable");
        assert!(gpl.contains("GNU GENERAL PUBLIC LICENSE"));
    }

    #[test]
    fn switches_standalone_angular_theme_in_both_directions_idempotently() {
        let fixture = angular_fixture();
        let angular = fixture.path().join("angular");

        patch_angular_theme_files(&angular, AbpTheme::Basic, &mut |_| {})
            .expect("Basic theme patch should succeed");
        patch_angular_theme_files(&angular, AbpTheme::Basic, &mut |_| {})
            .expect("repeated Basic patch should succeed");
        let basic_config = fs::read_to_string(angular.join("src/app/app.config.ts"))
            .expect("Basic config should exist");
        let basic_workspace =
            fs::read_to_string(angular.join("angular.json")).expect("workspace should exist");
        let basic_footer = fs::read_to_string(angular.join("src/app/footer/footer.config.ts"))
            .expect("footer config should exist");
        assert!(basic_config.contains("provideThemeBasicConfig()"));
        assert!(!basic_config.contains("provideThemeLeptonX"));
        assert!(basic_workspace.contains("bootstrap-ltr.min"));
        assert!(!basic_workspace.contains("ngx-lepton-x"));
        assert!(!basic_workspace.contains("bootstrap-icons"));
        assert_eq!(basic_footer, "export const FOOTER_PROVIDER = [];\n");

        patch_angular_theme_files(&angular, AbpTheme::LeptonX, &mut |_| {})
            .expect("LeptonX theme patch should succeed");
        let first_lepton = fs::read_to_string(angular.join("src/app/app.config.ts"))
            .expect("LeptonX config should exist");
        patch_angular_theme_files(&angular, AbpTheme::LeptonX, &mut |_| {})
            .expect("repeated LeptonX patch should succeed");
        let second_lepton = fs::read_to_string(angular.join("src/app/app.config.ts"))
            .expect("LeptonX config should exist");
        let lepton_workspace =
            fs::read_to_string(angular.join("angular.json")).expect("workspace should exist");
        assert_eq!(first_lepton, second_lepton);
        assert!(second_lepton.contains("provideThemeLeptonX()"));
        assert!(second_lepton.contains("provideSideMenuLayout()"));
        assert!(second_lepton.contains("FOOTER_PROVIDER"));
        assert!(lepton_workspace.contains("ngx-lepton-x.lite"));
        assert!(lepton_workspace.contains("bootstrap-icons"));
        assert!(!lepton_workspace.contains("bootstrap-ltr.min"));
    }

    #[test]
    fn applies_and_removes_angular_bootstrap_override_without_replacing_app_styles() {
        let fixture = angular_fixture();
        let angular = fixture.path().join("angular");
        let app_styles = angular.join("src/styles.scss");
        fs::write(&app_styles, "/* application-owned styles */\n").expect("app styles");

        let has_angular = ensure_bootstrap_override_files(
            fixture.path(),
            None,
            BootstrapOverride::Nord,
            &mut |_| {},
        )
        .expect("Nord override should apply");
        assert!(has_angular);
        assert_eq!(
            fs::read_to_string(&app_styles).expect("app styles should remain"),
            "/* application-owned styles */\n"
        );
        assert!(angular.join("src/banna-theme.scss").is_file());
        let configured = fs::read_to_string(angular.join("angular.json")).expect("workspace");
        assert_eq!(configured.matches("src/banna-theme.scss").count(), 1);

        ensure_bootstrap_override_files(fixture.path(), None, BootstrapOverride::None, &mut |_| {})
            .expect("override should be removable");
        assert!(!angular.join("src/banna-theme.scss").exists());
        let restored = fs::read_to_string(angular.join("angular.json")).expect("workspace");
        assert!(!restored.contains("src/banna-theme.scss"));
        assert!(restored.contains("src/styles.scss"));
    }

    #[test]
    fn angular_bootstrap_override_requires_a_successful_build() {
        let fixture = angular_fixture();
        let runner = RecordingRunner::default();
        apply_project_meta_change(
            &runner,
            &ProjectMetaChangeRequest {
                project_root: fixture.path(),
                theme_changed: false,
                theme: AbpTheme::LeptonX,
                override_changed: true,
                override_css: BootstrapOverride::Modern,
                mobile_changed: false,
                mobile_ui: MobileUi::None,
            },
            &mut |_| {},
        )
        .expect("Angular override should build");

        let commands = runner.0.lock().expect("recording lock");
        assert!(
            commands
                .iter()
                .any(|command| command.program == "npm" && command.args == ["run", "build"])
        );
    }

    #[test]
    fn angular_only_theme_change_uses_unversioned_npm_packages_and_builds() {
        let fixture = angular_fixture();
        let runner = RecordingRunner::default();

        apply_theme_change(&runner, fixture.path(), AbpTheme::Basic, &mut |_| {})
            .expect("Angular theme change should succeed");

        let commands = runner.0.lock().expect("recording lock");
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands[0].args,
            [
                "install",
                "--save",
                "--legacy-peer-deps",
                "@abp/ng.theme.basic"
            ]
        );
        assert_eq!(
            commands[1].args,
            ["uninstall", "--legacy-peer-deps", "@abp/ng.theme.lepton-x"]
        );
        assert_eq!(commands[2].args, ["run", "build"]);
        assert!(commands.iter().all(|command| command.program == "npm"));
    }
}

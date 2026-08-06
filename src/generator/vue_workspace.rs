use super::command::run_command_with_output;
use super::{ensure_bundle_line_any, insert_into_js_block};
use crate::ports::CommandRunner;
use crate::templates::read_tpl_text;
use crate::utils::write_text;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub(super) fn vue_packages_missing(project_root: &Path, domain: &str) -> bool {
    let libs_dir = project_root
        .join("src")
        .join(format!("{}.Web", domain))
        .join("wwwroot")
        .join("libs");
    !libs_dir.join("vue").is_dir() || !libs_dir.join("vue-select").is_dir()
}

fn ensure_vue_resource_mapping(
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let web_dir = project_root.join("src").join(format!("{}.Web", domain));
    let candidates = [
        web_dir.join("abp.resourcemapping.js"),
        web_dir.join("Pages").join("abp.resourcemapping.js"),
    ];
    let mapping_path = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());

    if !mapping_path.exists() {
        log(&format!(
            "skip: abp.resourcemapping.js not found at {}",
            mapping_path.display()
        ));
        return Ok(());
    }

    let content = fs::read_to_string(&mapping_path)
        .with_context(|| format!("failed to read {}", mapping_path.display()))?;

    let mut updated = false;
    let (content, changed_aliases) = insert_into_js_block(
        &content,
        "aliases",
        &[
            "\"@node_modules\": \"./node_modules\",",
            "\"@libs\": \"./wwwroot/libs\",",
        ],
    )?;
    updated |= changed_aliases;

    let (content, changed_mappings) = insert_into_js_block(
        &content,
        "mappings",
        &[
            "\"@node_modules/vue/dist/**/*\": \"@libs/vue/\",",
            "\"@node_modules/vue-select/dist/**/*\": \"@libs/vue-select/\",",
        ],
    )?;
    updated |= changed_mappings;

    if updated {
        write_text(&mapping_path, &content)?;
        log(&format!(
            "updated vue resource mappings in {}",
            mapping_path.display()
        ));
    } else {
        log("resource mappings already contain vue entries");
    }

    Ok(())
}

fn ensure_vue_packages(
    runner: &dyn CommandRunner,
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let web_dir = project_root.join("src").join(format!("{}.Web", domain));
    let libs_dir = web_dir.join("wwwroot").join("libs");
    let vue_dir = libs_dir.join("vue");
    let vue_select_dir = libs_dir.join("vue-select");

    if !web_dir.exists() {
        log(&format!(
            "skip: web project directory not found at {}",
            web_dir.display()
        ));
        return Ok(());
    }

    if vue_dir.exists() && vue_select_dir.exists() {
        log("wwwroot/libs/vue and vue-select already exist, skipping npm/abp commands");
        return Ok(());
    }

    run_command_with_output(
        runner,
        &web_dir,
        "npm",
        &["install", "vue@latest", "vue-select@latest"],
        log,
    )?;
    run_command_with_output(runner, &web_dir, "abp", &["install-libs"], log)?;
    Ok(())
}

fn ensure_vue_helper_file(
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
    let helper_path = wwwroot.join("app.vuehelper.js");
    let tpl_path = "Web/MVC/wwwroot/app.vuehelper.js";
    let content = read_tpl_text(tpl_path)
        .with_context(|| format!("failed to read embedded template {tpl_path}"))?;
    if helper_path.exists() && fs::read_to_string(&helper_path)? == content {
        log("app.vuehelper.js is up to date");
        return Ok(());
    }
    let action = if helper_path.exists() {
        "updated"
    } else {
        "added"
    };
    write_text(&helper_path, &content)?;
    log(&format!("{action} {}", helper_path.display()));
    Ok(())
}

fn ensure_vue_bundle_registration(
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let web_dir = project_root.join("src").join(format!("{}.Web", domain));

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

    if web_module.exists() {
        let mut text = fs::read_to_string(&web_module)?;
        let mut changed = false;

        let style_markers = [
            "BasicThemeBundles.Styles.Global",
            "StandardBundles.Styles.Global",
            "LeptonXLiteThemeBundles.Styles.Global",
            "LeptonXThemeBundles.Styles.Global",
        ];
        let script_markers = [
            "BasicThemeBundles.Scripts.Global",
            "StandardBundles.Scripts.Global",
            "LeptonXLiteThemeBundles.Scripts.Global",
            "LeptonXThemeBundles.Scripts.Global",
        ];

        if !style_markers.iter().any(|marker| text.contains(marker))
            || !script_markers.iter().any(|marker| text.contains(marker))
        {
            bail!(
                "could not safely register Vue assets in {}; no supported ABP style/script bundle was found",
                web_module.display()
            );
        }

        let vue_select_style_added = ensure_bundle_line_any(
            &mut text,
            &style_markers,
            r#"bundle.AddFiles("/libs/vue-select/vue-select.css");"#,
        );
        let style_added = ensure_bundle_line_any(
            &mut text,
            &style_markers,
            r#"bundle.AddFiles("/global-styles.css");"#,
        );
        let helper_added = ensure_bundle_line_any(
            &mut text,
            &script_markers,
            r#"bundle.AddFiles("/app.vuehelper.js");"#,
        );
        let vue_select_added = ensure_bundle_line_any(
            &mut text,
            &script_markers,
            r#"bundle.AddFiles("/libs/vue-select/vue-select.umd.js");"#,
        );
        let vue_added = ensure_bundle_line_any(
            &mut text,
            &script_markers,
            r#"bundle.AddFiles("/libs/vue/vue.global.prod.js");"#,
        );
        let script_added = ensure_bundle_line_any(
            &mut text,
            &script_markers,
            r#"bundle.AddFiles("/global-scripts.js");"#,
        );

        changed |= style_added
            || script_added
            || helper_added
            || vue_added
            || vue_select_added
            || vue_select_style_added;

        if changed {
            fs::write(&web_module, text)?;
            log(&format!(
                "patched {} (bundle registration)",
                web_module.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn prepare_vue_workspace(
    runner: &dyn CommandRunner,
    project_root: &Path,
    domain: &str,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    ensure_vue_resource_mapping(project_root, domain, log)?;
    ensure_vue_packages(runner, project_root, domain, log)?;
    ensure_vue_helper_file(project_root, domain, log)?;
    ensure_vue_bundle_registration(project_root, domain, log)?;
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

    fn web_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let fixture = tempfile::tempdir().expect("fixture");
        let web_dir = fixture.path().join("src/Acme.BookStore.Web");
        fs::create_dir_all(web_dir.join("wwwroot")).expect("web fixture");
        (fixture, web_dir)
    }

    #[test]
    fn installs_latest_vue_packages_without_version_locks() {
        let (fixture, _) = web_fixture();
        let runner = RecordingRunner::default();

        ensure_vue_packages(&runner, fixture.path(), "Acme.BookStore", &mut |_| {})
            .expect("Vue package setup");

        let commands = runner.0.lock().expect("recording lock");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0].args,
            ["install", "vue@latest", "vue-select@latest"]
        );
        assert_eq!(commands[1].args, ["install-libs"]);
    }

    #[test]
    fn refreshes_the_owned_vue_helper_when_the_template_changes() {
        let (fixture, web_dir) = web_fixture();
        let helper = web_dir.join("wwwroot/app.vuehelper.js");
        fs::write(&helper, "stale helper").expect("stale helper");

        ensure_vue_helper_file(fixture.path(), "Acme.BookStore", &mut |_| {})
            .expect("helper refresh");

        assert_eq!(
            fs::read_to_string(helper).expect("updated helper"),
            read_tpl_text("Web/MVC/wwwroot/app.vuehelper.js").expect("embedded helper")
        );
    }

    #[test]
    fn bundle_registration_is_idempotent_and_rejects_unknown_layouts() {
        let (fixture, web_dir) = web_fixture();
        let module = web_dir.join("BookStoreWebModule.cs");
        fs::write(
            &module,
            r#"public class BookStoreWebModule
{
    private void ConfigureBundles()
    {
        options.StyleBundles.Configure(
            BasicThemeBundles.Styles.Global,
            bundle =>
            {
            });
        options.ScriptBundles.Configure(
            BasicThemeBundles.Scripts.Global,
            bundle =>
            {
            });
    }
}
"#,
        )
        .expect("known module");

        ensure_vue_bundle_registration(fixture.path(), "Acme.BookStore", &mut |_| {})
            .expect("first registration");
        ensure_vue_bundle_registration(fixture.path(), "Acme.BookStore", &mut |_| {})
            .expect("second registration");
        let registered = fs::read_to_string(&module).expect("registered module");
        for asset in [
            "/libs/vue/vue.global.prod.js",
            "/libs/vue-select/vue-select.umd.js",
            "/libs/vue-select/vue-select.css",
            "/app.vuehelper.js",
        ] {
            assert_eq!(
                registered.matches(asset).count(),
                1,
                "duplicate asset: {asset}"
            );
        }

        fs::write(&module, "public class BookStoreWebModule {}\n").expect("unknown module");
        let error = ensure_vue_bundle_registration(fixture.path(), "Acme.BookStore", &mut |_| {})
            .expect_err("unknown bundle layout should fail safely");
        assert!(
            error
                .to_string()
                .contains("could not safely register Vue assets")
        );
        assert_eq!(
            fs::read_to_string(module).expect("unchanged module"),
            "public class BookStoreWebModule {}\n"
        );
    }
}

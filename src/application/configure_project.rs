use crate::adapters::{DryRunCommandRunner, ProcessCommandRunner, StagedProject};
use crate::application::history::{load_history, save_history};
use crate::application::inspect::detect_abp_theme;
use crate::generator::{self, ProjectMetaChangeRequest};
use crate::models::{AbpTheme, BootstrapOverride, MobileUi, ProjectRecord};
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::PathBuf;

pub(crate) struct ConfigureProjectRequest {
    pub project_root: PathBuf,
    pub theme: Option<AbpTheme>,
    pub bootstrap_override: Option<BootstrapOverride>,
    pub mobile_ui: Option<MobileUi>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigureProjectOutcome {
    pub project_root: PathBuf,
    pub theme: AbpTheme,
    pub bootstrap_override: BootstrapOverride,
    pub mobile_ui: MobileUi,
    pub dry_run: bool,
    pub changed: bool,
    pub events: Vec<String>,
}

pub(crate) fn configure_project(
    request: ConfigureProjectRequest,
    logger: Option<&mut dyn FnMut(&str)>,
) -> Result<ConfigureProjectOutcome> {
    if !request.project_root.join("src").is_dir() {
        bail!(
            "project root must contain a src directory: {}",
            request.project_root.display()
        );
    }
    let loaded_history = load_history(&request.project_root)?;
    let history_was_missing = loaded_history.is_none();
    let mut history = loaded_history.unwrap_or_else(|| ProjectRecord {
        project_name: request.project_root.file_name().map_or_else(
            || "Project".into(),
            |name| name.to_string_lossy().into_owned(),
        ),
        project_dir: request.project_root.to_string_lossy().into_owned(),
        theme: AbpTheme::Basic,
        bootstrap_override: BootstrapOverride::None,
        mobile_ui: MobileUi::None,
        entities: Vec::new(),
    });
    let stored_theme = history.theme;
    if request.theme.is_none()
        && let Some(detected_theme) = detect_abp_theme(&request.project_root)?
    {
        history.theme = detected_theme;
    }
    let theme_metadata_changed = stored_theme != history.theme;

    let theme = request.theme.unwrap_or(history.theme);
    let bootstrap_override = request
        .bootstrap_override
        .unwrap_or(history.bootstrap_override);
    let mobile_ui = request.mobile_ui.unwrap_or(history.mobile_ui);
    // An explicit selection is also a recovery command: reapply the idempotent
    // transformation even if a previous interrupted attempt already changed
    // the startup marker before dependency installation or build validation.
    let theme_changed = request.theme.is_some() || history.theme != theme;
    let override_changed = history.bootstrap_override != bootstrap_override;
    let mobile_changed = history.mobile_ui != mobile_ui;
    let changed = theme_changed || override_changed || mobile_changed;
    let mut events = Vec::new();

    if changed {
        let staged = request
            .dry_run
            .then(|| StagedProject::copy_from(&request.project_root))
            .transpose()?;
        let execution_root = staged
            .as_ref()
            .map_or(request.project_root.as_path(), StagedProject::root);
        let original_root_text = request.project_root.display().to_string();
        let execution_root_text = execution_root.display().to_string();
        let mut logger = logger;
        let mut report = |message: &str| {
            let message = if request.dry_run {
                message.replace(&execution_root_text, &original_root_text)
            } else {
                message.to_owned()
            };
            events.push(message.clone());
            if let Some(logger) = logger.as_mut() {
                logger(&message);
            }
        };
        if request.dry_run {
            report("dry-run: applying project configuration in an isolated workspace");
        }
        let process_runner = ProcessCommandRunner;
        let dry_run_runner = DryRunCommandRunner;
        let runner = if request.dry_run {
            &dry_run_runner as &dyn crate::ports::CommandRunner
        } else {
            &process_runner as &dyn crate::ports::CommandRunner
        };
        let generator_request = ProjectMetaChangeRequest {
            project_root: execution_root,
            theme_changed,
            theme,
            override_changed,
            override_css: bootstrap_override,
            mobile_changed,
            mobile_ui,
        };
        generator::apply_project_meta_change(runner, &generator_request, &mut report)?;
    }

    if !request.dry_run && (changed || history_was_missing || theme_metadata_changed) {
        history.theme = theme;
        history.bootstrap_override = bootstrap_override;
        history.mobile_ui = mobile_ui;
        save_history(&request.project_root, &history)?;
    }

    Ok(ConfigureProjectOutcome {
        project_root: request.project_root,
        theme,
        bootstrap_override,
        mobile_ui,
        dry_run: request.dry_run,
        changed,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_matching_project_configuration_unchanged() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        std::fs::create_dir(fixture.path().join("src")).expect("src should exist");
        save_history(
            fixture.path(),
            &ProjectRecord {
                project_name: "Demo".into(),
                project_dir: fixture.path().display().to_string(),
                theme: AbpTheme::Basic,
                bootstrap_override: BootstrapOverride::None,
                mobile_ui: MobileUi::None,
                entities: Vec::new(),
            },
        )
        .expect("history should save");

        let outcome = configure_project(
            ConfigureProjectRequest {
                project_root: fixture.path().to_path_buf(),
                theme: None,
                bootstrap_override: None,
                mobile_ui: None,
                dry_run: false,
            },
            None,
        )
        .expect("matching configuration should succeed");

        assert!(!outcome.changed);
        assert!(outcome.events.is_empty());
    }

    #[test]
    fn applies_and_persists_a_bootstrap_override() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        std::fs::create_dir_all(fixture.path().join("src/Acme.Demo.Domain"))
            .expect("domain project should exist");
        std::fs::create_dir_all(fixture.path().join("src/Acme.Demo.Web"))
            .expect("web project should exist");
        save_history(
            fixture.path(),
            &ProjectRecord {
                project_name: "Demo".into(),
                project_dir: fixture.path().display().to_string(),
                theme: AbpTheme::Basic,
                bootstrap_override: BootstrapOverride::None,
                mobile_ui: MobileUi::None,
                entities: Vec::new(),
            },
        )
        .expect("history should save");

        let outcome = configure_project(
            ConfigureProjectRequest {
                project_root: fixture.path().to_path_buf(),
                theme: None,
                bootstrap_override: Some(BootstrapOverride::Modern),
                mobile_ui: None,
                dry_run: false,
            },
            None,
        )
        .expect("override should apply");

        assert!(outcome.changed);
        assert_eq!(outcome.bootstrap_override, BootstrapOverride::Modern);
        let history = load_history(fixture.path())
            .expect("history should load")
            .expect("history should exist");
        assert_eq!(history.bootstrap_override, BootstrapOverride::Modern);
        assert!(
            fixture
                .path()
                .join("src/Acme.Demo.Web/wwwroot/global-styles.css")
                .exists()
        );
    }

    #[test]
    fn reconciles_an_old_basic_default_with_the_detected_lepton_x_theme() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let web = fixture.path().join("src/Acme.Demo.Web");
        std::fs::create_dir_all(&web).expect("web project should exist");
        std::fs::write(
            web.join("DemoWebModule.cs"),
            "[DependsOn(typeof(AbpAspNetCoreMvcUiLeptonXLiteThemeModule))]",
        )
        .expect("web module should exist");
        save_history(
            fixture.path(),
            &ProjectRecord {
                project_name: "Demo".into(),
                project_dir: fixture.path().display().to_string(),
                theme: AbpTheme::Basic,
                bootstrap_override: BootstrapOverride::None,
                mobile_ui: MobileUi::None,
                entities: Vec::new(),
            },
        )
        .expect("history should save");

        let outcome = configure_project(
            ConfigureProjectRequest {
                project_root: fixture.path().to_path_buf(),
                theme: None,
                bootstrap_override: None,
                mobile_ui: None,
                dry_run: false,
            },
            None,
        )
        .expect("configuration should reconcile");

        assert_eq!(outcome.theme, AbpTheme::LeptonX);
        assert!(!outcome.changed);
        assert_eq!(
            load_history(fixture.path())
                .expect("history should load")
                .expect("history should exist")
                .theme,
            AbpTheme::LeptonX
        );
    }

    #[test]
    fn initializes_missing_history_with_the_detected_theme() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let web = fixture.path().join("src/Acme.Demo.Web");
        std::fs::create_dir_all(&web).expect("web project should exist");
        std::fs::write(
            web.join("DemoWebModule.cs"),
            "[DependsOn(typeof(AbpAspNetCoreMvcUiLeptonXLiteThemeModule))]",
        )
        .expect("web module should exist");

        let outcome = configure_project(
            ConfigureProjectRequest {
                project_root: fixture.path().to_path_buf(),
                theme: None,
                bootstrap_override: None,
                mobile_ui: None,
                dry_run: false,
            },
            None,
        )
        .expect("configuration should initialize history");

        assert_eq!(outcome.theme, AbpTheme::LeptonX);
        assert!(!outcome.changed);
        assert_eq!(
            load_history(fixture.path())
                .expect("history should load")
                .expect("history should be initialized")
                .theme,
            AbpTheme::LeptonX
        );
    }

    #[test]
    fn explicit_theme_selection_reapplies_for_interrupted_change_recovery() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        std::fs::create_dir_all(fixture.path().join("src/Acme.Demo.Domain"))
            .expect("domain project should exist");
        save_history(
            fixture.path(),
            &ProjectRecord {
                project_name: "Demo".into(),
                project_dir: fixture.path().display().to_string(),
                theme: AbpTheme::Basic,
                bootstrap_override: BootstrapOverride::None,
                mobile_ui: MobileUi::None,
                entities: Vec::new(),
            },
        )
        .expect("history should save");

        let error = configure_project(
            ConfigureProjectRequest {
                project_root: fixture.path().to_path_buf(),
                theme: Some(AbpTheme::Basic),
                bootstrap_override: None,
                mobile_ui: None,
                dry_run: false,
            },
            None,
        )
        .expect_err("explicit theme should attempt the missing host operation");

        assert!(error.to_string().contains("theme change requires"));
    }
}

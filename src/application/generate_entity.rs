use crate::adapters::{DryRunCommandRunner, ProcessCommandRunner, StagedProject};
use crate::application::migrate::{MigrationRequest, run_ef_migration_with};
use crate::generator;
use crate::models::{Field, MobileUi, UiTarget};
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub(crate) struct GenerationOptions {
    pub no_merge: bool,
    pub run_migration: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct GenerateEntityRequest {
    pub project_root: PathBuf,
    pub domain: String,
    pub namespace: String,
    pub entity: String,
    pub fields: Vec<Field>,
    pub ui_target: UiTarget,
    pub mobile_ui: MobileUi,
    pub options: GenerationOptions,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GenerationEventKind {
    Created,
    Modified,
    Skipped,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct GenerationEvent {
    pub kind: GenerationEventKind,
    pub message: String,
}

impl GenerationEvent {
    fn from_message(message: &str) -> Self {
        let normalized = message.trim_start().to_ascii_lowercase();
        let kind = if normalized.starts_with("created") {
            GenerationEventKind::Created
        } else if normalized.starts_with("merged")
            || normalized.starts_with("updated")
            || normalized.starts_with("patched")
            || normalized.starts_with("localized")
            || normalized.starts_with("l10n+")
        {
            GenerationEventKind::Modified
        } else if normalized.starts_with("skip") || normalized.starts_with("l10n=") {
            GenerationEventKind::Skipped
        } else if normalized.starts_with("warn") {
            GenerationEventKind::Warning
        } else {
            GenerationEventKind::Info
        };

        Self {
            kind,
            message: message.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct GenerationOutcome {
    pub project_root: PathBuf,
    pub entity: String,
    pub dry_run: bool,
    pub events: Vec<GenerationEvent>,
}

pub(crate) fn generate_entity(
    request: GenerateEntityRequest,
    logger: Option<&mut dyn FnMut(&str)>,
) -> Result<GenerationOutcome> {
    validate(&request)?;

    let mut events = Vec::new();
    let staged = request
        .options
        .dry_run
        .then(|| StagedProject::copy_from(&request.project_root))
        .transpose()?;
    let execution_root = staged
        .as_ref()
        .map_or(request.project_root.as_path(), StagedProject::root);
    let original_root_text = request.project_root.display().to_string();
    let execution_root_text = execution_root.display().to_string();
    let generator_request = generator::GenerationRequest {
        project_root: execution_root,
        domain: &request.domain,
        namespace: &request.namespace,
        entity: &request.entity,
        fields: &request.fields,
        ui_target: request.ui_target,
        mobile_ui: request.mobile_ui,
        no_merge: request.options.no_merge,
    };
    {
        let mut logger = logger;
        let mut report = |message: &str| {
            let message = if request.options.dry_run {
                message.replace(&execution_root_text, &original_root_text)
            } else {
                message.to_owned()
            };
            events.push(GenerationEvent::from_message(&message));
            if let Some(logger) = logger.as_mut() {
                logger(&message);
            }
        };
        if request.options.dry_run {
            report(
                "dry-run: generating in an isolated workspace; no project files or external systems will be changed",
            );
        }
        let process_runner = ProcessCommandRunner;
        let dry_run_runner = DryRunCommandRunner;
        let runner = if request.options.dry_run {
            &dry_run_runner as &dyn crate::ports::CommandRunner
        } else {
            &process_runner as &dyn crate::ports::CommandRunner
        };
        generator::run_generate(&generator_request, Some(&mut report), runner)?;
        if request.options.run_migration {
            if request.options.dry_run {
                report("dry-run: simulating EF Core migration and database update");
            } else {
                report("Running EF Core migration and database update...");
            }
            run_ef_migration_with(
                runner,
                &MigrationRequest {
                    project_root: execution_root.to_path_buf(),
                    domain: request.domain.clone(),
                    entity: request.entity.clone(),
                },
                &mut report,
            )?;
        }
    }

    Ok(GenerationOutcome {
        project_root: request.project_root,
        entity: request.entity,
        dry_run: request.options.dry_run,
        events,
    })
}

fn validate(request: &GenerateEntityRequest) -> Result<()> {
    if !request.project_root.join("src").is_dir() {
        bail!(
            "project root must contain a src directory: {}",
            request.project_root.display()
        );
    }
    if request.domain.trim().is_empty() {
        bail!("domain cannot be empty");
    }
    if !is_valid_qualified_name(&request.domain) {
        bail!(
            "domain must be a dot-separated identifier: {}",
            request.domain
        );
    }
    if request.namespace.trim().is_empty() {
        bail!("namespace cannot be empty");
    }
    if !is_valid_qualified_name(&request.namespace) {
        bail!(
            "namespace must be a dot-separated identifier: {}",
            request.namespace
        );
    }
    if request.entity.trim().is_empty() {
        bail!("entity cannot be empty");
    }
    if !is_valid_identifier(&request.entity) {
        bail!("entity must be a valid identifier: {}", request.entity);
    }
    if request.fields.is_empty() {
        bail!("at least one field is required");
    }

    let reserved = [
        "id",
        "concurrencystamp",
        "extraproperties",
        "creationtime",
        "creatorid",
        "lastmodificationtime",
        "lastmodifierid",
        "isdeleted",
        "deleterid",
        "deletiontime",
        "filtertext",
    ];
    let mut names = HashSet::new();
    for field in &request.fields {
        if !is_valid_identifier(&field.name) {
            bail!("field must have a valid identifier: {}", field.name);
        }
        let normalized = field.name.to_ascii_lowercase();
        if reserved.contains(&normalized.as_str()) {
            bail!("field name is reserved by ABP or Banna: {}", field.name);
        }
        if !names.insert(normalized) {
            bail!("field names must be unique: {}", field.name);
        }
    }

    Ok(())
}

fn is_valid_qualified_name(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_valid_identifier)
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GenerateEntityRequest {
        GenerateEntityRequest {
            project_root: PathBuf::from("/missing/project"),
            domain: "Acme.Billing".into(),
            namespace: "Acme.Billing.Invoices".into(),
            entity: "Invoice".into(),
            fields: vec![Field {
                name: "Number".into(),
                ftype: "string".into(),
                required: true,
                navigation_display: None,
                max_length: Some(64),
                navigation: None,
                filterable: true,
                show_in_ui: true,
            }],
            ui_target: UiTarget::None,
            mobile_ui: MobileUi::None,
            options: GenerationOptions::default(),
        }
    }

    #[test]
    fn rejects_a_project_without_a_src_directory() {
        let error = validate(&request()).expect_err("missing project should fail");
        assert!(error.to_string().contains("must contain a src directory"));
    }

    #[test]
    fn rejects_an_empty_entity_name() {
        let mut request = request();
        request.entity.clear();
        request.project_root = std::env::temp_dir();

        // The project-path guard runs first; test the remaining invariant directly
        // with a path that always contains a directory named `src` in this repository.
        request.project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let error = validate(&request).expect_err("empty entity should fail");
        assert_eq!(error.to_string(), "entity cannot be empty");
    }

    #[test]
    fn rejects_unsafe_reserved_and_duplicate_identifiers() {
        let mut invalid_entity = request();
        invalid_entity.project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        invalid_entity.entity = "../Invoice".into();
        assert!(
            validate(&invalid_entity)
                .expect_err("path-like entity should fail")
                .to_string()
                .contains("valid identifier")
        );

        let mut reserved_field = request();
        reserved_field.project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        reserved_field.fields[0].name = "ConcurrencyStamp".into();
        assert!(
            validate(&reserved_field)
                .expect_err("reserved field should fail")
                .to_string()
                .contains("reserved")
        );

        let mut duplicate_fields = request();
        duplicate_fields.project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        duplicate_fields
            .fields
            .push(duplicate_fields.fields[0].clone());
        duplicate_fields.fields[1].name = "number".into();
        assert!(
            validate(&duplicate_fields)
                .expect_err("case-insensitive duplicate should fail")
                .to_string()
                .contains("unique")
        );
    }

    #[test]
    fn classifies_generation_messages_for_machine_readable_reports() {
        assert_eq!(
            GenerationEvent::from_message("created: Invoice.cs").kind,
            GenerationEventKind::Created
        );
        assert_eq!(
            GenerationEvent::from_message("merged: AppModule.cs").kind,
            GenerationEventKind::Modified
        );
        assert_eq!(
            GenerationEvent::from_message("skip: existing file").kind,
            GenerationEventKind::Skipped
        );
        assert_eq!(
            GenerationEvent::from_message("warn: unsupported layout").kind,
            GenerationEventKind::Warning
        );
    }
}

use crate::application::generate_entity::{
    GenerateEntityRequest, GenerationOptions, GenerationOutcome, generate_entity,
};
use crate::application::history::{load_history, save_history};
use anyhow::{Result, bail};
use chrono::Local;
use std::path::PathBuf;

pub(crate) struct RegenerateEntityRequest {
    pub project_root: PathBuf,
    pub entity: String,
    pub options: GenerationOptions,
}

pub(crate) fn regenerate_entity(
    request: RegenerateEntityRequest,
    logger: Option<&mut dyn FnMut(&str)>,
) -> Result<GenerationOutcome> {
    let Some(mut history) = load_history(&request.project_root)? else {
        bail!(
            "project history not found: {}",
            request.project_root.display()
        );
    };
    let Some(position) = history
        .entities
        .iter()
        .position(|entity| entity.name == request.entity)
    else {
        bail!("entity not found in project history: {}", request.entity);
    };
    let entity = history.entities[position].clone();
    let dry_run = request.options.dry_run;

    let outcome = generate_entity(
        GenerateEntityRequest {
            project_root: request.project_root.clone(),
            domain: entity.domain,
            namespace: entity.namespace,
            entity: entity.name,
            fields: entity.fields,
            ui_target: entity.ui_target,
            mobile_ui: history.mobile_ui,
            options: request.options,
        },
        logger,
    )?;

    if !dry_run {
        history.entities[position].generated_at = Local::now();
        save_history(&request.project_root, &history)?;
    }
    Ok(outcome)
}

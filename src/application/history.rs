use crate::adapters::JsonProjectHistoryStore;
use crate::models::{EntityRecord, ProjectRecord};
use crate::ports::ProjectHistoryStore;
use anyhow::{Result, bail};
use std::path::Path;

pub(crate) fn load_history(project_root: &Path) -> Result<Option<ProjectRecord>> {
    JsonProjectHistoryStore::for_project(project_root).load()
}

pub(crate) fn save_history(project_root: &Path, record: &ProjectRecord) -> Result<()> {
    JsonProjectHistoryStore::for_project(project_root).save(record)
}

pub(crate) fn list_history(project_root: &Path) -> Result<Vec<EntityRecord>> {
    Ok(load_history(project_root)?
        .map(|record| record.entities)
        .unwrap_or_default())
}

pub(crate) fn remove_history_entity(
    project_root: &Path,
    entity_name: &str,
) -> Result<EntityRecord> {
    let store = JsonProjectHistoryStore::for_project(project_root);
    let Some(mut record) = store.load()? else {
        bail!("project history not found: {}", project_root.display());
    };
    let Some(position) = record
        .entities
        .iter()
        .position(|entity| entity.name == entity_name)
    else {
        bail!("entity not found in project history: {entity_name}");
    };

    let removed = record.entities.remove(position);
    store.save(&record)?;
    Ok(removed)
}

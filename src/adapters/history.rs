use crate::models::ProjectRecord;
use crate::ports::ProjectHistoryStore;
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const HISTORY_DIRECTORY: &str = ".history";
const HISTORY_FILE: &str = "codegen_history.json";

pub(crate) struct JsonProjectHistoryStore {
    path: PathBuf,
}

impl JsonProjectHistoryStore {
    pub(crate) fn for_project(project_root: &Path) -> Self {
        Self {
            path: project_root.join(HISTORY_DIRECTORY).join(HISTORY_FILE),
        }
    }
}

impl ProjectHistoryStore for JsonProjectHistoryStore {
    fn load(&self) -> Result<Option<ProjectRecord>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read project history: {}", self.path.display()))?;
        serde_json::from_str(&contents)
            .map(Some)
            .with_context(|| format!("invalid project history: {}", self.path.display()))
    }

    fn save(&self, record: &ProjectRecord) -> Result<()> {
        let directory = self.path.parent().ok_or_else(|| {
            anyhow!(
                "project history path has no parent: {}",
                self.path.display()
            )
        })?;
        fs::create_dir_all(directory).with_context(|| {
            format!(
                "failed to create history directory: {}",
                directory.display()
            )
        })?;

        let mut temporary = NamedTempFile::new_in(directory).with_context(|| {
            format!(
                "failed to create temporary history in {}",
                directory.display()
            )
        })?;
        serde_json::to_writer_pretty(&mut temporary, record)?;
        temporary.flush()?;
        temporary
            .persist(&self.path)
            .map_err(|error| error.error)
            .with_context(|| format!("failed to save project history: {}", self.path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AbpTheme, BootstrapOverride, MobileUi};

    fn record(project_root: &Path) -> ProjectRecord {
        ProjectRecord {
            project_name: "Demo".into(),
            project_dir: project_root.display().to_string(),
            theme: AbpTheme::Basic,
            bootstrap_override: BootstrapOverride::None,
            mobile_ui: MobileUi::None,
            entities: Vec::new(),
        }
    }

    #[test]
    fn round_trips_history_atomically() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let store = JsonProjectHistoryStore::for_project(fixture.path());
        assert!(store.load().expect("missing history should load").is_none());

        store
            .save(&record(fixture.path()))
            .expect("history should save");
        let loaded = store
            .load()
            .expect("history should load")
            .expect("history should exist");
        assert_eq!(loaded.project_name, "Demo");
        assert!(
            fixture
                .path()
                .join(HISTORY_DIRECTORY)
                .join(HISTORY_FILE)
                .exists()
        );
    }

    #[test]
    fn reports_invalid_history_instead_of_hiding_it() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        let directory = fixture.path().join(HISTORY_DIRECTORY);
        fs::create_dir_all(&directory).expect("history directory should exist");
        fs::write(directory.join(HISTORY_FILE), "not json").expect("fixture should be written");
        let store = JsonProjectHistoryStore::for_project(fixture.path());

        let error = store.load().expect_err("invalid history should fail");
        assert!(error.to_string().contains("invalid project history"));
    }
}

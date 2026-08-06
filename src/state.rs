use crate::models::GlobalIndex;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Furkan Cemal Caliskan";
const APPLICATION: &str = "banna";

fn global_index_path() -> PathBuf {
    if let Some(config_dir) = std::env::var_os("BANNA_CONFIG_DIR") {
        return PathBuf::from(config_dir).join("projects.json");
    }
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).map_or_else(
        || PathBuf::from(".banna-projects.json"),
        |project_dirs| project_dirs.config_dir().join("projects.json"),
    )
}

pub(crate) trait ProjectIndexStore {
    fn load(&self) -> Result<GlobalIndex>;
    fn save(&self, index: &GlobalIndex) -> Result<()>;
}

pub(crate) struct JsonProjectIndexStore {
    path: PathBuf,
}

impl JsonProjectIndexStore {
    pub(crate) fn platform_default() -> Self {
        Self {
            path: global_index_path(),
        }
    }

    #[cfg(test)]
    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ProjectIndexStore for JsonProjectIndexStore {
    fn load(&self) -> Result<GlobalIndex> {
        if !self.path.exists() {
            return Ok(GlobalIndex::default());
        }

        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read project index: {}", self.path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("invalid project index: {}", self.path.display()))
    }

    fn save(&self, index: &GlobalIndex) -> Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory: {}", parent.display()))?;

        let mut temporary = NamedTempFile::new_in(parent).with_context(|| {
            format!(
                "failed to create temporary project index in {}",
                parent.display()
            )
        })?;
        serde_json::to_writer_pretty(&mut temporary, index)?;
        temporary.flush()?;
        temporary
            .persist(&self.path)
            .map_err(|error| error.error)
            .with_context(|| format!("failed to save project index: {}", self.path.display()))?;
        Ok(())
    }
}

pub(crate) fn load_global_index() -> Result<GlobalIndex> {
    JsonProjectIndexStore::platform_default().load()
}

pub(crate) fn save_global_index(index: &GlobalIndex) -> Result<()> {
    JsonProjectIndexStore::platform_default().save(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectRef;

    #[test]
    fn missing_index_loads_as_empty() {
        let directory = tempfile::tempdir().expect("test directory should exist");
        let store = JsonProjectIndexStore::at(directory.path().join("projects.json"));
        assert!(
            store
                .load()
                .expect("missing index should load")
                .projects
                .is_empty()
        );
    }

    #[test]
    fn saves_and_loads_the_project_index() {
        let directory = tempfile::tempdir().expect("test directory should exist");
        let store = JsonProjectIndexStore::at(directory.path().join("projects.json"));
        let index = GlobalIndex {
            projects: vec![ProjectRef {
                project_name: "Demo".into(),
                project_dir: "/tmp/demo".into(),
            }],
        };

        store.save(&index).expect("index should save");
        assert_eq!(store.load().expect("index should load"), index);
    }
}

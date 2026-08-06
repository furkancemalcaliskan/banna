use crate::models::ProjectRef;
use crate::state::ProjectIndexStore;
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub(crate) fn list_projects(store: &dyn ProjectIndexStore) -> Result<Vec<ProjectRef>> {
    Ok(store.load()?.projects)
}

pub(crate) fn add_project(
    store: &dyn ProjectIndexStore,
    name: Option<String>,
    path: PathBuf,
) -> Result<ProjectRef> {
    validate_project_path(&path)?;
    let canonical_path = path.canonicalize()?;
    let project_name = name
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            canonical_path
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_owned)
        })
        .ok_or_else(|| anyhow::anyhow!("project name could not be inferred; pass --name"))?;

    let mut index = store.load()?;
    if index
        .projects
        .iter()
        .any(|project| project.project_name == project_name)
    {
        bail!("a project named '{project_name}' already exists");
    }
    if index
        .projects
        .iter()
        .any(|project| Path::new(&project.project_dir) == canonical_path)
    {
        bail!(
            "project path is already registered: {}",
            canonical_path.display()
        );
    }

    let project = ProjectRef {
        project_name,
        project_dir: canonical_path.to_string_lossy().into_owned(),
    };
    index.projects.push(project.clone());
    store.save(&index)?;
    Ok(project)
}

pub(crate) fn remove_project(store: &dyn ProjectIndexStore, name: &str) -> Result<ProjectRef> {
    let mut index = store.load()?;
    let Some(position) = index
        .projects
        .iter()
        .position(|project| project.project_name == name)
    else {
        bail!("project not found: {name}");
    };

    let removed = index.projects.remove(position);
    store.save(&index)?;
    Ok(removed)
}

fn validate_project_path(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("project path is not a directory: {}", path.display());
    }
    if !path.join("src").is_dir() {
        bail!(
            "project root must contain a src directory: {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GlobalIndex;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MemoryStore(RefCell<GlobalIndex>);

    impl ProjectIndexStore for MemoryStore {
        fn load(&self) -> Result<GlobalIndex> {
            Ok(self.0.borrow().clone())
        }

        fn save(&self, index: &GlobalIndex) -> Result<()> {
            *self.0.borrow_mut() = index.clone();
            Ok(())
        }
    }

    #[test]
    fn adds_lists_and_removes_a_project_without_touching_its_files() {
        let directory = tempfile::tempdir().expect("project fixture should exist");
        std::fs::create_dir(directory.path().join("src")).expect("project src should be created");
        let store = MemoryStore::default();

        let project = add_project(&store, Some("Demo".into()), directory.path().into())
            .expect("project should be added");
        assert_eq!(
            list_projects(&store).expect("projects should list"),
            vec![project]
        );

        let removed = remove_project(&store, "Demo").expect("project should be removed");
        assert_eq!(removed.project_name, "Demo");
        assert!(directory.path().exists());
        assert!(
            list_projects(&store)
                .expect("projects should list")
                .is_empty()
        );
    }

    #[test]
    fn rejects_duplicate_names() {
        let first = tempfile::tempdir().expect("first fixture should exist");
        let second = tempfile::tempdir().expect("second fixture should exist");
        std::fs::create_dir(first.path().join("src")).expect("first src should be created");
        std::fs::create_dir(second.path().join("src")).expect("second src should be created");
        let store = MemoryStore::default();

        add_project(&store, Some("Demo".into()), first.path().into())
            .expect("first project should be added");
        let error = add_project(&store, Some("Demo".into()), second.path().into())
            .expect_err("duplicate should fail");
        assert_eq!(error.to_string(), "a project named 'Demo' already exists");
    }
}

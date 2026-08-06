use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

const EXCLUDED_DIRECTORIES: &[&str] = &[".git", "target", "node_modules", "bin", "obj"];

pub(crate) struct StagedProject {
    _temporary: TempDir,
    root: PathBuf,
}

impl StagedProject {
    pub(crate) fn copy_from(source: &Path) -> Result<Self> {
        let temporary = tempfile::tempdir().context("failed to create dry-run workspace")?;
        let root = temporary.path().join("project");
        fs::create_dir(&root).context("failed to initialize dry-run workspace")?;

        for entry in WalkDir::new(source)
            .follow_links(false)
            .into_iter()
            .filter_entry(should_copy)
        {
            let entry = entry.with_context(|| {
                format!(
                    "failed to traverse project for dry-run: {}",
                    source.display()
                )
            })?;
            let relative = entry.path().strip_prefix(source)?;
            if relative.as_os_str().is_empty() {
                continue;
            }
            let destination = root.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&destination)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &destination).with_context(|| {
                    format!(
                        "failed to stage file for dry-run: {}",
                        entry.path().display()
                    )
                })?;
            } else if entry.file_type().is_symlink() {
                copy_symlink(entry.path(), &destination)?;
            }
        }

        Ok(Self {
            _temporary: temporary,
            root,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

fn should_copy(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || !entry.file_type().is_dir()
        || !EXCLUDED_DIRECTORIES
            .iter()
            .any(|excluded| entry.file_name() == OsStr::new(excluded))
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    symlink(fs::read_link(source)?, destination)?;
    Ok(())
}

#[cfg(windows)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<()> {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let target = fs::read_link(source)?;
    if source.is_dir() {
        symlink_dir(target, destination)?;
    } else {
        symlink_file(target, destination)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_sources_but_skips_build_and_dependency_trees() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/App")).expect("src should exist");
        fs::create_dir_all(fixture.path().join("target/debug")).expect("target should exist");
        fs::create_dir_all(fixture.path().join("node_modules/pkg"))
            .expect("dependencies should exist");
        fs::write(fixture.path().join("src/App/source.cs"), "source").expect("source should exist");
        fs::write(fixture.path().join("target/debug/output"), "build")
            .expect("build output should exist");

        let staged = StagedProject::copy_from(fixture.path()).expect("project should stage");

        assert!(staged.root().join("src/App/source.cs").exists());
        assert!(!staged.root().join("target").exists());
        assert!(!staged.root().join("node_modules").exists());
    }
}

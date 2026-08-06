use crate::models::{AbpTheme, BootstrapOverride, EntityRecord, Field, MobileUi, UiTarget};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc;

pub(super) struct JobState {
    pub(super) rx: mpsc::Receiver<JobEvent>,
    pub(super) handle: std::thread::JoinHandle<Result<()>>,
    pub(super) action: JobAction,
}

pub(super) enum JobEvent {
    Log(String),
}

pub(super) enum JobAction {
    AddNew {
        root: PathBuf,
        record: EntityRecord,
    },
    UpdateGeneratedAt {
        root: PathBuf,
        ent_idx: usize,
    },
    ProjectMetaChange {
        theme: AbpTheme,
        override_css: BootstrapOverride,
        theme_changed: bool,
        mobile_ui: MobileUi,
        mobile_changed: bool,
    },
}

pub(super) struct GenerationTask {
    pub(super) root: PathBuf,
    pub(super) domain: String,
    pub(super) namespace: String,
    pub(super) entity: String,
    pub(super) fields: Vec<Field>,
    pub(super) ui_target: UiTarget,
    pub(super) mobile_ui: MobileUi,
    pub(super) run_migration: bool,
}

pub(super) struct PendingGeneration {
    pub(super) root: PathBuf,
    pub(super) domain: String,
    pub(super) namespace: String,
    pub(super) entity: String,
    pub(super) fields: Vec<Field>,
    pub(super) ui_target: UiTarget,
    pub(super) run_migration: bool,
}

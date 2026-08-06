use super::{FieldCol, PendingGeneration};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Default)]
pub(super) struct WorkflowState(Arc<Mutex<WorkflowMailbox>>);

#[derive(Default)]
struct WorkflowMailbox {
    created_project: Option<(PathBuf, String)>,
    generation: Option<PendingGeneration>,
    renamed_project: Option<String>,
    wizard_field_meta: Option<(bool, bool)>,
    field_edit: Option<(usize, FieldCol, String)>,
    meta_edit: Option<String>,
}

impl WorkflowState {
    fn mailbox(&self) -> MutexGuard<'_, WorkflowMailbox> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn complete_project_creation(&self, root: PathBuf, name: String) {
        self.mailbox().created_project = Some((root, name));
    }

    pub(super) fn take_created_project(&self) -> Option<(PathBuf, String)> {
        self.mailbox().created_project.take()
    }

    pub(super) fn complete_generation(&self, generation: PendingGeneration) {
        self.mailbox().generation = Some(generation);
    }

    pub(super) fn take_generation(&self) -> Option<PendingGeneration> {
        self.mailbox().generation.take()
    }

    pub(super) fn complete_project_rename(&self, name: String) {
        self.mailbox().renamed_project = Some(name);
    }

    pub(super) fn take_project_rename(&self) -> Option<String> {
        self.mailbox().renamed_project.take()
    }

    pub(super) fn set_wizard_field_meta(&self, filterable: bool, show_in_ui: bool) {
        self.mailbox().wizard_field_meta = Some((filterable, show_in_ui));
    }

    pub(super) fn take_wizard_field_meta(&self, defaults: (bool, bool)) -> (bool, bool) {
        self.mailbox().wizard_field_meta.take().unwrap_or(defaults)
    }

    pub(super) fn complete_field_edit(&self, row: usize, column: FieldCol, value: String) {
        self.mailbox().field_edit = Some((row, column, value));
    }

    pub(super) fn take_field_edit(&self) -> Option<(usize, FieldCol, String)> {
        self.mailbox().field_edit.take()
    }

    pub(super) fn complete_meta_edit(&self, value: String) {
        self.mailbox().meta_edit = Some(value);
    }

    pub(super) fn take_meta_edit(&self) -> Option<String> {
        self.mailbox().meta_edit.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_state_shares_and_consumes_typed_completions_once() {
        let state = WorkflowState::default();
        let callback_state = state.clone();
        callback_state.complete_field_edit(2, FieldCol::Name, "Number".into());

        assert!(matches!(
            state.take_field_edit(),
            Some((2, FieldCol::Name, value)) if value == "Number"
        ));
        assert!(state.take_field_edit().is_none());
    }

    #[test]
    fn wizard_metadata_uses_explicit_defaults_when_absent() {
        let state = WorkflowState::default();
        assert_eq!(state.take_wizard_field_meta((true, false)), (true, false));
        state.set_wizard_field_meta(false, true);
        assert_eq!(state.take_wizard_field_meta((true, false)), (false, true));
    }
}

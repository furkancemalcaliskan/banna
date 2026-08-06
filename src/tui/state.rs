#[derive(Clone)]
pub(super) enum PendingAction {
    ExitProgram,
    DeleteProject,
    OpenProjectBack,
    DeleteEntity,
    RegenEntity,
    CancelEditFields,
    SaveEditFields,
    CancelEditMeta,
    SaveEditMeta,
    CancelCreateProjectWizard,
    CancelCreateEntityWizard,
    CancelInlineConsole,
}

pub(super) struct ConfirmState {
    pub(super) message: String,
    pub(super) ok_selected: bool,
    pub(super) action: PendingAction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Screen {
    Projects,
    ProjectDetail,
}

#[derive(Clone)]
pub(super) struct FieldRow {
    pub(super) name: String,
    pub(super) ftype: String,
    pub(super) required: bool,
    pub(super) max_len: Option<i32>,
    pub(super) nav_name: Option<String>,
    pub(super) nav_ns: Option<String>,
    pub(super) nav_display: Option<String>,
    pub(super) filterable: bool,
    pub(super) show_ui: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FieldCol {
    Name,
    Type,
    Required,
    Filterable,
    ShowUi,
    MaxLen,
    NavName,
    NavNs,
    NavDisplay,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum MetaTarget {
    Entity(usize),
    Project,
}

pub(super) enum Modal {
    None,
    EditFields {
        ent_idx: usize,
        rows: Vec<FieldRow>,
        row: usize,
        col: FieldCol,
        dirty: bool,
        dragging: bool,
    },
    EditMeta {
        target: MetaTarget,
        rows: Vec<(String, String)>,
        row: usize,
        col: usize,
    },
}

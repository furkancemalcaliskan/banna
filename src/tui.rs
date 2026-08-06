mod choices;
mod console;
mod entity_wizard;
mod input;
mod jobs;
mod layout;
mod modal_input;
mod navigation;
mod render;
mod screen_input;
mod state;
mod workflow;

use self::choices::{
    cycle_next_mobile_ui, cycle_next_override, cycle_next_theme, cycle_next_ui, mobile_ui_label,
    override_label, sanitize_mobile_ui_choice, sanitize_override_choice, sanitize_theme_choice,
    sanitize_ui_choice, theme_label,
};
use self::console::{ConsoleMode, InlineConsole, InputHint, prompt};
use self::entity_wizard::EntityWizard;
use self::input::{handle_confirm_key, handle_console_key, handle_help_key};
use self::jobs::{GenerationTask, JobAction, JobEvent, JobState, PendingGeneration};
use self::layout::centered_rect;
use self::modal_input::handle_modal_key;
use self::navigation::{vim_go_bottom, vim_go_top, wrap_next, wrap_prev};
use self::render::{
    draw_confirm_modal, draw_help_modal, draw_inline_console, draw_list_modal,
    draw_project_detail_ui, draw_projects_ui,
};
use self::screen_input::handle_screen_key;
use self::state::{ConfirmState, FieldCol, FieldRow, MetaTarget, Modal, PendingAction, Screen};
use self::workflow::WorkflowState;
use crate::Cli;
use crate::application::{
    ConfigureProjectRequest, GenerateEntityRequest, GenerationOptions, configure_project,
    detect_abp_theme, generate_entity, load_history, save_history,
};
use crate::generator;
use crate::helpers::{VERBOSE, last_segment, ns_of};
use crate::models::{
    AbpTheme, BootstrapOverride, EntityRecord, Field, GlobalIndex, MobileUi, ProjectRecord,
    ProjectRef, UiTarget,
};
use crate::state::{load_global_index, save_global_index};
use crate::theme::{THEME, accent, block, block_muted, danger, faint, list_highlight, success};
use anyhow::{Result, anyhow};
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal as cterm};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, mpsc};
use std::time::Duration;

/* -------------------------- HISTORY (per-project) ---------------------- */

fn load_project_history(project_root: &Path) -> Result<Option<ProjectRecord>> {
    load_history(project_root)
}
fn save_project_history(project_root: &Path, record: &ProjectRecord) -> Result<()> {
    save_history(project_root, record)
}

fn project_record_for(root: &Path, project: &ProjectRef) -> Result<ProjectRecord> {
    let mut record = load_project_history(root)?.unwrap_or_else(|| ProjectRecord {
        project_name: project.project_name.clone(),
        project_dir: project.project_dir.clone(),
        theme: AbpTheme::Basic,
        bootstrap_override: BootstrapOverride::None,
        mobile_ui: MobileUi::None,
        entities: Vec::new(),
    });
    if let Some(theme) = detect_abp_theme(root)? {
        record.theme = theme;
    }
    Ok(record)
}

fn reconcile_project_record(root: &Path, project: &ProjectRef) -> Result<ProjectRecord> {
    let stored = load_project_history(root)?;
    let record = project_record_for(root, project)?;
    if stored
        .as_ref()
        .is_none_or(|stored| stored.theme != record.theme)
    {
        save_project_history(root, &record)?;
    }
    Ok(record)
}

/* ------------------------------ INLINE MODAL TERMINAL ------------------ */

struct App<'a> {
    cli: &'a Cli,
    global: GlobalIndex,
    current_project: Option<ProjectRef>,
    current_project_data: Option<ProjectRecord>,
    list_index: usize,
    screen: Screen,
    status: String,
    console: InlineConsole,
    modal: Modal,
    confirm: Option<ConfirmState>,
    vim_pending_g: bool,
    help_open: bool,
    job: Option<JobState>,
    entity_dragging: bool,
    entity_drag_dirty: bool,
    project_dragging: bool,
    project_drag_dirty: bool,
    workflow: WorkflowState,
}

impl<'a> App<'a> {
    fn new(cli: &'a Cli) -> Self {
        let global = load_global_index().unwrap_or_else(|error| {
            eprintln!("warning: {error}");
            GlobalIndex::default()
        });
        let mut reconciliation_errors = Vec::new();
        if !cli.dry_run {
            for project in &global.projects {
                let root = PathBuf::from(&project.project_dir);
                if let Err(error) = reconcile_project_record(&root, project) {
                    reconciliation_errors.push(format!("{}: {error}", project.project_name));
                }
            }
        }
        let status = if reconciliation_errors.is_empty() {
            "Ready".into()
        } else {
            format!(
                "Metadata detection warning: {}",
                reconciliation_errors.join("; ")
            )
        };

        Self {
            cli,
            global,
            current_project: None,
            current_project_data: None,
            list_index: 0,
            screen: Screen::Projects,
            status,
            console: InlineConsole::new("Interactive"),
            modal: Modal::None,
            confirm: None,
            vim_pending_g: false,
            help_open: false,
            job: None,
            entity_dragging: false,
            entity_drag_dirty: false,
            project_dragging: false,
            project_drag_dirty: false,
            workflow: WorkflowState::default(),
        }
    }

    fn confirm<S: Into<String>>(&mut self, message: S, action: PendingAction) {
        self.confirm = Some(ConfirmState {
            message: message.into(),
            ok_selected: false,
            action,
        });
        self.status = "Confirmation required".into();
    }

    fn clear_confirm(&mut self) {
        self.confirm = None;
        self.vim_pending_g = false;
        self.help_open = false;
    }

    fn start_generate_job(&mut self, task: GenerationTask, action: JobAction) {
        let title = format!("Generating {}", task.entity);
        let mut console = InlineConsole::new_logs(title);
        console.open();
        console.println("Starting...");
        console.auto_scroll = true;
        self.console = console;

        let (tx, rx) = mpsc::channel::<JobEvent>();
        let log_tx = tx.clone();
        let mut log = move |msg: &str| {
            let _ = log_tx.send(JobEvent::Log(msg.to_string()));
        };
        let no_merge = self.cli.no_merge;
        let dry_run = self.cli.dry_run;
        let handle = std::thread::spawn(move || {
            let request = GenerateEntityRequest {
                project_root: task.root.clone(),
                domain: task.domain.clone(),
                namespace: task.namespace.clone(),
                entity: task.entity.clone(),
                fields: task.fields.clone(),
                ui_target: task.ui_target,
                mobile_ui: task.mobile_ui,
                options: GenerationOptions {
                    no_merge,
                    run_migration: task.run_migration,
                    dry_run,
                },
            };
            generate_entity(request, Some(&mut log))?;

            log("Done. Press q/Esc/Enter to close this log.");
            Ok(())
        });

        self.job = Some(JobState { rx, handle, action });
        self.status = "Running generation...".into();
    }

    fn start_project_meta_job(
        &mut self,
        root: PathBuf,
        theme: AbpTheme,
        override_css: BootstrapOverride,
        mobile_ui: MobileUi,
        theme_changed: bool,
        mobile_changed: bool,
    ) {
        let title = "Applying project meta (theme/override/mobile)";
        let mut console = InlineConsole::new_logs(title);
        console.open();
        console.println("Starting...");
        console.auto_scroll = true;
        self.console = console;

        let (tx, rx) = mpsc::channel::<JobEvent>();
        let root_clone = root.clone();
        let dry_run = self.cli.dry_run;
        let mut log = move |msg: &str| {
            let _ = tx.send(JobEvent::Log(msg.to_string()));
        };

        let handle = std::thread::spawn(move || {
            configure_project(
                ConfigureProjectRequest {
                    project_root: root_clone,
                    theme: Some(theme),
                    bootstrap_override: Some(override_css),
                    mobile_ui: Some(mobile_ui),
                    dry_run,
                },
                Some(&mut log),
            )?;
            Ok(())
        });
        let action = JobAction::ProjectMetaChange {
            theme,
            override_css,
            theme_changed,
            mobile_ui,
            mobile_changed,
        };
        self.job = Some(JobState { rx, handle, action });
        self.status = "Applying project meta...".into();
    }

    fn poll_job(&mut self) {
        if let Some(job) = &self.job {
            while let Ok(event) = job.rx.try_recv() {
                match event {
                    JobEvent::Log(line) => self.console.println(line),
                }
            }
        }

        if let Some(job) = self.job.take() {
            if job.handle.is_finished() {
                match job.handle.join() {
                    Ok(Ok(())) => match job.action {
                        JobAction::AddNew { root, mut record } => {
                            if self.cli.dry_run {
                                self.status =
                                    "Dry run completed; no entity history was added".into();
                            } else {
                                record.generated_at = Local::now();
                                if let Some(hist) = self.current_project_data.as_mut() {
                                    hist.entities.push(record);
                                    let _ = save_project_history(&root, hist);
                                }
                                self.status = "Generated new entity".into();
                            }
                        }
                        JobAction::UpdateGeneratedAt { root, ent_idx } => {
                            if self.cli.dry_run {
                                self.status =
                                    "Dry run completed; entity history was not changed".into();
                            } else {
                                if let Some(hist) = self.current_project_data.as_mut() {
                                    if let Some(ent) = hist.entities.get_mut(ent_idx) {
                                        ent.generated_at = Local::now();
                                    }
                                    let _ = save_project_history(&root, hist);
                                }
                                self.status = "Regenerated".into();
                            }
                        }
                        JobAction::ProjectMetaChange {
                            theme,
                            override_css,
                            theme_changed,
                            mobile_ui,
                            mobile_changed,
                        } => {
                            if self.cli.dry_run {
                                self.status =
                                    "Dry run completed; project configuration was not changed"
                                        .into();
                            } else {
                                if let Some(history) = self.current_project_data.as_mut() {
                                    history.theme = theme;
                                    history.bootstrap_override = override_css;
                                    history.mobile_ui = mobile_ui;
                                }
                                let theme_msg = if theme_changed {
                                    format!("Theme applied: {}", theme_label(theme))
                                } else {
                                    format!("Theme unchanged ({})", theme_label(theme))
                                };
                                let override_msg =
                                    format!("Bootstrap override: {}", override_label(override_css));
                                let mobile_msg = if mobile_changed {
                                    format!("Mobile UI applied: {}", mobile_ui_label(mobile_ui))
                                } else {
                                    format!("Mobile UI unchanged ({})", mobile_ui_label(mobile_ui))
                                };
                                self.status = format!("{theme_msg}; {override_msg}; {mobile_msg}");
                            }
                        }
                    },
                    Ok(Err(e)) => {
                        self.status = format!("Error: {e}");
                        self.console.println(format!("Error: {e}"));
                    }
                    Err(e) => {
                        self.status =
                            format!("Job panicked: {:?}", e.downcast_ref::<String>().cloned());
                        self.console.println("Job panicked");
                    }
                }
                if matches!(self.console.mode, ConsoleMode::Logs) {
                    self.console.input_visible = false;
                    self.console.input_mask = false;
                    self.console.input_label = None;
                    self.console.input.clear();
                }
            } else {
                // put back if still running
                self.job = Some(job);
            }
        }
    }

    fn relist_bound(&mut self) {
        let len = match self.screen {
            Screen::Projects => self.global.projects.len(),
            Screen::ProjectDetail => self
                .current_project_data
                .as_ref()
                .map(|p| p.entities.len())
                .unwrap_or(0),
        };
        if len == 0 {
            self.list_index = 0;
        } else if self.list_index >= len {
            self.list_index = len - 1;
        }
    }

    fn move_entity(&mut self, delta: isize) {
        if let Some(hist) = self.current_project_data.as_mut() {
            if hist.entities.is_empty() {
                return;
            }
            let len = hist.entities.len() as isize;
            let cur = self.list_index as isize;
            let target = (cur + delta).clamp(0, len - 1);
            if target != cur {
                hist.entities.swap(cur as usize, target as usize);
                self.list_index = target as usize;
                self.entity_drag_dirty = true;
                self.status = "Reordering entities...".into();
            }
        }
    }

    fn finish_entity_drag(&mut self) {
        if !self.entity_dragging {
            return;
        }
        self.entity_dragging = false;
        if self.entity_drag_dirty {
            if let (Some(pref), Some(hist)) =
                (&self.current_project, self.current_project_data.as_ref())
            {
                let root = PathBuf::from(&pref.project_dir);
                match save_project_history(&root, hist) {
                    Ok(_) => self.status = "Entity order saved".into(),
                    Err(e) => self.status = format!("Error saving order: {e}"),
                }
            }
        } else {
            self.status = "Drag cancelled".into();
        }
        self.entity_drag_dirty = false;
    }

    fn move_project(&mut self, delta: isize) {
        if self.global.projects.is_empty() {
            return;
        }
        let len = self.global.projects.len() as isize;
        let cur = self.list_index as isize;
        let target = (cur + delta).clamp(0, len - 1);
        if target != cur {
            self.global.projects.swap(cur as usize, target as usize);
            self.list_index = target as usize;
            self.project_drag_dirty = true;
            self.status = "Reordering projects...".into();
        }
    }

    fn finish_project_drag(&mut self) {
        if !self.project_dragging {
            return;
        }
        self.project_dragging = false;
        if self.project_drag_dirty {
            match save_global_index(&self.global) {
                Ok(_) => self.status = "Project order saved".into(),
                Err(e) => self.status = format!("Error saving project order: {e}"),
            }
        } else {
            self.status = "Drag cancelled".into();
        }
        self.project_drag_dirty = false;
    }

    fn entity_action_delete_from_history(&mut self) -> Result<()> {
        let Some(pref) = self.current_project.clone() else {
            self.status = "No project loaded".into();
            return Ok(());
        };
        let root = PathBuf::from(&pref.project_dir);
        if let Some(hist) = self.current_project_data.as_mut() {
            if hist.entities.is_empty() {
                self.status = "No entities".into();
                return Ok(());
            }
            if self.list_index >= hist.entities.len() {
                self.status = "Invalid entity selection".into();
                self.list_index = hist.entities.len().saturating_sub(1);
                return Ok(());
            }
            let removed = hist.entities.remove(self.list_index);
            save_project_history(&root, hist)?;
            self.list_index = self.list_index.saturating_sub(1);
            self.status = format!(
                "Deleted '{}' from history (files not removed)",
                removed.name
            );
        }
        Ok(())
    }

    fn start_create_project_inline(&mut self) -> Result<()> {
        #[derive(Debug)]
        enum Phase {
            AskProjectRoot,
            AskProjectName { root: PathBuf },
            Done,
        }

        let mut phase = Phase::AskProjectRoot;
        let mut console = InlineConsole::new("Create Project (Wizard)");
        let workflow = self.workflow.clone();

        console.on_submit = Some(Box::new(move |line: String| {
            let input = line.trim();

            match &mut phase {
                Phase::AskProjectRoot => {
                    if input.is_empty() {
                        return prompt("Enter your project root (absolute path)");
                    }
                    let p = PathBuf::from(input);
                    if !p.is_absolute() || !p.exists() {
                        return prompt("Path must be an existing absolute path");
                    }
                    let default_name = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("MyProject")
                        .to_string();
                    phase = Phase::AskProjectName { root: p.clone() };
                    prompt(format!("Project name [{}]", default_name))
                }
                Phase::AskProjectName { root } => {
                    let default_name = root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("MyProject")
                        .to_string();
                    let name = if input.is_empty() {
                        default_name
                    } else {
                        input.to_string()
                    };

                    workflow.complete_project_creation(root.clone(), name);
                    phase = Phase::Done;
                    None
                }
                Phase::Done => None,
            }
        }));

        console.on_cancel = Some(Box::new(|| {}));
        console.open();
        console.println("Enter your project root (absolute path)");
        self.console = console;
        self.status = "Project create wizard started".into();
        Ok(())
    }

    fn open_selected_project(&mut self) {
        if self.global.projects.is_empty() {
            self.status = "No projects yet. Press 'n' to create.".into();
            return;
        }
        let Some(project) = self.global.projects.get(self.list_index).cloned() else {
            self.list_index = self.global.projects.len().saturating_sub(1);
            self.status = "Invalid project selection".into();
            return;
        };
        let root = PathBuf::from(&project.project_dir);
        let record = match if self.cli.dry_run {
            project_record_for(&root, &project)
        } else {
            reconcile_project_record(&root, &project)
        } {
            Ok(record) => record,
            Err(error) => {
                self.status = format!("Error loading project metadata: {error}");
                return;
            }
        };
        self.current_project_data = Some(record);
        self.current_project = Some(project);
        self.screen = Screen::ProjectDetail;
        self.list_index = 0;
        self.status = "Opened project".into();
        self.vim_pending_g = false;
        self.help_open = false;
    }

    fn create_new_entity(&mut self, _term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        let pref = self
            .current_project
            .clone()
            .ok_or_else(|| anyhow!("No project loaded"))?;
        let root = PathBuf::from(&pref.project_dir);

        let mut wizard = EntityWizard::new(root, self.workflow.clone());
        let mut console = InlineConsole::new("Create Entity (Wizard)");
        console.on_submit = Some(Box::new(move |line| wizard.submit(line)));
        console.on_cancel = Some(Box::new(|| {}));
        console.open();
        console.println("Enter entity name (e.g., Book)");

        self.console = console;
        self.status = "Entity creation wizard started".into();
        Ok(())
    }

    fn entity_action_regen(&mut self) -> Result<()> {
        let pref = self
            .current_project
            .clone()
            .ok_or_else(|| anyhow!("No project loaded"))?;
        let hist = self
            .current_project_data
            .as_ref()
            .ok_or_else(|| anyhow!("No project history loaded"))?;
        if hist.entities.is_empty() {
            self.status = "No entities".into();
            return Ok(());
        }
        let root = PathBuf::from(&pref.project_dir);
        let ent = hist
            .entities
            .get(self.list_index)
            .ok_or_else(|| anyhow!("Invalid entity selection"))?;
        let (domain, namespace, name, fields, ui_target, mobile_ui) = (
            ent.domain.clone(),
            ent.namespace.clone(),
            ent.name.clone(),
            ent.fields.clone(),
            ent.ui_target,
            hist.mobile_ui,
        );

        let action = JobAction::UpdateGeneratedAt {
            root: root.clone(),
            ent_idx: self.list_index,
        };
        self.start_generate_job(
            GenerationTask {
                root,
                domain,
                namespace,
                entity: name,
                fields,
                ui_target,
                mobile_ui,
                run_migration: false,
            },
            action,
        );
        Ok(())
    }

    fn entity_action_edit_fields_and_regen(
        &mut self,
        _term: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let hist = match self.current_project_data.as_ref() {
            Some(h) => h,
            None => {
                self.status = "No project loaded".into();
                return Ok(());
            }
        };
        if hist.entities.is_empty() {
            self.status = "No entities".into();
            return Ok(());
        }

        let ent_idx = self.list_index;
        let Some(ent) = hist.entities.get(ent_idx) else {
            self.status = "Invalid entity selection".into();
            return Ok(());
        };

        let rows = ent
            .fields
            .iter()
            .map(|f| {
                let (nav_ns, nav_name, nav_display) = if f.ftype.eq_ignore_ascii_case("Guid")
                    || f.ftype.eq_ignore_ascii_case("enum")
                {
                    if let Some(full) = &f.navigation {
                        let mut disp = f.navigation_display.clone();
                        if disp.is_none() {
                            let (_, _, d) = split_nav(full);
                            disp = d;
                        }
                        let (ns, name, d) = split_nav(full);
                        (Some(ns), name, disp.or(d))
                    } else {
                        (None, None, None)
                    }
                } else {
                    (None, None, None)
                };
                FieldRow {
                    name: f.name.clone(),
                    ftype: f.ftype.clone(),
                    required: f.required,
                    max_len: f.max_length,
                    nav_name,
                    nav_ns,
                    nav_display,
                    filterable: f.filterable,
                    show_ui: f.show_in_ui,
                }
            })
            .collect::<Vec<_>>();

        self.modal = Modal::EditFields {
            ent_idx,
            rows,
            row: 0,
            col: FieldCol::Name,
            dirty: false,
            dragging: false,
        };
        self.status = format!("Editing fields of '{}'", ent.name);
        Ok(())
    }

    fn entity_action_edit_meta_and_regen(
        &mut self,
        _term: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let hist = match self.current_project_data.as_ref() {
            Some(h) => h,
            None => {
                self.status = "No project loaded".into();
                return Ok(());
            }
        };
        if hist.entities.is_empty() {
            self.status = "No entities".into();
            return Ok(());
        }
        let ent_idx = self.list_index;
        let Some(ent) = hist.entities.get(ent_idx) else {
            self.status = "Invalid entity selection".into();
            return Ok(());
        };

        self.modal = Modal::EditMeta {
            target: MetaTarget::Entity(ent_idx),
            rows: vec![
                ("Namespace".to_string(), ent.namespace.clone()),
                (
                    "UI".to_string(),
                    match ent.ui_target {
                        UiTarget::None => "None".to_string(),
                        UiTarget::Razor => "Razor".to_string(),
                        UiTarget::Vue => "Vue".to_string(),
                        UiTarget::Angular => "Angular".to_string(),
                    },
                ),
            ],
            row: 0,
            col: 1,
        };
        self.status = "Editing meta (Namespace/UI)".into();

        Ok(())
    }

    fn project_action_edit_meta(&mut self) -> Result<()> {
        let hist = match self.current_project_data.as_ref() {
            Some(h) => h,
            None => {
                self.status = "No project loaded".into();
                return Ok(());
            }
        };
        self.open_project_theme_modal(hist.theme);
        Ok(())
    }

    fn project_action_edit_meta_from_list(&mut self) -> Result<()> {
        if self.global.projects.is_empty() {
            self.status = "No projects".into();
            return Ok(());
        }
        let Some(pref) = self.global.projects.get(self.list_index).cloned() else {
            self.list_index = self.global.projects.len().saturating_sub(1);
            self.status = "Invalid project selection".into();
            return Ok(());
        };
        let root = PathBuf::from(&pref.project_dir);
        let hist = if self.cli.dry_run {
            project_record_for(&root, &pref)?
        } else {
            reconcile_project_record(&root, &pref)?
        };

        self.current_project = Some(pref);
        self.current_project_data = Some(hist);

        if let Some(h) = self.current_project_data.as_ref() {
            self.open_project_theme_modal(h.theme);
        }
        Ok(())
    }

    fn open_project_theme_modal(&mut self, theme: AbpTheme) {
        self.modal = Modal::EditMeta {
            target: MetaTarget::Project,
            rows: vec![
                ("Theme".to_string(), theme_label(theme).to_string()),
                (
                    "Bootstrap Override".to_string(),
                    override_label(
                        self.current_project_data
                            .as_ref()
                            .map(|p| p.bootstrap_override)
                            .unwrap_or_default(),
                    )
                    .to_string(),
                ),
                (
                    "Mobile UI".to_string(),
                    mobile_ui_label(
                        self.current_project_data
                            .as_ref()
                            .map(|p| p.mobile_ui)
                            .unwrap_or_default(),
                    )
                    .to_string(),
                ),
            ],
            row: 0,
            col: 1,
        };
        self.status = "Editing project meta (Theme / Bootstrap override / Mobile UI)".into();
    }

    fn delete_selected_project(&mut self) -> Result<()> {
        if self.global.projects.is_empty() {
            return Ok(());
        }
        let idx = self.list_index;
        if idx >= self.global.projects.len() {
            self.list_index = self.global.projects.len().saturating_sub(1);
            self.status = "Invalid project selection".into();
            return Ok(());
        }
        let victim = self.global.projects.remove(idx);
        save_global_index(&self.global)?;
        self.list_index = self.list_index.saturating_sub(1);
        self.status = format!(
            "Removed from global index: {} ({})",
            victim.project_name, victim.project_dir
        );
        Ok(())
    }

    fn start_rename_project_inline(&mut self) -> Result<()> {
        if self.global.projects.is_empty() {
            self.status = "No projects to rename".into();
            return Ok(());
        }
        let Some(current) = self
            .global
            .projects
            .get(self.list_index)
            .map(|project| project.project_name.clone())
        else {
            self.list_index = self.global.projects.len().saturating_sub(1);
            self.status = "Invalid project selection".into();
            return Ok(());
        };
        let mut console = InlineConsole::new("Rename Project");
        let cur_name = current.clone();
        let workflow = self.workflow.clone();

        console.on_submit = Some(Box::new(move |line: String| {
            let newname = {
                let t = line.trim();
                if t.is_empty() {
                    cur_name.clone()
                } else {
                    t.to_string()
                }
            };
            workflow.complete_project_rename(newname);
            None
        }));

        console.on_cancel = Some(Box::new(|| {}));
        console.open();
        console.println(format!("New project name (current: {current})"));
        console.input = current.clone();
        self.console = console;
        self.status = "Rename dialog opened".into();
        Ok(())
    }
}

fn save_current_edit_fields(app: &mut App) -> Result<()> {
    let (ent_idx, rows) = match &app.modal {
        Modal::EditFields { ent_idx, rows, .. } => (*ent_idx, rows.clone()),
        _ => return Ok(()),
    };

    let Some(project_dir) = app
        .current_project
        .as_ref()
        .map(|project| project.project_dir.clone())
    else {
        app.status = "No project loaded".into();
        app.modal = Modal::None;
        return Ok(());
    };

    let (root, new_fields, ent_data) = if let Some(hist) = app.current_project_data.as_mut() {
        if ent_idx >= hist.entities.len() {
            app.status = "Invalid entity index".into();
            app.modal = Modal::None;
            return Ok(());
        }

        let root = PathBuf::from(project_dir);
        let norm = |t: &str| match t.to_ascii_lowercase().as_str() {
            "guid" => "Guid".to_string(),
            "datetime" => "DateTime".to_string(),
            "string" => "string".to_string(),
            "textarea" => "textarea".to_string(),
            "decimal" => "decimal".to_string(),
            "int" => "int".to_string(),
            "long" => "long".to_string(),
            "bool" => "bool".to_string(),
            "dateonly" => "DateOnly".to_string(),
            "timeonly" => "TimeOnly".to_string(),
            "enum" => "enum".to_string(),
            other => other.to_string(),
        };
        let new_fields: Vec<Field> = rows
            .iter()
            .map(|r| {
                let navigation = if r.ftype == "Guid" || r.ftype == "enum" {
                    match (&r.nav_ns, &r.nav_name) {
                        (Some(ns), Some(name)) if !ns.is_empty() && !name.is_empty() => {
                            let mut base = format!("{}.{}", ns.trim_end_matches('.'), name);
                            if r.ftype == "Guid"
                                && let Some(disp) = r.nav_display.as_ref().filter(|s| !s.is_empty())
                            {
                                base.push('#');
                                base.push_str(disp);
                            }
                            Some(base)
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                let nav_display = if r.ftype == "Guid" {
                    r.nav_display.clone().filter(|s| !s.is_empty())
                } else {
                    None
                };
                Field {
                    name: r.name.clone(),
                    ftype: norm(&r.ftype),
                    required: r.required,
                    max_length: r.max_len,
                    navigation_display: nav_display,
                    navigation,
                    filterable: r.filterable,
                    show_in_ui: r.show_ui,
                }
            })
            .collect();

        let ent_data = {
            let ent = &mut hist.entities[ent_idx];
            ent.fields = new_fields.clone();
            (
                ent.domain.clone(),
                ent.namespace.clone(),
                ent.name.clone(),
                ent.ui_target,
            )
        };
        (root, new_fields, ent_data)
    } else {
        return Ok(());
    };

    let (domain, namespace, entity, ui_target) = ent_data;
    let mobile_ui = app
        .current_project_data
        .as_ref()
        .map(|h| h.mobile_ui)
        .unwrap_or_default();
    let action = JobAction::UpdateGeneratedAt {
        root: root.clone(),
        ent_idx,
    };
    app.start_generate_job(
        GenerationTask {
            root,
            domain,
            namespace,
            entity,
            fields: new_fields,
            ui_target,
            mobile_ui,
            run_migration: false,
        },
        action,
    );
    app.status = "Regenerating with edited fields...".into();
    app.modal = Modal::None;
    Ok(())
}

fn save_current_edit_meta(app: &mut App) -> Result<()> {
    let (target, rows) = match &app.modal {
        Modal::EditMeta { target, rows, .. } => (*target, rows.clone()),
        _ => return Ok(()),
    };

    match target {
        MetaTarget::Entity(ent_idx) => {
            let Some(project_dir) = app
                .current_project
                .as_ref()
                .map(|project| project.project_dir.clone())
            else {
                app.status = "No project loaded".into();
                app.modal = Modal::None;
                return Ok(());
            };
            let (root, domain, entity, fields, namespace, ui_target, ent_idx_val) =
                if let Some(hist) = app.current_project_data.as_mut() {
                    if ent_idx < hist.entities.len() {
                        let root = PathBuf::from(project_dir);

                        let new_namespace = rows
                            .iter()
                            .find(|p| p.0 == "Namespace")
                            .map(|p| p.1.clone())
                            .unwrap_or_else(|| hist.entities[ent_idx].namespace.clone());

                        let raw_ui = rows
                            .iter()
                            .find(|p| p.0 == "UI")
                            .map(|p| p.1.clone())
                            .unwrap_or_else(|| "None".into());

                        let (has_mvc, has_ng) =
                            generator::detect_ui(&root, &hist.entities[ent_idx].domain);
                        let sanitized = sanitize_ui_choice(&raw_ui, has_mvc, has_ng);

                        let data = {
                            let ent = &mut hist.entities[ent_idx];
                            ent.namespace = new_namespace;
                            ent.ui_target = sanitized;
                            (
                                ent.domain.clone(),
                                ent.name.clone(),
                                ent.fields.clone(),
                                ent.namespace.clone(),
                                ent.ui_target,
                                ent_idx,
                            )
                        };
                        (root, data.0, data.1, data.2, data.3, data.4, data.5)
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                };

            let mobile_ui = app
                .current_project_data
                .as_ref()
                .map(|h| h.mobile_ui)
                .unwrap_or_default();

            let action = JobAction::UpdateGeneratedAt {
                root: root.clone(),
                ent_idx: ent_idx_val,
            };
            app.start_generate_job(
                GenerationTask {
                    root,
                    domain,
                    namespace,
                    entity,
                    fields,
                    ui_target,
                    mobile_ui,
                    run_migration: false,
                },
                action,
            );
            app.status = "Regenerating with edited meta...".into();
        }
        MetaTarget::Project => {
            let pref = match app.current_project.as_ref() {
                Some(p) => p,
                None => {
                    app.status = "No project loaded".into();
                    app.modal = Modal::None;
                    return Ok(());
                }
            };

            if let Some(hist) = app.current_project_data.as_mut() {
                let raw_theme = rows
                    .iter()
                    .find(|p| p.0 == "Theme")
                    .map(|p| p.1.clone())
                    .unwrap_or_else(|| theme_label(hist.theme).to_string());
                let new_theme = sanitize_theme_choice(&raw_theme);

                let raw_override = rows
                    .iter()
                    .find(|p| p.0 == "Bootstrap Override")
                    .map(|p| p.1.clone())
                    .unwrap_or_else(|| override_label(hist.bootstrap_override).to_string());
                let new_override = sanitize_override_choice(&raw_override);

                let raw_mobile = rows
                    .iter()
                    .find(|p| p.0 == "Mobile UI")
                    .map(|p| p.1.clone())
                    .unwrap_or_else(|| mobile_ui_label(hist.mobile_ui).to_string());
                let new_mobile = sanitize_mobile_ui_choice(&raw_mobile);

                let theme_changed = hist.theme != new_theme;
                let override_changed = hist.bootstrap_override != new_override;
                let mobile_changed = hist.mobile_ui != new_mobile;

                if !theme_changed && !override_changed && !mobile_changed {
                    app.status = "Project meta unchanged".into();
                } else {
                    let root = PathBuf::from(&pref.project_dir);
                    app.start_project_meta_job(
                        root,
                        new_theme,
                        new_override,
                        new_mobile,
                        theme_changed,
                        mobile_changed,
                    );
                }
            }
        }
    }

    app.modal = Modal::None;
    Ok(())
}

/* ------------------------------ TUI LOOP ------------------------------- */

pub fn run_tui(cli: &Cli) -> Result<()> {
    VERBOSE.store(false, Ordering::Relaxed);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cterm::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = tui_loop(cli, &mut terminal);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), cterm::LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    res
}

fn perform_pending_action(
    app: &mut App,
    _terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    action: PendingAction,
) -> Result<()> {
    app.vim_pending_g = false;
    app.help_open = false;
    match action {
        PendingAction::ExitProgram => Err(anyhow!("__EXIT__")),
        PendingAction::DeleteProject => app.delete_selected_project(),
        PendingAction::OpenProjectBack => {
            app.screen = Screen::Projects;
            app.current_project = None;
            app.current_project_data = None;
            app.list_index = 0;
            app.status = "Back to projects".into();
            Ok(())
        }
        PendingAction::DeleteEntity => app.entity_action_delete_from_history(),
        PendingAction::RegenEntity => app.entity_action_regen(),
        PendingAction::CancelEditFields => {
            app.modal = Modal::None;
            app.status = "Cancelled".into();
            Ok(())
        }
        PendingAction::SaveEditFields => save_current_edit_fields(app),
        PendingAction::CancelEditMeta => {
            app.modal = Modal::None;
            app.status = "Cancelled".into();
            Ok(())
        }
        PendingAction::SaveEditMeta => save_current_edit_meta(app),
        PendingAction::CancelCreateProjectWizard
        | PendingAction::CancelCreateEntityWizard
        | PendingAction::CancelInlineConsole => {
            if let Some(mut f) = app.console.on_cancel.take() {
                f();
            }
            app.console.close();
            app.vim_pending_g = false;
            app.help_open = false;
            app.status = "Cancelled".into();
            Ok(())
        }
    }
}

fn split_nav(nav: &str) -> (String, Option<String>, Option<String>) {
    let (base, disp) = nav
        .rsplit_once('#')
        .map(|(b, d)| (b.to_string(), Some(d.to_string())))
        .unwrap_or((nav.to_string(), None));
    let nav_name = last_segment(&base);
    let nav_ns = ns_of(&base);
    (nav_ns, Some(nav_name.to_string()), disp)
}

fn submit_console_input(app: &mut App, input: String) {
    app.console.println(format!("> {}", input));
    if let Some(cb) = app.console.on_submit.as_mut() {
        if let Some(next_prompt) = cb(input) {
            app.console.input.clear();
            app.console.println(next_prompt.text);
            app.console.input_hint = next_prompt.hint;
        } else {
            app.console.close();
            app.vim_pending_g = false;
            app.help_open = false;
            app.status = "Done".into();
        }
    }
}

fn try_quick_submit(app: &mut App, c: char) -> bool {
    match &app.console.input_hint {
        InputHint::YesNo => {
            if matches!(c, 'y' | 'Y' | 'n' | 'N') {
                submit_console_input(app, c.to_string());
                return true;
            }
        }
        InputHint::DigitChoice(opts) => {
            if opts.contains(&c) {
                submit_console_input(app, c.to_string());
                return true;
            }
        }
        InputHint::Free => {}
    }
    false
}

fn apply_inline_meta_edit(app: &mut App<'_>, value: String) {
    let Modal::EditMeta {
        rows, row, target, ..
    } = &mut app.modal
    else {
        return;
    };
    let key_name = rows.get(*row).map(|(key, _)| key.as_str());
    match (*target, key_name) {
        (MetaTarget::Entity(entity_index), Some("UI")) => {
            let detected_ui = app
                .current_project
                .as_ref()
                .zip(app.current_project_data.as_ref())
                .and_then(|(project, history)| {
                    history.entities.get(entity_index).map(|entity| {
                        generator::detect_ui(&PathBuf::from(&project.project_dir), &entity.domain)
                    })
                })
                .unwrap_or((false, false));
            let target = sanitize_ui_choice(&value, detected_ui.0, detected_ui.1);
            rows[*row].1 = match target {
                UiTarget::Razor => "Razor".into(),
                UiTarget::Vue => "Vue".into(),
                UiTarget::Angular => "Angular".into(),
                UiTarget::None => "None".into(),
            };
            app.status = if detected_ui == (false, false) {
                "There is no MVC/Angular UI; UI = None".into()
            } else {
                format!("UI: {}", rows[*row].1)
            };
        }
        (MetaTarget::Project, Some("Theme")) => {
            let theme = sanitize_theme_choice(&value);
            rows[*row].1 = theme_label(theme).to_string();
            app.status = format!("Theme: {}", theme_label(theme));
        }
        (MetaTarget::Project, Some("Bootstrap Override")) => {
            let override_css = sanitize_override_choice(&value);
            rows[*row].1 = override_label(override_css).to_string();
            app.status = format!("Bootstrap override: {}", override_label(override_css));
        }
        _ => rows[*row].1 = value,
    }
}

fn tui_loop(cli: &Cli, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app = App::new(cli);

    loop {
        app.poll_job();
        terminal.draw(|f| {
            match app.screen {
                Screen::Projects => draw_projects_ui(f, &app),
                Screen::ProjectDetail => draw_project_detail_ui(f, &app),
            }
            draw_list_modal(f, &app);
            draw_inline_console(f, &mut app);
            draw_help_modal(f, &app);
            draw_confirm_modal(f, &app);
        })?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(k) = event::read()?
        {
            if app.confirm.is_some() {
                if handle_confirm_key(&mut app, terminal, k.code)? {
                    return Ok(());
                }
                continue;
            }
            if app.help_open {
                handle_help_key(&mut app, k.code);
                continue;
            }
            if app.console.visible {
                handle_console_key(&mut app, k);
                continue;
            }
            if k.code == KeyCode::Char('?') && app.confirm.is_none() {
                app.help_open = true;
                app.vim_pending_g = false;
                continue;
            }
            if handle_modal_key(&mut app, k) {
                continue;
            }
            handle_screen_key(&mut app, terminal, k);
        }

        if !app.console.visible {
            if let Some((project_dir, project_name)) = app.workflow.take_created_project() {
                let pref = ProjectRef {
                    project_name: project_name.clone(),
                    project_dir: project_dir.display().to_string(),
                };
                if !app
                    .global
                    .projects
                    .iter()
                    .any(|p| p.project_dir == pref.project_dir)
                {
                    app.global.projects.push(pref.clone());
                    let _ = save_global_index(&app.global);
                }
                let root = PathBuf::from(&pref.project_dir);
                let record = if app.cli.dry_run {
                    project_record_for(&root, &pref)
                } else {
                    reconcile_project_record(&root, &pref)
                };
                match record {
                    Ok(record) => {
                        app.status = format!(
                            "Project created: {} ({}) [{}]",
                            project_name,
                            project_dir.display(),
                            theme_label(record.theme)
                        );
                    }
                    Err(error) => {
                        app.status = format!("Error detecting project metadata: {error}");
                    }
                }
            }

            if let Some(newname) = app.workflow.take_project_rename() {
                if app.list_index >= app.global.projects.len() {
                    app.status = "Rename failed: selection out of bounds".into();
                } else {
                    let (oldname, project_dir) = {
                        let pref = &mut app.global.projects[app.list_index];
                        let old = pref.project_name.clone();
                        let dir = pref.project_dir.clone();
                        pref.project_name = newname.clone();
                        (old, dir)
                    };
                    if let Err(e) = save_global_index(&app.global) {
                        app.status = format!("Error saving global index: {e}");
                    } else {
                        let root = PathBuf::from(&project_dir);
                        match load_project_history(&root) {
                            Ok(Some(mut rec)) => {
                                rec.project_name = newname.clone();
                                if let Err(e) = save_project_history(&root, &rec) {
                                    app.status = format!("Error saving project history: {e}");
                                } else {
                                    app.status = format!(
                                        "Project renamed: '{old}' → '{new}'",
                                        old = oldname,
                                        new = newname
                                    );
                                }
                            }
                            Ok(None) => {
                                app.status = format!(
                                    "Project renamed: '{old}' → '{new}' (no history)",
                                    old = oldname,
                                    new = newname
                                );
                            }
                            Err(error) => {
                                app.status = format!("Error loading project history: {error}");
                            }
                        }
                    }
                }
            }

            if let Some(pending) = app.workflow.take_generation() {
                let mobile_ui = app
                    .current_project_data
                    .as_ref()
                    .map(|h| h.mobile_ui)
                    .unwrap_or_default();
                let record = EntityRecord {
                    domain: pending.domain.clone(),
                    namespace: pending.namespace.clone(),
                    name: pending.entity.clone(),
                    fields: pending.fields.clone(),
                    ui_target: pending.ui_target,
                    generated_at: Local::now(),
                };
                let action = JobAction::AddNew {
                    root: pending.root.clone(),
                    record,
                };
                app.start_generate_job(
                    GenerationTask {
                        root: pending.root,
                        domain: pending.domain,
                        namespace: pending.namespace,
                        entity: pending.entity,
                        fields: pending.fields,
                        ui_target: pending.ui_target,
                        mobile_ui,
                        run_migration: pending.run_migration,
                    },
                    action,
                );
            }

            if let Some(value) = app.workflow.take_meta_edit() {
                apply_inline_meta_edit(&mut app, value);
            }

            if let Some((row_idx, which_col, line)) = app.workflow.take_field_edit()
                && let Modal::EditFields { rows, .. } = &mut app.modal
            {
                let v = line.trim();
                if row_idx < rows.len() {
                    match which_col {
                        FieldCol::Name => {
                            if !v.is_empty() {
                                rows[row_idx].name = v.to_string();
                            }
                        }
                        FieldCol::Type => {
                            let t = v.to_lowercase();
                            if matches!(
                                t.as_str(),
                                "string"
                                    | "textarea"
                                    | "int"
                                    | "long"
                                    | "decimal"
                                    | "guid"
                                    | "bool"
                                    | "datetime"
                            ) {
                                rows[row_idx].ftype = match t.as_str() {
                                    "guid" => "Guid".into(),
                                    "datetime" => "DateTime".into(),
                                    _ => t,
                                };
                            } else {
                                app.status = "Invalid type; use string|textarea|int|long|decimal|Guid|bool|DateTime".into();
                            }
                        }
                        FieldCol::Required => {
                            if !v.is_empty() {
                                rows[row_idx].required =
                                    matches!(v.to_lowercase().as_str(), "y" | "yes" | "true" | "1");
                            }
                        }
                        FieldCol::MaxLen => {
                            if v.is_empty() {
                                rows[row_idx].max_len = None;
                            } else if let Ok(n) = v.parse::<i32>() {
                                rows[row_idx].max_len = Some(n);
                            } else {
                                app.status = "MaxLen must be an integer".into();
                            }
                        }
                        FieldCol::NavName => {
                            if v.is_empty() {
                                rows[row_idx].nav_name = None;
                                rows[row_idx].nav_display = None;
                                rows[row_idx].nav_ns = None;
                            } else {
                                rows[row_idx].nav_name = Some(v.to_string());
                            }
                        }
                        FieldCol::NavDisplay => {
                            if rows[row_idx].ftype == "Guid" {
                                if v.is_empty() {
                                    rows[row_idx].nav_display = None;
                                } else {
                                    rows[row_idx].nav_display = Some(v.to_string());
                                }
                            }
                        }
                        FieldCol::NavNs => {
                            if v.is_empty() {
                                rows[row_idx].nav_ns = None;
                                rows[row_idx].nav_display = None;
                                rows[row_idx].nav_name = None;
                            } else {
                                rows[row_idx].nav_ns = Some(v.to_string());
                            }
                        }
                        FieldCol::Filterable | FieldCol::ShowUi => {
                            let v = line.trim().to_ascii_lowercase();
                            let b = matches!(v.as_str(), "y" | "yes" | "true" | "1");
                            match which_col {
                                FieldCol::Filterable => rows[row_idx].filterable = b,
                                FieldCol::ShowUi => rows[row_idx].show_ui = b,
                                _ => {}
                            }
                        }
                    }
                    app.status = "Cell updated".into();
                }
            }
        }

        app.relist_bound();
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use std::fs;

    #[test]
    fn reconciles_a_stale_basic_history_when_an_angular_project_is_registered() {
        let fixture = tempfile::tempdir().expect("fixture should exist");
        fs::create_dir_all(fixture.path().join("src/Acme.Demo.Domain"))
            .expect("domain should exist");
        fs::create_dir_all(fixture.path().join("angular/src/app"))
            .expect("Angular app should exist");
        fs::write(
            fixture.path().join("angular/package.json"),
            r#"{"dependencies":{"@abp/ng.theme.lepton-x":"latest"}}"#,
        )
        .expect("Angular package manifest should exist");

        let project = ProjectRef {
            project_name: "Demo".into(),
            project_dir: fixture.path().display().to_string(),
        };
        save_project_history(
            fixture.path(),
            &ProjectRecord {
                project_name: project.project_name.clone(),
                project_dir: project.project_dir.clone(),
                theme: AbpTheme::Basic,
                bootstrap_override: BootstrapOverride::None,
                mobile_ui: MobileUi::None,
                entities: Vec::new(),
            },
        )
        .expect("stale history should save");

        let record = reconcile_project_record(fixture.path(), &project)
            .expect("project metadata should reconcile");
        assert_eq!(record.theme, AbpTheme::LeptonX);
        assert_eq!(
            load_project_history(fixture.path())
                .expect("history should load")
                .expect("history should exist")
                .theme,
            AbpTheme::LeptonX
        );
    }
}

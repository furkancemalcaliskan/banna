use super::*;

pub(super) fn handle_screen_key(
    app: &mut App<'_>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    k: crossterm::event::KeyEvent,
) {
    match app.screen {
        Screen::Projects => {
            if app.project_dragging {
                match k.code {
                    KeyCode::Char(' ') => app.finish_project_drag(),
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                        app.move_project(-1);
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        app.move_project(1);
                    }
                    KeyCode::Esc => {
                        app.project_dragging = false;
                        app.project_drag_dirty = false;
                        app.status = "Drag cancelled".into();
                    }
                    _ => {}
                }
                return;
            }

            match k.code {
                KeyCode::Char('q') => {
                    app.confirm("Exit the program?", PendingAction::ExitProgram);
                }
                KeyCode::Char('j') => {
                    let len = app.global.projects.len();
                    if len > 0 {
                        wrap_next(&mut app.list_index, len);
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('k') => {
                    let len = app.global.projects.len();
                    if len > 0 {
                        wrap_prev(&mut app.list_index, len);
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('g') => {
                    if app.vim_pending_g {
                        vim_go_top(&mut app.list_index);
                        app.vim_pending_g = false;
                    } else {
                        app.vim_pending_g = true;
                    }
                }
                KeyCode::Char('G') => {
                    vim_go_bottom(&mut app.list_index, app.global.projects.len());
                    app.vim_pending_g = false;
                }
                KeyCode::Down => {
                    let len = app.global.projects.len();
                    if len > 0 {
                        wrap_next(&mut app.list_index, len);
                    }
                }
                KeyCode::Up => {
                    let len = app.global.projects.len();
                    if len > 0 {
                        wrap_prev(&mut app.list_index, len);
                    }
                }
                KeyCode::Enter => {
                    app.open_selected_project();
                }
                KeyCode::Char('n') => {
                    if let Err(e) = app.start_create_project_inline() {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Char('E') => {
                    if let Err(e) = app.project_action_edit_meta_from_list() {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Delete => {
                    app.confirm(
                        "Remove this project from the global index? (Files are not deleted.)",
                        PendingAction::DeleteProject,
                    );
                }
                KeyCode::Char('r') => {
                    if let Err(e) = app.start_rename_project_inline() {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Char(' ') if !app.global.projects.is_empty() => {
                    app.project_dragging = true;
                    app.project_drag_dirty = false;
                    app.status = "Drag mode: Space=drop, ↑/↓ or J/K to move, Esc=cancel".into();
                }
                _ => {}
            }
        }
        Screen::ProjectDetail => {
            if app.entity_dragging {
                match k.code {
                    KeyCode::Char(' ') => app.finish_entity_drag(),
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                        app.move_entity(-1);
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        app.move_entity(1);
                    }
                    KeyCode::Esc => {
                        app.entity_dragging = false;
                        app.entity_drag_dirty = false;
                        app.status = "Drag cancelled".into();
                    }
                    _ => {}
                }
                return;
            }

            match k.code {
                KeyCode::Char('q') => {
                    app.confirm(
                        "Leave this project and go back to the projects screen?",
                        PendingAction::OpenProjectBack,
                    );
                }
                KeyCode::Char('j') => {
                    if let Some(hist) = app.current_project_data.as_ref() {
                        let len = hist.entities.len();
                        if len > 0 {
                            wrap_next(&mut app.list_index, len);
                        }
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('k') => {
                    if let Some(hist) = app.current_project_data.as_ref() {
                        let len = hist.entities.len();
                        if len > 0 {
                            wrap_prev(&mut app.list_index, len);
                        }
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('g') => {
                    if app.vim_pending_g {
                        app.list_index = 0;
                        app.vim_pending_g = false;
                    } else {
                        app.vim_pending_g = true;
                    }
                }
                KeyCode::Char('G') => {
                    if let Some(hist) = app.current_project_data.as_ref() {
                        vim_go_bottom(&mut app.list_index, hist.entities.len());
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Down => {
                    if let Some(hist) = app.current_project_data.as_ref() {
                        let len = hist.entities.len();
                        if len > 0 {
                            wrap_next(&mut app.list_index, len);
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(hist) = app.current_project_data.as_ref() {
                        let len = hist.entities.len();
                        if len > 0 {
                            wrap_prev(&mut app.list_index, len);
                        }
                    }
                }
                KeyCode::Char('n') => {
                    if let Err(e) = app.create_new_entity(terminal) {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Char('t') => {
                    if let Err(e) = app.project_action_edit_meta() {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Enter => {
                    app.confirm(
                        "Regenerate this entity into the codebase?",
                        PendingAction::RegenEntity,
                    );
                }
                KeyCode::Char('e') => {
                    if let Err(e) = app.entity_action_edit_fields_and_regen(terminal) {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Char('E') => {
                    if let Err(e) = app.entity_action_edit_meta_and_regen(terminal) {
                        app.status = format!("Error: {e}");
                    }
                }
                KeyCode::Char('d') => {
                    app.confirm(
                        "Delete this entity from history (files are not removed)?",
                        PendingAction::DeleteEntity,
                    );
                }
                KeyCode::Char(' ') => {
                    if let Some(hist) = app.current_project_data.as_ref()
                        && !hist.entities.is_empty()
                    {
                        app.entity_dragging = true;
                        app.entity_drag_dirty = false;
                        app.status = "Drag mode: Space=drop, ↑/↓ or J/K to move, Esc=cancel".into();
                    }
                }
                _ => {}
            }
        }
    }
}

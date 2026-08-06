use super::*;

/// Handles one key while the confirmation overlay owns input.
/// Returns `true` when the application should exit.
pub(super) fn handle_confirm_key(
    app: &mut App<'_>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    key: KeyCode,
) -> Result<bool> {
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => toggle_confirmation(app),
        KeyCode::Char('h') | KeyCode::Char('l') => {
            toggle_confirmation(app);
            app.vim_pending_g = false;
        }
        KeyCode::Char('g') | KeyCode::Char('G') | KeyCode::Char('j') | KeyCode::Char('k') => {
            app.vim_pending_g = false;
        }
        KeyCode::Enter => {
            let selected = app
                .confirm
                .as_ref()
                .is_some_and(|confirm| confirm.ok_selected);
            if selected {
                let Some(action) = app.confirm.as_ref().map(|confirm| confirm.action.clone())
                else {
                    app.status = "Confirmation state expired".into();
                    return Ok(false);
                };
                app.clear_confirm();
                if let Err(error) = perform_pending_action(app, terminal, action) {
                    if error.to_string() == "__EXIT__" {
                        return Ok(true);
                    }
                    app.status = format!("Error: {error}");
                }
            } else {
                cancel_confirmation(app);
            }
        }
        KeyCode::Esc => cancel_confirmation(app),
        _ => {}
    }
    Ok(false)
}

fn toggle_confirmation(app: &mut App<'_>) {
    if let Some(confirm) = app.confirm.as_mut() {
        confirm.ok_selected = !confirm.ok_selected;
    }
}

fn cancel_confirmation(app: &mut App<'_>) {
    app.clear_confirm();
    app.status = "Cancelled".into();
}

pub(super) fn handle_help_key(app: &mut App<'_>, key: KeyCode) {
    if matches!(key, KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Esc) {
        app.help_open = false;
        app.vim_pending_g = false;
    }
}

pub(super) fn handle_console_key(app: &mut App<'_>, key: crossterm::event::KeyEvent) {
    if app.console.mode == ConsoleMode::Logs {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
            app.console.close();
            app.vim_pending_g = false;
            app.help_open = false;
            app.status = "Logs closed".into();
        } else {
            handle_console_scroll(app, key);
        }
        return;
    }

    match key.code {
        KeyCode::Esc => {
            let action = if app.console.title.contains("Create Project") {
                PendingAction::CancelCreateProjectWizard
            } else if app.console.title.contains("Create Entity") {
                PendingAction::CancelCreateEntityWizard
            } else {
                PendingAction::CancelInlineConsole
            };
            app.confirm("Cancel and close this dialog?", action);
        }
        KeyCode::Enter => {
            let line = std::mem::take(&mut app.console.input);
            submit_console_input(app, line);
        }
        KeyCode::Backspace => {
            app.console.input.pop();
        }
        KeyCode::Tab => app.console.input.push('\t'),
        KeyCode::Char(character) if !is_scroll_shortcut(key) => {
            if !try_quick_submit(app, character) {
                app.console.input.push(character);
            }
        }
        _ => {
            handle_console_scroll(app, key);
        }
    }
}

fn is_scroll_shortcut(key: crossterm::event::KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('f') | KeyCode::Char('b'))
        && key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
}

fn handle_console_scroll(app: &mut App<'_>, key: crossterm::event::KeyEvent) -> bool {
    let page = app.console.view_height.max(1);
    let maximum = app.console.max_offset();
    match key.code {
        KeyCode::Char('f') if is_scroll_shortcut(key) => {
            app.console.scroll = app.console.scroll.saturating_add(page).min(maximum);
            app.console.auto_scroll = app.console.scroll == maximum;
            app.vim_pending_g = false;
        }
        KeyCode::Char('b') if is_scroll_shortcut(key) => {
            app.console.scroll = app.console.scroll.saturating_sub(page);
            app.console.auto_scroll = false;
            app.vim_pending_g = false;
        }
        KeyCode::Up => {
            if app.console.scroll > 0 {
                app.console.scroll = app.console.scroll.saturating_sub(1);
                app.console.auto_scroll = false;
            }
        }
        KeyCode::Down => {
            app.console.scroll = app.console.scroll.saturating_add(1).min(maximum);
            app.console.auto_scroll = app.console.scroll == maximum;
        }
        KeyCode::PageUp => {
            app.console.scroll = app.console.scroll.saturating_sub(page);
            app.console.auto_scroll = false;
        }
        KeyCode::PageDown => {
            app.console.scroll = app.console.scroll.saturating_add(page).min(maximum);
            app.console.auto_scroll = app.console.scroll == maximum;
        }
        KeyCode::Home => {
            app.console.scroll = 0;
            app.console.auto_scroll = false;
        }
        KeyCode::End => {
            app.console.scroll = maximum;
            app.console.auto_scroll = true;
        }
        _ => return false,
    }
    true
}

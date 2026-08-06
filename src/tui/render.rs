use super::*;

pub(super) fn draw_inline_console(frame: &mut ratatui::Frame, app: &mut App) {
    if !app.console.visible {
        return;
    }
    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);
    let base = block(&app.console.title).style(Style::default().bg(THEME.surface));
    frame.render_widget(base.clone(), area);
    let inner = base.inner(area);
    let show_input = matches!(app.console.mode, ConsoleMode::Wizard) || app.console.input_visible;
    let chunks = if show_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(1)])
            .split(inner)
    };
    app.console.view_height = chunks[0].height.saturating_sub(2);
    let maximum_offset = app.console.max_offset();
    if app.console.auto_scroll || app.console.scroll > maximum_offset {
        app.console.scroll = maximum_offset;
    }
    let text = app
        .console
        .lines
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Text::from(text))
            .block(block_muted("Logs"))
            .wrap(Wrap { trim: false })
            .scroll((app.console.scroll, 0)),
        chunks[0],
    );
    let hint_message = match app.console.mode {
        ConsoleMode::Wizard => "(Enter=submit, q=cancel/close, ↑/↓, PgUp/PgDn=Home/End to scroll)",
        ConsoleMode::Logs => "(q/Esc/Enter to close, ↑/↓, PgUp/PgDn/Home/End to scroll)",
    };
    frame.render_widget(
        Paragraph::new(hint_message).style(faint()).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(THEME.border)),
        ),
        chunks[1],
    );
    if show_input {
        let input_text = if app.console.input_mask {
            "*".repeat(app.console.input.len())
        } else {
            app.console.input.clone()
        };
        let label = app.console.input_label.as_deref().unwrap_or("Input");
        frame.render_widget(Paragraph::new(input_text).block(block(label)), chunks[2]);
        frame.set_cursor_position((
            chunks[2].x + 1 + app.console.input.len() as u16,
            chunks[2].y + 1,
        ));
    }
}

pub(super) fn draw_list_modal(frame: &mut ratatui::Frame, app: &App) {
    match &app.modal {
        Modal::None => {}
        Modal::EditFields {
            rows,
            row,
            col,
            ent_idx,
            dragging,
            ..
        } => draw_fields_modal(frame, app, rows, *row, *col, *ent_idx, *dragging),
        Modal::EditMeta { rows, row, .. } => draw_meta_modal(frame, rows, *row),
    }
}

fn draw_fields_modal(
    frame: &mut ratatui::Frame,
    app: &App,
    rows: &[FieldRow],
    selected_row: usize,
    selected_column: FieldCol,
    entity_index: usize,
    dragging: bool,
) {
    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );
    let area = centered_rect(90, 60, frame.area());
    frame.render_widget(Clear, area);
    let outer = block(
        "Edit Fields (↑/↓, ←/→, Enter=toggle/edit, Space=drag, a=add, x=del, s=save, q=cancel)",
    );
    frame.render_widget(outer.clone(), area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(outer.inner(area));
    let header = format!(
        "{:<15}  | {:<10} | {:<5} | {:<5} | {:<5} | {:<15} | {:<22} | {:<14} | {}",
        "name", "type", "req", "filt", "ui", "maxlen", "nav name", "nav display", "nav ns"
    );
    frame.render_widget(Paragraph::new(Span::styled(header, faint())), chunks[0]);

    let ui_locked = app
        .current_project_data
        .as_ref()
        .and_then(|history| history.entities.get(entity_index))
        .is_some_and(|entity| matches!(entity.ui_target, UiTarget::None));
    let items = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let selected = index == selected_row;
            let mut item = ListItem::new(format_field_row(
                row,
                selected.then_some(selected_column),
                ui_locked,
                selected && dragging,
            ));
            if index % 2 == 1 {
                item = item.style(Style::default().bg(THEME.surface2));
            }
            if selected {
                item = item.style(list_highlight());
            }
            item
        })
        .collect::<Vec<_>>();
    let mut state = ratatui::widgets::ListState::default().with_selected(Some(selected_row));
    frame.render_stateful_widget(List::new(items), chunks[1], &mut state);
}

fn format_field_row(
    row: &FieldRow,
    selected_column: Option<FieldCol>,
    ui_locked: bool,
    dragging: bool,
) -> Line<'static> {
    let is_guid = row.ftype.eq_ignore_ascii_case("Guid");
    let is_enum = row.ftype.eq_ignore_ascii_case("enum");
    let is_text =
        row.ftype.eq_ignore_ascii_case("string") || row.ftype.eq_ignore_ascii_case("textarea");
    let has_navigation = row.nav_name.is_some() || row.nav_ns.is_some();

    let mut name = row.name.clone();
    let mut field_type = row.ftype.clone();
    let mut required = if row.required { "✓" } else { " " }.to_owned();
    let mut filterable = if row.filterable { "✓" } else { " " }.to_owned();
    let mut show_ui = if ui_locked {
        "-".to_owned()
    } else if row.show_ui {
        "✓".to_owned()
    } else {
        " ".to_owned()
    };
    let mut maximum_length = if is_text {
        row.max_len
            .map(|value| value.to_string())
            .unwrap_or_default()
    } else {
        "-".to_owned()
    };
    let (mut navigation_name, mut navigation_display, mut navigation_namespace) =
        if is_guid || is_enum {
            if has_navigation {
                (
                    row.nav_name.clone().unwrap_or_default(),
                    if is_guid {
                        row.nav_display.clone().unwrap_or_else(|| "Name".into())
                    } else {
                        "-".into()
                    },
                    row.nav_ns.clone().unwrap_or_default(),
                )
            } else {
                (
                    String::new(),
                    if is_guid { String::new() } else { "-".into() },
                    String::new(),
                )
            }
        } else {
            ("-".into(), "-".into(), "-".into())
        };

    match selected_column {
        Some(FieldCol::Name) => name = selected_cell(name),
        Some(FieldCol::Type) => field_type = selected_cell(field_type),
        Some(FieldCol::Required) => required = selected_cell(required),
        Some(FieldCol::Filterable) => filterable = selected_cell(filterable),
        Some(FieldCol::ShowUi) if !ui_locked => show_ui = selected_cell(show_ui),
        Some(FieldCol::MaxLen) if is_text => maximum_length = selected_cell(maximum_length),
        Some(FieldCol::NavName) if is_guid || is_enum => {
            navigation_name = selected_cell(navigation_name)
        }
        Some(FieldCol::NavDisplay) if is_guid => {
            navigation_display = selected_cell(navigation_display)
        }
        Some(FieldCol::NavNs) if is_guid || is_enum => {
            navigation_namespace = selected_cell(navigation_namespace)
        }
        _ => {}
    }

    Line::from(format!(
        "{mark}{name} | {field_type} | {required} | {filterable} | {show_ui} | {maximum_length} | {navigation_name} | {navigation_display} | {navigation_namespace}",
        mark = if dragging { "▸" } else { " " },
        name = fit_cell(15, &name, CellAlignment::Left),
        field_type = fit_cell(10, &field_type, CellAlignment::Left),
        required = fit_cell(5, &required, CellAlignment::Center),
        filterable = fit_cell(5, &filterable, CellAlignment::Center),
        show_ui = fit_cell(5, &show_ui, CellAlignment::Center),
        maximum_length = fit_cell(15, &maximum_length, CellAlignment::Left),
        navigation_name = fit_cell(22, &navigation_name, CellAlignment::Left),
        navigation_display = fit_cell(14, &navigation_display, CellAlignment::Left),
        navigation_namespace = fit_cell(30, &navigation_namespace, CellAlignment::Left),
    ))
}

fn selected_cell(value: String) -> String {
    format!("[ {value} ]")
}

#[derive(Clone, Copy)]
enum CellAlignment {
    Left,
    Center,
}

fn fit_cell(width: usize, value: &str, alignment: CellAlignment) -> String {
    let truncated = value.chars().take(width).collect::<String>();
    let padding = width.saturating_sub(truncated.chars().count());
    match alignment {
        CellAlignment::Left => format!("{truncated}{}", " ".repeat(padding)),
        CellAlignment::Center => {
            let left = padding / 2;
            format!(
                "{}{truncated}{}",
                " ".repeat(left),
                " ".repeat(padding - left)
            )
        }
    }
}

fn draw_meta_modal(frame: &mut ratatui::Frame, rows: &[(String, String)], selected_row: usize) {
    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );
    let area = centered_rect(70, 40, frame.area());
    frame.render_widget(Clear, area);
    let block = block("Edit Meta (↑/↓, Enter=edit/cycle UI/Theme, s=save, q=cancel)");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items = rows
        .iter()
        .enumerate()
        .map(|(index, (key, value))| {
            let key_style = if index == selected_row {
                Style::default().fg(THEME.border)
            } else {
                accent()
            };
            let mut item = ListItem::new(Line::from(vec![
                Span::styled(format!("{key:<12}"), key_style),
                Span::raw(" | "),
                Span::styled(value, Style::default().fg(THEME.fg)),
            ]));
            if index % 2 == 1 {
                item = item.style(Style::default().bg(THEME.surface2));
            }
            if index == selected_row {
                item = item.style(list_highlight());
            }
            item
        })
        .collect::<Vec<_>>();
    frame.render_widget(List::new(items), inner);
}

pub(super) fn draw_confirm_modal(frame: &mut ratatui::Frame, app: &App) {
    if let Some(conf) = &app.confirm {
        frame.render_widget(
            Block::default().style(Style::default().bg(THEME.bg)),
            frame.area(),
        );

        let area = centered_rect(60, 28, frame.area());
        frame.render_widget(Clear, area);

        let block_confirm = block("Confirm").style(Style::default().bg(THEME.surface));
        let inner = block_confirm.inner(area);
        frame.render_widget(block_confirm, area);

        let message_height: u16 = 3;
        let button_height: u16 = 3;
        let spacing: u16 = 1;
        let total_content = message_height + spacing + button_height;
        let inner_height = inner.height.max(total_content);
        let bias_down = (inner_height as f32 * 0.1) as u16;
        let top_spacer = (inner_height - total_content) / 2 + bias_down;
        let bottom_spacer = inner_height.saturating_sub(total_content + top_spacer);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(top_spacer),
                Constraint::Length(message_height),
                Constraint::Length(spacing),
                Constraint::Length(button_height),
                Constraint::Length(bottom_spacer),
            ])
            .split(inner);

        let message = Paragraph::new(conf.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(THEME.fg));
        frame.render_widget(message, chunks[1]);

        let ok = if conf.ok_selected { "[ OK ]" } else { "  OK  " };
        let cancel = if conf.ok_selected {
            " Cancel "
        } else {
            "[Cancel]"
        };
        let buttons = Line::from(vec![
            Span::styled(cancel, if conf.ok_selected { faint() } else { accent() }),
            Span::raw("    "),
            Span::styled(ok, if conf.ok_selected { accent() } else { faint() }),
        ]);
        frame.render_widget(
            Paragraph::new(buttons).alignment(Alignment::Center),
            chunks[3],
        );
    }
}

pub(super) fn draw_help_modal(frame: &mut ratatui::Frame, app: &App) {
    if !app.help_open {
        return;
    }

    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);

    let block = block("Help — Shortcuts & Vim-style Navigation (q / Esc to close)");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = r#"
GLOBAL
  ?               : Help (this modal)
  q               : Exit / Back (contextual)
  Esc             : Cancel / Close (contextual)

LIST / SELECTION (Projects & Entities)
  j / k           : Down / Up
  gg / G          : Top / Bottom
  Enter           : Open / Regenerate
  n               : New (Project/Entity)
  r               : Rename project (on Projects screen)
  E               : Edit project meta (Theme / Override / Mobile UI) on Projects screen
  d               : Delete entity from history only (does not remove files)
  Del             : Remove project from global index (does not remove files)

CONFIRM (confirmation dialog)
  h / l, ← / →    : Toggle between Cancel / OK
  Enter           : Activate selected button
  Esc             : Close (Cancel)

INLINE CONSOLE (Wizard / textbox)
  Enter           : Submit
  Esc             : Cancel (asks for confirmation)
  Up/Down         : Scroll line by line
  PgUp / PgDn     : Page up / Page down
  Home / End      : Top / Bottom
  Ctrl+f / Ctrl+b : Page down / Page up

EDIT FIELDS (field editor)
  j / k           : Move selection down / up (when not dragging)
  h / l           : Move across columns
  gg / G          : First / Last row
  Space           : Toggle move mode (drag)
  J / K           : With drag ON, move row down / up
  a               : Insert new field (after selection)
  x               : Delete selected field
  Enter           : Edit/Toggle (Type/Name/MaxLen/NavName/NavDisplay/NavNs etc.)
  s               : Save + Regenerate
  q / Esc         : Cancel (asks for confirmation)

EDIT META (Namespace / UI)
  j / k           : Row selection
  h / l           : Focus value column (the only editable one)
  gg / G          : First / Last row
  Enter           : On UI row, cycle next valid target (None/MVC/Angular);
                    on others, open edit dialog
  s               : Save + Regenerate
  q / Esc         : Cancel (asks for confirmation)
"#;

    let paragraph = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(THEME.fg))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(THEME.surface)),
        );
    frame.render_widget(paragraph, inner);
}

pub(super) fn draw_projects_ui(frame: &mut ratatui::Frame, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );

    const LOGO: &str = r#"
 █████                                              
░░███                                               
 ░███████   ██████   ████████   ████████    ██████  
 ░███░░███ ░░░░░███ ░░███░░███ ░░███░░███  ░░░░░███ 
 ░███ ░███  ███████  ░███ ░███  ░███ ░███   ███████ 
 ░███ ░███ ███░░███  ░███ ░███  ░███ ░███  ███░░███ 
 ████████ ░░████████ ████ █████ ████ █████░░████████
░░░░░░░░   ░░░░░░░░ ░░░░ ░░░░░ ░░░░ ░░░░░  ░░░░░░░░ 

Shape the model. Generate the layers.
"#;

    let header_height = LOGO.lines().count() as u16 + 2;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height.max(8)),
            Constraint::Min(5),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let header = Paragraph::new(LOGO)
        .block(block("Home"))
        .style(accent())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = if app.global.projects.is_empty() {
        vec![ListItem::new(Span::styled(
            "No projects. Press 'n' to create.",
            faint(),
        ))]
    } else {
        app.global
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let mark = if app.project_dragging && app.list_index == index {
                    "▸ "
                } else {
                    "  "
                };
                let theme = load_project_history(Path::new(&project.project_dir))
                    .ok()
                    .flatten()
                    .map(|record| theme_label(record.theme))
                    .unwrap_or("N/A");
                ListItem::new(Line::from(vec![
                    Span::raw(mark),
                    Span::styled(format!("{:>2}. ", index + 1), faint()),
                    Span::styled(
                        &project.project_name,
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("—", faint()),
                    Span::raw("  "),
                    Span::styled(&project.project_dir, Style::default().fg(THEME.info)),
                    Span::raw("  ["),
                    Span::styled(theme, Style::default().fg(THEME.accent_alt)),
                    Span::raw("]"),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(block(
            "Projects (↑/↓, Enter=open, r=rename, n=new, E=edit meta, Del=remove, Space=drag, q=quit)",
        ))
        .highlight_style(list_highlight());
    frame.render_stateful_widget(
        list,
        chunks[1],
        &mut ratatui::widgets::ListState::default().with_selected(Some(std::cmp::min(
            app.list_index,
            app.global.projects.len().saturating_sub(1),
        ))),
    );

    let status_style = if app.status.contains("Error") {
        danger()
    } else if app.status.contains("Done") || app.status.contains("Generated") {
        success()
    } else {
        Style::default().fg(THEME.fg)
    };
    frame.render_widget(
        Paragraph::new(app.status.clone())
            .block(block_muted("Status"))
            .style(status_style)
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

pub(super) fn draw_project_detail_ui(frame: &mut ratatui::Frame, app: &App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(THEME.bg)),
        frame.area(),
    );

    let (Some(project), Some(history)) = (
        app.current_project.as_ref(),
        app.current_project_data.as_ref(),
    ) else {
        frame.render_widget(
            Paragraph::new("Project data is unavailable. Press Esc to return to the project list.")
                .block(block_muted("Project unavailable"))
                .style(danger())
                .wrap(Wrap { trim: true }),
            frame.area(),
        );
        return;
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let header = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(project.project_name.as_str(), accent())),
        Line::from(Span::styled(
            &project.project_dir,
            Style::default().fg(THEME.info),
        )),
        Line::from(vec![
            Span::styled("Theme: ", faint()),
            Span::styled(
                theme_label(history.theme),
                Style::default().fg(THEME.accent_alt),
            ),
        ]),
    ]))
    .block(block("Project"));
    frame.render_widget(header, chunks[0]);

    let selected = (!history.entities.is_empty())
        .then(|| std::cmp::min(app.list_index, history.entities.len() - 1));
    let items: Vec<ListItem> = if history.entities.is_empty() {
        vec![ListItem::new(Span::styled(
            "No entities. Press 'n' to create.",
            faint(),
        ))]
    } else {
        history
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| {
                let mark = if app.entity_dragging && Some(index) == selected {
                    "▸ "
                } else {
                    "  "
                };
                ListItem::new(Line::from(vec![
                    Span::raw(mark),
                    Span::styled(format!("{:>2}. ", index + 1), faint()),
                    Span::styled(&entity.name, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("  ["),
                    Span::styled(&entity.namespace, Style::default().fg(THEME.muted)),
                    Span::raw("]  "),
                    Span::styled(
                        entity.generated_at.format("%d.%m.%Y %H:%M").to_string(),
                        Style::default().fg(THEME.accent_alt),
                    ),
                ]))
            })
            .collect()
    };
    let list = List::new(items)
        .block(block(
            "Entities (↑/↓, Enter=regen, n=new, e=edit fields, E=edit meta, d=delete, Space=drag, q=back)",
        ))
        .highlight_style(list_highlight());
    frame.render_stateful_widget(
        list,
        chunks[1],
        &mut ratatui::widgets::ListState::default().with_selected(selected),
    );

    let status_style = if app.status.contains("Error") {
        danger()
    } else if app.status.contains("Regenerated") || app.status.contains("Generated") {
        success()
    } else {
        Style::default().fg(THEME.fg)
    };
    frame.render_widget(
        Paragraph::new(app.status.clone())
            .block(block_muted("Status"))
            .style(status_style)
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_row() -> FieldRow {
        FieldRow {
            name: "CustomerId".into(),
            ftype: "Guid".into(),
            required: true,
            max_len: None,
            nav_name: Some("Customer".into()),
            nav_ns: Some("Acme.Customers".into()),
            nav_display: Some("Name".into()),
            filterable: true,
            show_ui: true,
        }
    }

    #[test]
    fn field_rows_keep_navigation_columns_and_selection() {
        let rendered =
            format_field_row(&field_row(), Some(FieldCol::NavDisplay), false, true).to_string();

        assert!(rendered.starts_with('▸'));
        assert!(rendered.contains("Customer"));
        assert!(rendered.contains("[ Name ]"));
        assert!(rendered.contains("Acme.Customers"));
    }

    #[test]
    fn locked_ui_cells_are_not_presented_as_editable() {
        let rendered =
            format_field_row(&field_row(), Some(FieldCol::ShowUi), true, false).to_string();

        assert!(!rendered.contains("[ - ]"));
    }
}

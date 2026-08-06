use super::*;

/// Handles field/meta editor input and reports whether a modal consumed the key.
pub(super) fn handle_modal_key(app: &mut App<'_>, k: crossterm::event::KeyEvent) -> bool {
    match &mut app.modal {
        Modal::None => false,
        Modal::EditFields {
            rows,
            row,
            col,
            dirty,
            ent_idx,
            dragging,
        } => {
            let ui_locked = {
                if let Some(hist) = app.current_project_data.as_ref() {
                    if *ent_idx < hist.entities.len() {
                        matches!(hist.entities[*ent_idx].ui_target, UiTarget::None)
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            match k.code {
                KeyCode::Char('j') => {
                    if *dragging {
                        if *row + 1 < rows.len() {
                            rows.swap(*row, *row + 1);
                            *row += 1;
                            *dirty = true;
                        }
                    } else {
                        if *row + 1 < rows.len() {
                            *row += 1;
                        }
                        if rows
                            .get(*row)
                            .map(|r| !r.ftype.eq_ignore_ascii_case("string"))
                            .unwrap_or(false)
                            && matches!(col, FieldCol::MaxLen)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows
                            .get(*row)
                            .map(|r| r.ftype != "Guid" && r.ftype != "enum")
                            .unwrap_or(true)
                            && matches!(col, FieldCol::NavName | FieldCol::NavNs)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows.get(*row).map(|r| r.ftype != "Guid").unwrap_or(true)
                            && matches!(col, FieldCol::NavDisplay)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows.get(*row).map(|r| r.ftype != "Guid").unwrap_or(true)
                            && matches!(col, FieldCol::NavDisplay)
                        {
                            *col = FieldCol::Name;
                        }
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('k') => {
                    if *dragging {
                        if *row > 0 {
                            rows.swap(*row, *row - 1);
                            *row -= 1;
                            *dirty = true;
                        }
                    } else {
                        if *row > 0 {
                            *row -= 1;
                        }
                        if rows
                            .get(*row)
                            .map(|r| !r.ftype.eq_ignore_ascii_case("string"))
                            .unwrap_or(false)
                            && matches!(col, FieldCol::MaxLen)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows
                            .get(*row)
                            .map(|r| r.ftype != "Guid" && r.ftype != "enum")
                            .unwrap_or(true)
                            && matches!(col, FieldCol::NavName | FieldCol::NavNs)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows.get(*row).map(|r| r.ftype != "Guid").unwrap_or(true)
                            && matches!(col, FieldCol::NavDisplay)
                        {
                            *col = FieldCol::Name;
                        }
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('h') => {
                    if !*dragging {
                        *col = next_field_column(
                            *col,
                            rows.get(*row),
                            ui_locked,
                            ColumnDirection::Left,
                        );
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('l') => {
                    if !*dragging {
                        *col = next_field_column(
                            *col,
                            rows.get(*row),
                            ui_locked,
                            ColumnDirection::Right,
                        );
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('g') => {
                    if app.vim_pending_g {
                        if !rows.is_empty() {
                            *row = 0;
                        }
                        app.vim_pending_g = false;
                    } else {
                        app.vim_pending_g = true;
                    }
                }
                KeyCode::Char('G') => {
                    if !rows.is_empty() {
                        *row = rows.len() - 1;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('J') => {
                    if *dragging && *row + 1 < rows.len() {
                        rows.swap(*row, *row + 1);
                        *row += 1;
                        *dirty = true;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('K') => {
                    if *dragging && *row > 0 {
                        rows.swap(*row, *row - 1);
                        *row -= 1;
                        *dirty = true;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.vim_pending_g = false;
                    app.confirm(
                        "Discard changes and close Edit Fields?",
                        PendingAction::CancelEditFields,
                    );
                }
                KeyCode::Down => {
                    app.vim_pending_g = false;
                    if *dragging {
                        if *row + 1 < rows.len() {
                            rows.swap(*row, *row + 1);
                            *row += 1;
                            *dirty = true;
                        }
                    } else {
                        if *row + 1 < rows.len() {
                            *row += 1;
                        }
                        if rows
                            .get(*row)
                            .map(|r| !r.ftype.eq_ignore_ascii_case("string"))
                            .unwrap_or(false)
                            && matches!(col, FieldCol::MaxLen)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows
                            .get(*row)
                            .map(|r| r.ftype != "Guid" && r.ftype != "enum")
                            .unwrap_or(true)
                            && matches!(col, FieldCol::NavName | FieldCol::NavNs)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows.get(*row).map(|r| r.ftype != "Guid").unwrap_or(true)
                            && matches!(col, FieldCol::NavDisplay)
                        {
                            *col = FieldCol::Name;
                        }
                    }
                }
                KeyCode::Up => {
                    app.vim_pending_g = false;
                    if *dragging {
                        if *row > 0 {
                            rows.swap(*row, *row - 1);
                            *row -= 1;
                            *dirty = true;
                        }
                    } else {
                        if *row > 0 {
                            *row -= 1;
                        }
                        if rows
                            .get(*row)
                            .map(|r| !r.ftype.eq_ignore_ascii_case("string"))
                            .unwrap_or(false)
                            && matches!(col, FieldCol::MaxLen)
                        {
                            *col = FieldCol::Name;
                        }
                        if rows.get(*row).map(|r| r.ftype != "Guid").unwrap_or(true)
                            && matches!(
                                col,
                                FieldCol::NavName | FieldCol::NavDisplay | FieldCol::NavNs
                            )
                        {
                            *col = FieldCol::Name;
                        }
                    }
                }
                KeyCode::Left => {
                    app.vim_pending_g = false;
                    if !*dragging {
                        *col = next_field_column(
                            *col,
                            rows.get(*row),
                            ui_locked,
                            ColumnDirection::Left,
                        );
                    }
                }
                KeyCode::Right => {
                    app.vim_pending_g = false;
                    if !*dragging {
                        *col = next_field_column(
                            *col,
                            rows.get(*row),
                            ui_locked,
                            ColumnDirection::Right,
                        );
                    }
                }
                KeyCode::Char('a') => {
                    app.vim_pending_g = false;
                    if !*dragging {
                        let insert_at = if rows.is_empty() {
                            0
                        } else {
                            (*row + 1).min(rows.len())
                        };

                        rows.insert(
                            insert_at,
                            FieldRow {
                                name: "NewField".into(),
                                ftype: "string".into(),
                                required: false,
                                max_len: None,
                                nav_name: None,
                                nav_display: None,
                                nav_ns: None,
                                filterable: false,
                                show_ui: false,
                            },
                        );

                        *row = insert_at;
                        *dirty = true;
                    }
                }
                KeyCode::Char('x') => {
                    app.vim_pending_g = false;
                    if !*dragging && !rows.is_empty() {
                        rows.remove(*row);
                        if *row >= rows.len() {
                            *row = rows.len().saturating_sub(1);
                        }
                        *dirty = true;
                    }
                }
                KeyCode::Enter => {
                    app.vim_pending_g = false;
                    if !*dragging {
                        match col {
                            FieldCol::Type => {
                                const TYPES: [&str; 8] = [
                                    "string", "textarea", "int", "long", "decimal", "Guid", "bool",
                                    "DateTime",
                                ];
                                let cur = rows[*row].ftype.as_str();
                                let mut idx = TYPES
                                    .iter()
                                    .position(|t| t.eq_ignore_ascii_case(cur))
                                    .unwrap_or(0);
                                idx = (idx + 1) % TYPES.len();
                                rows[*row].ftype = TYPES[idx].to_string();

                                if rows[*row].ftype != "Guid" && rows[*row].ftype != "enum" {
                                    rows[*row].nav_name = None;
                                    rows[*row].nav_display = None;
                                    rows[*row].nav_ns = None;
                                    if matches!(
                                        col,
                                        FieldCol::NavName | FieldCol::NavDisplay | FieldCol::NavNs
                                    ) {
                                        *col = FieldCol::MaxLen;
                                    }
                                } else if rows[*row].ftype == "enum" {
                                    rows[*row].nav_display = None;
                                }
                                app.status = format!("Type = {}", rows[*row].ftype);
                            }
                            FieldCol::Required => {
                                rows[*row].required = !rows[*row].required;
                                app.status = if rows[*row].required {
                                    "Required = true"
                                } else {
                                    "Required = false"
                                }
                                .into();
                            }
                            FieldCol::Filterable => {
                                rows[*row].filterable = !rows[*row].filterable;
                                app.status = if rows[*row].filterable {
                                    "Filterable = true"
                                } else {
                                    "Filterable = false"
                                }
                                .into();
                            }
                            FieldCol::ShowUi => {
                                rows[*row].show_ui = !rows[*row].show_ui;
                                app.status = if rows[*row].show_ui {
                                    "ShowUi = true"
                                } else {
                                    "ShowUi = false"
                                }
                                .into();
                            }

                            FieldCol::Name
                            | FieldCol::MaxLen
                            | FieldCol::NavName
                            | FieldCol::NavDisplay
                            | FieldCol::NavNs => {
                                if matches!(col, FieldCol::NavDisplay) && rows[*row].ftype != "Guid"
                                {
                                    app.status =
                                        "Navigation Display is only for Guid fields".into();
                                } else if matches!(col, FieldCol::NavName | FieldCol::NavNs)
                                    && !(rows[*row].ftype == "Guid" || rows[*row].ftype == "enum")
                                {
                                    app.status =
                                        "Navigation fields are disabled unless Type = Guid or Enum"
                                            .into();
                                } else {
                                    let (prompt, current) = match col {
                                        FieldCol::Name => ("Name", rows[*row].name.clone()),
                                        FieldCol::MaxLen => (
                                            "Max length (empty = clear)",
                                            rows[*row]
                                                .max_len
                                                .map(|n| n.to_string())
                                                .unwrap_or_default(),
                                        ),
                                        FieldCol::NavName => (
                                            "Navigation Class Name (empty = clear)",
                                            rows[*row].nav_name.clone().unwrap_or_default(),
                                        ),
                                        FieldCol::NavDisplay => (
                                            "Navigation Display Property (empty = clear, default Name)",
                                            rows[*row].nav_display.clone().unwrap_or_default(),
                                        ),
                                        FieldCol::NavNs => (
                                            "Navigation Namespace (empty = clear)",
                                            rows[*row].nav_ns.clone().unwrap_or_default(),
                                        ),
                                        _ => unreachable!(),
                                    };

                                    let mut c = InlineConsole::new(format!("Edit {}", prompt));
                                    let row_idx = *row;
                                    let col_now = *col;
                                    let workflow = app.workflow.clone();

                                    c.on_submit = Some(Box::new(move |line: String| {
                                        workflow.complete_field_edit(row_idx, col_now, line);
                                        None
                                    }));
                                    c.on_cancel = Some(Box::new(|| {}));
                                    c.open();
                                    c.println(format!("{} (current: {})", prompt, current));
                                    c.input = current;
                                    app.console = c;
                                    app.status = "Editing cell...".into();
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    app.vim_pending_g = false;
                    *dragging = !*dragging;
                    if *dragging {
                        app.status =
                            "Move mode ON: use ↑/↓ to reorder, press Space to finish".into();
                    } else {
                        app.status = "Move mode OFF".into();
                    }
                }
                KeyCode::Char('s') => {
                    app.vim_pending_g = false;
                    *dragging = false;
                    app.confirm(
                        "Save changes and regenerate now?",
                        PendingAction::SaveEditFields,
                    );
                }
                _ => {}
            }
            true
        }
        Modal::EditMeta {
            rows,
            row,
            col,
            target,
            ..
        } => {
            match k.code {
                KeyCode::Char('j') => {
                    let len = rows.len();
                    if len > 0 {
                        *row = (*row + 1) % len;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('k') => {
                    let len = rows.len();
                    if len > 0 {
                        *row = (*row + len - 1) % len;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('g') => {
                    if app.vim_pending_g {
                        if !rows.is_empty() {
                            *row = 0;
                        }
                        app.vim_pending_g = false;
                    } else {
                        app.vim_pending_g = true;
                    }
                }
                KeyCode::Char('G') => {
                    if !rows.is_empty() {
                        *row = rows.len() - 1;
                    }
                    app.vim_pending_g = false;
                }
                KeyCode::Char('h') | KeyCode::Char('l') => {
                    *col = 1;
                    app.vim_pending_g = false;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.vim_pending_g = false;
                    app.confirm(
                        "Discard changes and close Edit Meta?",
                        PendingAction::CancelEditMeta,
                    );
                }
                KeyCode::Down => {
                    app.vim_pending_g = false;
                    let len = rows.len();
                    if len > 0 {
                        *row = (*row + 1) % len;
                    }
                }
                KeyCode::Up => {
                    app.vim_pending_g = false;
                    let len = rows.len();
                    if len > 0 {
                        *row = (*row + len - 1) % len;
                    }
                }
                KeyCode::Enter => {
                    app.vim_pending_g = false;
                    if *col == 1 {
                        let key = rows.get(*row).map(|(k, _)| k.as_str());
                        match (*target, key) {
                            (MetaTarget::Entity(ent_idx), Some("UI")) => {
                                let (has_mvc, has_ng) = (|| {
                                    if let (Some(pref), Some(hist)) =
                                        (&app.current_project, &app.current_project_data)
                                        && ent_idx < hist.entities.len()
                                    {
                                        let domain = hist.entities[ent_idx].domain.clone();
                                        let root = PathBuf::from(&pref.project_dir);
                                        return generator::detect_ui(&root, &domain);
                                    }
                                    (false, false)
                                })();

                                let cur = match rows[*row].1.to_ascii_lowercase().as_str() {
                                    "razor" => UiTarget::Razor,
                                    "vue" => UiTarget::Vue,
                                    "angular" => UiTarget::Angular,
                                    _ => UiTarget::None,
                                };

                                let next = cycle_next_ui(cur, has_mvc, has_ng);
                                rows[*row].1 = match next {
                                    UiTarget::Razor => "Razor".into(),
                                    UiTarget::Vue => "Vue".into(),
                                    UiTarget::Angular => "Angular".into(),
                                    UiTarget::None => "None".into(),
                                };

                                app.status = match (has_mvc, has_ng) {
                                    (false, false) => "There is no UI Target; UI = None".into(),
                                    _ => format!("UI: {}", rows[*row].1),
                                };
                            }
                            (MetaTarget::Project, Some("Theme")) => {
                                let cur = sanitize_theme_choice(&rows[*row].1);
                                let next = cycle_next_theme(cur);
                                rows[*row].1 = theme_label(next).to_string();
                                app.status = format!("Theme: {}", theme_label(next));
                            }
                            (MetaTarget::Project, Some("Bootstrap Override")) => {
                                let cur = sanitize_override_choice(&rows[*row].1);
                                let next = cycle_next_override(cur);
                                rows[*row].1 = override_label(next).to_string();
                                app.status = format!("Bootstrap override: {}", rows[*row].1);
                            }
                            (MetaTarget::Project, Some("Mobile UI")) => {
                                let cur = sanitize_mobile_ui_choice(&rows[*row].1);
                                let next = cycle_next_mobile_ui(cur);
                                rows[*row].1 = mobile_ui_label(next).to_string();
                                app.status = format!("Mobile UI: {}", mobile_ui_label(next));
                            }
                            _ => {
                                let key = rows[*row].0.clone();
                                let cur = rows[*row].1.clone();
                                let mut c = InlineConsole::new(format!("Edit {}", key));
                                let workflow = app.workflow.clone();
                                c.on_submit = Some(Box::new(move |line: String| {
                                    let v = line.trim().to_string();
                                    workflow.complete_meta_edit(v);
                                    None
                                }));
                                c.open();
                                c.println(format!("Value (current: {})", cur));
                                app.console = c;
                            }
                        }
                    }
                }
                KeyCode::Char('s') => {
                    app.vim_pending_g = false;
                    let msg = match target {
                        MetaTarget::Entity(_) => "Save meta changes and regenerate now?",
                        MetaTarget::Project => "Save project meta changes?",
                    };
                    app.confirm(msg, PendingAction::SaveEditMeta);
                }
                _ => {}
            }
            true
        }
    }
}

const FIELD_COLUMNS: [FieldCol; 9] = [
    FieldCol::Name,
    FieldCol::Type,
    FieldCol::Required,
    FieldCol::Filterable,
    FieldCol::ShowUi,
    FieldCol::MaxLen,
    FieldCol::NavName,
    FieldCol::NavDisplay,
    FieldCol::NavNs,
];

#[derive(Clone, Copy)]
enum ColumnDirection {
    Left,
    Right,
}

fn next_field_column(
    current: FieldCol,
    field: Option<&FieldRow>,
    ui_locked: bool,
    direction: ColumnDirection,
) -> FieldCol {
    let mut index = FIELD_COLUMNS
        .iter()
        .position(|column| *column == current)
        .unwrap_or(0);
    for _ in 0..FIELD_COLUMNS.len() {
        index = match direction {
            ColumnDirection::Left => {
                (index as i32 - 1).rem_euclid(FIELD_COLUMNS.len() as i32) as usize
            }
            ColumnDirection::Right => (index + 1) % FIELD_COLUMNS.len(),
        };
        let candidate = FIELD_COLUMNS[index];
        if field_column_available(candidate, field, ui_locked) {
            return candidate;
        }
    }
    current
}

fn field_column_available(column: FieldCol, field: Option<&FieldRow>, ui_locked: bool) -> bool {
    let field_type = field.map(|field| field.ftype.as_str()).unwrap_or_default();
    match column {
        FieldCol::ShowUi => !ui_locked,
        FieldCol::MaxLen => {
            field_type.eq_ignore_ascii_case("string") || field_type.eq_ignore_ascii_case("textarea")
        }
        FieldCol::NavDisplay => field_type == "Guid",
        FieldCol::NavName | FieldCol::NavNs => matches!(field_type, "Guid" | "enum"),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(field_type: &str) -> FieldRow {
        FieldRow {
            name: "Value".into(),
            ftype: field_type.into(),
            required: false,
            max_len: None,
            nav_name: None,
            nav_ns: None,
            nav_display: None,
            filterable: false,
            show_ui: false,
        }
    }

    #[test]
    fn skips_inapplicable_columns_for_plain_fields() {
        let plain = field("int");
        assert_eq!(
            next_field_column(
                FieldCol::ShowUi,
                Some(&plain),
                false,
                ColumnDirection::Right,
            ),
            FieldCol::Name
        );
    }

    #[test]
    fn exposes_navigation_columns_for_guid_fields() {
        let guid = field("Guid");
        assert_eq!(
            next_field_column(FieldCol::MaxLen, Some(&guid), false, ColumnDirection::Right,),
            FieldCol::NavName
        );
        assert_eq!(
            next_field_column(
                FieldCol::NavName,
                Some(&guid),
                false,
                ColumnDirection::Right,
            ),
            FieldCol::NavDisplay
        );
    }

    #[test]
    fn skips_ui_column_when_the_entity_has_no_ui_target() {
        let text = field("string");
        assert_eq!(
            next_field_column(
                FieldCol::Filterable,
                Some(&text),
                true,
                ColumnDirection::Right,
            ),
            FieldCol::MaxLen
        );
    }
}

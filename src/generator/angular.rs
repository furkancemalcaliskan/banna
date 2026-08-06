use super::GenerationContext;
use crate::helpers::last_segment;
use crate::models::Field;
use crate::templates::{embedded_walk, read_tpl_text};
use crate::utils::{pluralize, render_template, to_kebab, to_lower_camel, write_text};
use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn insert_into_array(source: &str, declaration: &Regex, snippet: &str) -> Option<String> {
    let declaration = declaration.find(source)?;
    let open = declaration.end() + source[declaration.end()..].find('[')?;
    let mut depth = 0_u32;

    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    let close = open + offset;
                    let mut updated = String::with_capacity(source.len() + snippet.len());
                    updated.push_str(&source[..close]);
                    updated.push_str(snippet);
                    updated.push_str(&source[close..]);
                    return Some(updated);
                }
            }
            _ => {}
        }
    }

    None
}

fn ensure_angular_theme_changer(app_dir: &Path, log: &mut dyn FnMut(&str)) -> Result<()> {
    let feature_dir = app_dir.join("theme-changer");
    fs::create_dir_all(&feature_dir)?;
    for (template, output) in [
        (
            "Web/Angular/theme-changer/theme-changer.component.ts.tpl",
            "theme-changer.component.ts",
        ),
        (
            "Web/Angular/theme-changer/theme-changer.provider.ts.tpl",
            "theme-changer.provider.ts",
        ),
    ] {
        let target = feature_dir.join(output);
        if target.exists() {
            log(&format!("skip: {} already exists", target.display()));
            continue;
        }
        write_text(&target, &read_tpl_text(template)?)?;
        log(&format!("created (ng theme changer): {}", target.display()));
    }

    let standalone_config = app_dir.join("app.config.ts");
    if standalone_config.is_file() {
        let mut text = fs::read_to_string(&standalone_config)?;
        let import =
            "import { provideThemeChanger } from './theme-changer/theme-changer.provider';";
        if !text.contains(import) {
            text.insert_str(0, &format!("{import}\n"));
        }
        if !text.contains("provideThemeChanger()") {
            let declaration = Regex::new(r"providers\s*:\s*")?;
            text = insert_into_array(&text, &declaration, "\n    provideThemeChanger(),")
                .ok_or_else(|| {
                    anyhow!(
                        "Angular providers array not found in {}; theme changer was not registered",
                        standalone_config.display()
                    )
                })?;
        }
        write_text(&standalone_config, &text)?;
        log(&format!(
            "registered Angular theme changer in {}",
            standalone_config.display()
        ));
        return Ok(());
    }

    let legacy_module = ["app.module.ts", "app.module.tsx"]
        .into_iter()
        .map(|name| app_dir.join(name))
        .find(|path| path.is_file());
    if let Some(legacy_module) = legacy_module {
        let mut text = fs::read_to_string(&legacy_module)?;
        let import =
            "import { THEME_CHANGER_PROVIDER } from './theme-changer/theme-changer.provider';";
        if !text.contains(import) {
            text.insert_str(0, &format!("{import}\n"));
        }
        if !text.contains("THEME_CHANGER_PROVIDER,") {
            let declaration = Regex::new(r"providers\s*:\s*")?;
            if let Some(updated) =
                insert_into_array(&text, &declaration, "\n    THEME_CHANGER_PROVIDER,")
            {
                text = updated;
            } else {
                let module = Regex::new(r"@NgModule\s*\(\s*\{")?;
                let marker = module.find(&text).ok_or_else(|| {
                    anyhow!(
                        "Angular NgModule metadata not found in {}; theme changer was not registered",
                        legacy_module.display()
                    )
                })?;
                text.insert_str(marker.end(), "\n  providers: [THEME_CHANGER_PROVIDER],");
            }
        }
        write_text(&legacy_module, &text)?;
        log(&format!(
            "registered Angular theme changer in {}",
            legacy_module.display()
        ));
        return Ok(());
    }

    Err(anyhow!(
        "Angular startup file not found under {}; expected app.config.ts or app.module.ts",
        app_dir.display()
    ))
}

/* ------------------------------ ANGULAR HELPERS ------------------------- */

struct AngularBlocks {
    datatable_columns: String,
    modal_form_fields: String,
    form_group_entries: String,
    advanced_filter_inputs: String,
    nav_option_arrays: String,
    nav_search_subjects: String,
    nav_search_bindings: String,
    nav_load_calls: String,
    nav_helpers: String,
    payload_mappings: String,
    excel_filter_body: String,
    excel_query_appends: String,
    proxy_create_fields: String,
    proxy_dto_fields: String,
    proxy_update_fields: String,
    proxy_excel_fields: String,
    proxy_filter_fields: String,
    proxy_filter_params: String,
    proxy_delete_filter_params: String,
    proxy_excel_params: String,
    proxy_with_nav_fields: String,
    proxy_nav_models: String,
    proxy_lookup_methods: String,
}

fn angular_type(field: &Field) -> &'static str {
    match field.ftype.as_str() {
        "bool" => "boolean",
        "int" | "long" | "decimal" | "enum" => "number",
        _ => "string",
    }
}

fn angular_blocks(entity: &str, fields: &[Field]) -> AngularBlocks {
    let mut cols = String::new();
    let mut modal = String::new();
    let mut entries = String::new();
    let mut adv = String::new();
    let mut payload_mappings = String::new();
    let mut excel_filter_body = String::from("      filterText: f.filterText ?? '',\n");
    let mut excel_query_appends = String::new();

    let mut nav_option_arrays = String::new();
    let mut nav_search_subjects = String::new();
    let mut nav_search_bindings = String::new();
    let mut nav_load_calls = String::new();
    let mut nav_helpers = String::new();

    let mut proxy_create_fields = String::new();
    let mut proxy_dto_fields = String::new();
    let mut proxy_update_fields = String::new();
    let mut proxy_excel_fields = String::new();
    let mut proxy_filter_fields = String::new();
    let mut proxy_filter_params = vec![
        "filterText: input.filterText".to_string(),
        "sorting: input.sorting".to_string(),
        "skipCount: input.skipCount".to_string(),
        "maxResultCount: input.maxResultCount".to_string(),
    ];
    let mut proxy_delete_filter_params = vec!["filterText: input.filterText".to_string()];
    let mut proxy_excel_params = vec![
        "downloadToken: input.downloadToken".to_string(),
        "filterText: input.filterText".to_string(),
    ];
    let mut proxy_with_nav_fields = String::new();
    let mut proxy_nav_models = String::new();
    let mut proxy_lookup_methods = String::new();

    let entity_lc = to_lower_camel(entity);
    let mut used_navs: Vec<String> = Vec::new();

    for f in fields {
        let field_cs = &f.name;
        let fname = to_lower_camel(field_cs);
        let required = f.required;

        let is_nav = f.ftype == "Guid" && f.navigation.is_some();
        let is_textarea = f.ftype.eq_ignore_ascii_case("textarea");
        let is_bool = f.ftype == "bool";
        let is_dt = f.ftype.eq_ignore_ascii_case("datetime");
        let is_num = matches!(f.ftype.as_str(), "int" | "long" | "decimal" | "enum");
        let ts_type = angular_type(f);

        let input_field = if required {
            format!("  {fname}: {ts_type};\n")
        } else {
            format!("  {fname}?: {ts_type} | null;\n")
        };
        proxy_create_fields.push_str(&input_field);
        proxy_update_fields.push_str(&input_field);
        proxy_dto_fields.push_str(&format!("  {fname}?: {ts_type} | null;\n"));
        proxy_excel_fields.push_str(&format!("  {fname}?: {ts_type} | null;\n"));

        if f.filterable {
            proxy_filter_fields.push_str(&format!("  {fname}?: {ts_type} | null;\n"));
            proxy_filter_params.push(format!("{fname}: input.{fname}"));
            proxy_delete_filter_params.push(format!("{fname}: input.{fname}"));
            proxy_excel_params.push(format!("{fname}: input.{fname}"));
        }

        if f.show_in_ui {
            if is_nav {
                let nav_short = f
                    .navigation
                    .as_deref()
                    .map(last_segment)
                    .unwrap_or_default();
                let nav_lc = to_lower_camel(nav_short);
                let display_lc = to_lower_camel(f.navigation_display.as_deref().unwrap_or("Name"));
                cols.push_str(&format!(
                    r#"
  <!-- {cap} (navigation) -->
  <ngx-datatable-column [name]="'::{scr}:{cap}' | abpLocalization">
    <ng-template let-row="row" ngx-datatable-cell-template>
      {{{{ row.{nav}?.{display} ?? '-' }}}}
    </ng-template>
  </ngx-datatable-column>"#,
                    cap = field_cs,
                    scr = entity,
                    nav = nav_lc,
                    display = display_lc
                ));
            } else {
                cols.push_str(&format!(
                    r#"
  <!-- {cap} -->
  <ngx-datatable-column [name]="'::{scr}:{cap}' | abpLocalization">
    <ng-template let-row="row" ngx-datatable-cell-template>
      {{{{ row.{ent}.{prop} ?? '-' }}}}
    </ng-template>
  </ngx-datatable-column>"#,
                    cap = field_cs,
                    scr = entity,
                    ent = entity_lc,
                    prop = fname
                ));
            }
        }

        let req_mark = if required { " *" } else { "" };
        let validation = if required {
            format!(
                r#"
      <div class="text-danger" *ngIf="form.get('{name}')?.invalid && form.get('{name}')?.touched">
        {{{{ '::FieldIsRequired' | abpLocalization }}}}
      </div>"#,
                name = fname
            )
        } else {
            String::new()
        };
        let max_length_attribute = f
            .max_length
            .filter(|_| f.ftype == "string" || is_textarea)
            .map(|max| format!(r#" maxlength="{max}""#))
            .unwrap_or_default();

        if is_nav {
            let nav_short = f
                .navigation
                .as_deref()
                .map(last_segment)
                .unwrap_or_default();
            let nav_lc = to_lower_camel(nav_short);

            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <div class="input-group">
          <input
            class="form-control"
            type="text"
            [placeholder]="'::Search' | abpLocalization"
            #{nav}ModalSearch
            (input)="on{Nav}Search({nav}ModalSearch.value)"
          />
          <select class="form-select" formControlName="{ctrl}">
            <option [ngValue]="null"></option>
            <option *ngFor="let o of {nav}Options" [ngValue]="o.id">{{ o.displayName }}</option>
          </select>
        </div>{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                nav = nav_lc,
                Nav = nav_short,
                val = validation
            ));

            if !used_navs.iter().any(|s| s == nav_short) {
                used_navs.push(nav_short.to_string());
                nav_option_arrays.push_str(&format!(
                    "  {nav}Options: LookupDto<string>[] = [];\n",
                    nav = nav_lc
                ));
                nav_search_subjects.push_str(&format!(
                    "  private readonly {nav}Search$ = new Subject<string>();\n",
                    nav = nav_lc
                ));
                nav_search_bindings.push_str(&format!(
                    r#"
    this.{nav}Search$
      .pipe(
        debounceTime(300),
        distinctUntilChanged(),
        switchMap(filter =>
          this.service.get{Nav}Lookup({{ filter, maxResultCount: 50, skipCount: 0 }}),
        ),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(res => (this.{nav}Options = res.items ?? []));"#,
                    Nav = nav_short,
                    nav = nav_lc
                ));
                nav_load_calls.push_str(&format!("    this.on{Nav}Search('');\n", Nav = nav_short));
                nav_helpers.push_str(&format!(
                    r#"
  on{Nav}Search(term: string): void {{
    this.{nav}Search$.next(term.trim());
  }}"#,
                    Nav = nav_short,
                    nav = nav_lc
                ));

                let display_name = f.navigation_display.as_deref().unwrap_or("Name");
                let display_lc = to_lower_camel(display_name);
                proxy_with_nav_fields.push_str(&format!("  {nav_lc}?: {nav_short}Dto;\n"));
                proxy_nav_models.push_str(&format!(
                    "export interface {nav_short}Dto {{\n  id?: string;\n  {display_lc}?: string | null;\n}}\n\n"
                ));
                proxy_lookup_methods.push_str(&format!(
                    r#"  get{nav_short}Lookup = (input: LookupRequestDto, config?: Partial<Rest.Config>) =>
    this.restService.request<unknown, PagedResultDto<LookupDto<string>>>(
      {{
        method: 'GET',
        url: '/api/app/{plural_kebab}/{nav_kebab}-lookup',
        params: {{
          filter: input.filter,
          sorting: input.sorting,
          skipCount: input.skipCount,
          maxResultCount: input.maxResultCount,
        }},
      }},
      {{ apiName: this.apiName, ...config }},
    );

"#,
                    plural_kebab = to_kebab(&pluralize(entity)),
                    nav_kebab = to_kebab(nav_short),
                ));
            }
        } else if is_bool {
            modal.push_str(&format!(
r#"
      <div class="form-group mt-2">
        <div class="form-check">
          <input type="checkbox" id="{id}" class="form-check-input" formControlName="{ctrl}" />
          <label class="form-check-label" for="{id}">{{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}</label>
        </div>{val}
      </div>"#,
                id = fname,
                ctrl = fname,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                val = validation
            ));
        } else if is_textarea {
            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <textarea class="form-control" formControlName="{ctrl}" rows="4"{max}></textarea>{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                max = max_length_attribute,
                val = validation
            ));
        } else if is_dt || f.ftype == "DateOnly" {
            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <input class="form-control" type="{input_type}" formControlName="{ctrl}" />{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                input_type = if is_dt { "datetime-local" } else { "date" },
                val = validation
            ));
        } else if f.ftype == "TimeOnly" {
            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <input class="form-control" type="time" formControlName="{ctrl}" />{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                val = validation
            ));
        } else if is_num {
            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <input class="form-control" type="number" formControlName="{ctrl}" />{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                val = validation
            ));
        } else {
            modal.push_str(&format!(
                r#"
      <div class="mb-3">
        <label class="form-label">
          {{{{ '::{scr}:{cap}' | abpLocalization }}}}{req}
        </label>
        <input class="form-control" type="text" formControlName="{ctrl}"{max} />{val}
      </div>"#,
                scr = entity,
                cap = field_cs,
                req = req_mark,
                ctrl = fname,
                max = max_length_attribute,
                val = validation
            ));
        }

        let default_val = if is_bool {
            format!("c?.{ctrl} ?? false", ctrl = fname)
        } else if is_dt {
            format!("this.toDatetimeLocal(c?.{ctrl})", ctrl = fname)
        } else if is_num || is_nav {
            format!("c?.{ctrl} ?? null", ctrl = fname)
        } else {
            format!("c?.{ctrl} ?? ''", ctrl = fname)
        };

        let mut validators = Vec::new();
        if required {
            validators.push("Validators.required".to_string());
        }
        if let Some(max) = f.max_length.filter(|_| f.ftype == "string" || is_textarea) {
            validators.push(format!("Validators.maxLength({max})"));
        }
        let entry_line = if validators.is_empty() {
            format!("\n      {ctrl}: [{def}],", ctrl = fname, def = default_val)
        } else {
            format!(
                "\n      {ctrl}: [{def}, [{validators}]],",
                ctrl = fname,
                def = default_val,
                validators = validators.join(", ")
            )
        };
        entries.push_str(&entry_line);

        if f.filterable {
            if is_nav {
                let nav_short = f
                    .navigation
                    .as_deref()
                    .map(last_segment)
                    .unwrap_or_default();
                let nav_lc = to_lower_camel(nav_short);
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label">{{{{ '::{scr}:{cap}' | abpLocalization }}}}</label>
        <div class="input-group">
          <input
            class="form-control"
            type="text"
            [placeholder]="'::Search' | abpLocalization"
            #{nav}FilterSearch
            (input)="on{Nav}Search({nav}FilterSearch.value)"
          />
          <select class="form-select" [(ngModel)]="filter.{ctrl}" (ngModelChange)="applyFilter()">
            <option [ngValue]="null"></option>
            <option *ngFor="let o of {nav}Options" [ngValue]="o.id">{{ o.displayName }}</option>
          </select>
        </div>
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    nav = nav_lc,
                    ctrl = fname,
                    Nav = nav_short
                ));
            } else if f.ftype == "string" || is_textarea {
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label">{{{{'::{scr}:{cap}' | abpLocalization }}}}</label>
        <input class="form-control" [(ngModel)]="filter.{ctrl}" (keyup.enter)="applyFilter()" />
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    ctrl = fname
                ));
            } else if is_num {
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label">{{{{ '::{scr}:{cap}' | abpLocalization }}}}</label>
        <input class="form-control" type="number" [(ngModel)]="filter.{ctrl}" />
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    ctrl = fname
                ));
            } else if is_bool {
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label d-block">{{{{ '::{scr}:{cap}' | abpLocalization }}}}</label>
        <select class="form-select" [(ngModel)]="filter.{ctrl}">
          <option [ngValue]="null"></option>
          <option [ngValue]="true">True</option>
          <option [ngValue]="false">False</option>
        </select>
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    ctrl = fname
                ));
            } else if is_dt || f.ftype == "DateOnly" {
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label">{{{{'::{scr}:{cap}' | abpLocalization }}}}</label>
        <input class="form-control" type="{input_type}" [(ngModel)]="filter.{ctrl}" />
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    ctrl = fname,
                    input_type = if is_dt { "datetime-local" } else { "date" }
                ));
            } else if f.ftype == "TimeOnly" {
                adv.push_str(&format!(
                    r#"
      <div class="col-md-3">
        <label class="form-label">{{{{'::{scr}:{cap}' | abpLocalization }}}}</label>
        <input class="form-control" type="time" [(ngModel)]="filter.{ctrl}" />
      </div>"#,
                    scr = entity,
                    cap = field_cs,
                    ctrl = fname
                ));
            }
        }

        if is_dt {
            payload_mappings.push_str(&format!(
                "      {ctrl}: this.toIsoStringSafe(raw.{ctrl}),\n",
                ctrl = fname
            ));
        } else if is_num {
            payload_mappings.push_str(&format!(
                "      {ctrl}: raw.{ctrl} !== null && raw.{ctrl} !== undefined ? Number(raw.{ctrl}) : null,\n",
                ctrl = fname
            ));
        }

        let (excel_default, excel_append) = if f.filterable {
            let def = if is_bool || is_num || is_dt || is_nav {
                "null"
            } else {
                "''"
            };
            let append_line = if is_dt {
                format!(
                    "    const {ctrl}Iso = this.toIsoStringSafe(input.{ctrl});\n    if ({ctrl}Iso) this.appendIfHasValue(qs, \"{ctrl}\", {ctrl}Iso);\n",
                    ctrl = fname
                )
            } else {
                format!(
                    "    this.appendIfHasValue(qs, \"{ctrl}\", input.{ctrl});\n",
                    ctrl = fname
                )
            };
            (def.to_string(), append_line)
        } else {
            ("null".into(), String::new())
        };

        if f.filterable {
            excel_filter_body.push_str(&format!(
                "      {ctrl}: f.{ctrl} ?? {def},\n",
                ctrl = fname,
                def = excel_default
            ));
            excel_query_appends.push_str(&excel_append);
        }
    }

    excel_query_appends.insert_str(
        0,
        "    this.appendIfHasValue(qs, \"filterText\", input.filterText);\n",
    );

    AngularBlocks {
        datatable_columns: cols.trim().into(),
        modal_form_fields: modal.trim().into(),
        form_group_entries: entries.trim().into(),
        advanced_filter_inputs: adv.trim().into(),
        nav_option_arrays: nav_option_arrays.trim().into(),
        nav_search_subjects: nav_search_subjects.trim().into(),
        nav_search_bindings: nav_search_bindings.trim().into(),
        nav_load_calls: nav_load_calls.trim().into(),
        nav_helpers: nav_helpers.trim().into(),
        payload_mappings: payload_mappings.trim_end().into(),
        excel_filter_body: excel_filter_body.trim_end().into(),
        excel_query_appends: excel_query_appends.trim_end().into(),
        proxy_create_fields: proxy_create_fields.trim_end().into(),
        proxy_dto_fields: proxy_dto_fields.trim_end().into(),
        proxy_update_fields: proxy_update_fields.trim_end().into(),
        proxy_excel_fields: proxy_excel_fields.trim_end().into(),
        proxy_filter_fields: proxy_filter_fields.trim_end().into(),
        proxy_filter_params: proxy_filter_params.join(", "),
        proxy_delete_filter_params: proxy_delete_filter_params.join(", "),
        proxy_excel_params: proxy_excel_params.join(", "),
        proxy_with_nav_fields: proxy_with_nav_fields.trim_end().into(),
        proxy_nav_models: proxy_nav_models.trim_end().into(),
        proxy_lookup_methods: proxy_lookup_methods.trim_end().into(),
    }
}

pub(super) fn generate_angular_ui(
    context: &GenerationContext<'_>,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let project_root = context.project_root;
    let entity = context.entity;
    let fields = context.fields;
    let app_dir = project_root.join("angular").join("src").join("app");
    if !app_dir.exists() {
        return Err(anyhow!(
            "Angular app directory not found: {}",
            app_dir.display()
        ));
    }

    ensure_angular_theme_changer(&app_dir, log)?;

    let name = entity.to_string();
    let kebab = to_kebab(&name);
    let lower = to_lower_camel(&name);
    let plural = pluralize(&name);
    let plural_kebab = to_kebab(&plural);
    let plural_lower = to_lower_camel(&plural);

    let (
        datatable_columns,
        modal_form_fields,
        form_group_entries,
        advanced_filter_inputs,
        nav_lookup_option_arrays,
        nav_lookup_search_subjects,
        nav_lookup_search_bindings,
        nav_lookup_load_calls,
        nav_lookup_helpers,
        payload_mappings,
        excel_filter_body,
        excel_query_appends,
        proxy_create_fields,
        proxy_dto_fields,
        proxy_update_fields,
        proxy_excel_fields,
        proxy_filter_fields,
        proxy_filter_params,
        proxy_delete_filter_params,
        proxy_excel_params,
        proxy_with_nav_fields,
        proxy_nav_models,
        proxy_lookup_methods,
    ) = {
        let parts = angular_blocks(entity, fields);
        (
            parts.datatable_columns,
            parts.modal_form_fields,
            parts.form_group_entries,
            parts.advanced_filter_inputs,
            parts.nav_option_arrays,
            parts.nav_search_subjects,
            parts.nav_search_bindings,
            parts.nav_load_calls,
            parts.nav_helpers,
            parts.payload_mappings,
            parts.excel_filter_body,
            parts.excel_query_appends,
            parts.proxy_create_fields,
            parts.proxy_dto_fields,
            parts.proxy_update_fields,
            parts.proxy_excel_fields,
            parts.proxy_filter_fields,
            parts.proxy_filter_params,
            parts.proxy_delete_filter_params,
            parts.proxy_excel_params,
            parts.proxy_with_nav_fields,
            parts.proxy_nav_models,
            parts.proxy_lookup_methods,
        )
    };

    let mut local_map = mapping.clone();
    local_map.insert("entity_name".into(), name.clone());
    local_map.insert("entity_name_lower".into(), lower.clone());
    local_map.insert("entity_name_kebab".into(), kebab.clone());
    local_map.insert("entity_plural".into(), plural.clone());
    local_map.insert("entity_plural_lower".into(), plural_lower.clone());
    local_map.insert("entity_plural_kebab".into(), plural_kebab.clone());

    local_map.insert("datatable_columns".into(), datatable_columns);
    local_map.insert("modal_form_fields".into(), modal_form_fields);
    local_map.insert("form_group_entries".into(), form_group_entries);
    local_map.insert("advanced_filter_inputs".into(), advanced_filter_inputs);
    local_map.insert("payload_mappings".into(), payload_mappings);
    local_map.insert("excel_filter_body".into(), excel_filter_body);
    local_map.insert("excel_query_appends".into(), excel_query_appends);
    local_map.insert("angular_proxy_create_fields".into(), proxy_create_fields);
    local_map.insert("angular_proxy_dto_fields".into(), proxy_dto_fields);
    local_map.insert("angular_proxy_update_fields".into(), proxy_update_fields);
    local_map.insert("angular_proxy_excel_fields".into(), proxy_excel_fields);
    local_map.insert("angular_proxy_filter_fields".into(), proxy_filter_fields);
    local_map.insert("angular_proxy_filter_params".into(), proxy_filter_params);
    local_map.insert(
        "angular_proxy_delete_filter_params".into(),
        proxy_delete_filter_params,
    );
    local_map.insert("angular_proxy_excel_params".into(), proxy_excel_params);
    local_map.insert(
        "angular_proxy_with_nav_fields".into(),
        proxy_with_nav_fields,
    );
    local_map.insert("angular_proxy_nav_models".into(), proxy_nav_models);
    local_map.insert("angular_proxy_lookup_methods".into(), proxy_lookup_methods);

    local_map.insert(
        "nav_lookup_option_arrays".into(),
        nav_lookup_option_arrays.clone(),
    );
    local_map.insert(
        "nav_lookup_search_subjects".into(),
        nav_lookup_search_subjects,
    );
    local_map.insert(
        "nav_lookup_search_bindings".into(),
        nav_lookup_search_bindings,
    );
    local_map.insert(
        "nav_lookup_load_calls".into(),
        nav_lookup_load_calls.clone(),
    );
    local_map.insert("nav_lookup_helpers".into(), nav_lookup_helpers.clone());
    local_map.insert(
        "mapping.nav_lookup_option_arrays".into(),
        nav_lookup_option_arrays,
    );
    local_map.insert(
        "mapping.nav_lookup_load_calls".into(),
        nav_lookup_load_calls,
    );
    local_map.insert("mapping.nav_lookup_helpers".into(), nav_lookup_helpers);

    local_map.insert(
        "domain_short".into(),
        mapping.get("domain_short").cloned().unwrap_or_default(),
    );

    let out_dir = app_dir.join(&kebab);
    fs::create_dir_all(&out_dir)?;
    for rel in embedded_walk("Web/Angular") {
        let stem = Path::new(&rel)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let out_name = match stem {
            "component.html" => Some(format!("{}.component.html", kebab)),
            "component.ts" => Some(format!("{}.component.ts", kebab)),
            "component.scss" => Some(format!("{}.component.scss", kebab)),
            "component.spec.ts" => Some(format!("{}.component.spec.ts", kebab)),
            "routes.ts" => Some(format!("{}.routes.ts", kebab)),
            _ => None,
        };

        let Some(out_name) = out_name else { continue };

        let content = render_template(&read_tpl_text(&rel)?, &local_map);
        let out_path = out_dir.join(out_name);
        write_text(&out_path, &content)?;
        log(&format!("created (ng): {}", out_path.display()));
    }

    let proxy_dir = app_dir.join("proxy").join(&plural_kebab);
    fs::create_dir_all(&proxy_dir)?;
    let proxy_outputs = [
        ("Web/Angular/proxy.models.ts.tpl", "models.ts".to_string()),
        (
            "Web/Angular/proxy.service.ts.tpl",
            format!("{plural_kebab}.service.ts"),
        ),
        ("Web/Angular/proxy.index.ts.tpl", "index.ts".to_string()),
    ];
    for (template, output) in proxy_outputs {
        let content = render_template(&read_tpl_text(template)?, &local_map);
        let out_path = proxy_dir.join(output);
        write_text(&out_path, &content)?;
        log(&format!("created (ng proxy): {}", out_path.display()));
    }

    let tsconfig_path = project_root.join("angular").join("tsconfig.json");
    if tsconfig_path.exists() {
        let text = fs::read_to_string(&tsconfig_path)?;
        let legacy_alias = Regex::new(&format!(
            r#"(?m)^\s*"@proxy/{}"\s*:\s*\[\s*"src/app/proxy/{}/index\.ts"\s*\],?\s*\r?\n"#,
            regex::escape(&plural_kebab),
            regex::escape(&plural_kebab)
        ))?;
        let cleaned = legacy_alias.replace(&text, "").into_owned();
        if cleaned != text {
            fs::write(&tsconfig_path, &cleaned)?;
            log(&format!(
                "removed obsolete @proxy/{} tsconfig alias; generated features use a relative import",
                plural_kebab
            ));
        }
    }

    let app_routes_path = app_dir.join("app.routes.ts");
    if app_routes_path.exists() {
        let mut text = fs::read_to_string(&app_routes_path)?;

        let snippet = format!(
            "  {{ path: '{path}', loadChildren: () => import('./{feature}/{feature}.routes').then(m => m.{lower}Routes) }},\n",
            path = plural_kebab,
            feature = kebab,
            lower = lower
        );

        if !text.contains(&format!("path: '{}'", plural_kebab))
            && !text.contains(&format!("import('./{}/{}", kebab, kebab))
        {
            let declaration =
                Regex::new(r"export\s+const\s+(?:APP_ROUTES|appRoutes)\s*(?::[^=]+)?=\s*")?;
            if let Some(updated) = insert_into_array(&text, &declaration, &snippet) {
                text = updated;
                fs::write(&app_routes_path, &text)?;
                log(&format!(
                    "patched app.routes.ts by appending route for '{}'",
                    plural_kebab
                ));
            } else {
                log(
                    "app.routes.ts: APP_ROUTES/appRoutes array not found, skipping route injection",
                );
            }
        }
    }

    let route_provider_path = app_dir.join("route.provider.ts");
    if route_provider_path.exists() {
        let mut text = fs::read_to_string(&route_provider_path)?;

        if !text.contains(&format!("path: '/{}'", plural_kebab)) {
            let mut max_order = 0i32;
            for line in text.lines() {
                if let Some(idx) = line.find("order:") {
                    let tail = &line[idx + "order:".len()..];
                    if let Some(num) = tail.trim().split(|c: char| !c.is_ascii_digit()).next()
                        && let Ok(n) = num.parse::<i32>()
                    {
                        max_order = max_order.max(n);
                    }
                }
            }
            let new_order = max_order + 1;

            let snippet = format!(
                "      {{\n        path: '/{path}',\n        name: '::Menu:{menu}',\n        iconClass: 'fa fa-file-alt',\n        order: {order},\n        layout: eLayoutType.application,\n        requiredPolicy: '{domain}.{plural}',\n      }},\n",
                path = plural_kebab,
                menu = plural,
                order = new_order,
                domain = mapping.get("domain_short").cloned().unwrap_or_default(),
                plural = plural
            );

            let declaration = Regex::new(r"routes(?:Service)?\.add\s*\(\s*")?;
            if let Some(updated) = insert_into_array(&text, &declaration, &snippet) {
                text = updated;
                fs::write(&route_provider_path, &text)?;
                log(&format!(
                    "patched route.provider.ts by appending menu '/{}' (order={})",
                    plural_kebab, new_order
                ));
            } else {
                log("route.provider.ts: routes.add([...]) call not found, skipping menu injection");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_support::GenerationFixture;

    fn field(name: &str, field_type: &str) -> Field {
        Field {
            name: name.into(),
            ftype: field_type.into(),
            required: false,
            navigation_display: None,
            max_length: None,
            navigation: None,
            filterable: false,
            show_in_ui: false,
        }
    }

    #[test]
    fn builds_required_filterable_string_controls() {
        let mut title = field("Title", "string");
        title.required = true;
        title.filterable = true;
        title.show_in_ui = true;

        let blocks = angular_blocks("Book", &[title]);

        assert!(blocks.datatable_columns.contains("row.book.title"));
        assert!(blocks.form_group_entries.contains("Validators.required"));
        assert!(blocks.advanced_filter_inputs.contains("filter.title"));
        assert!(blocks.excel_filter_body.contains("title: f.title ?? ''"));
    }

    #[test]
    fn builds_typed_payload_mappings() {
        let published_at = field("PublishedAt", "DateTime");
        let price = field("Price", "decimal");
        let status = field("Status", "enum");

        let blocks = angular_blocks("Book", &[published_at, price, status]);

        assert!(
            blocks
                .payload_mappings
                .contains("publishedAt: this.toIsoStringSafe(raw.publishedAt)")
        );
        assert!(blocks.payload_mappings.contains("price: raw.price"));
        assert!(blocks.payload_mappings.contains("Number(raw.price)"));
        assert!(blocks.payload_mappings.contains("status: raw.status"));
        assert!(blocks.proxy_dto_fields.contains("status?: number | null"));
    }

    #[test]
    fn builds_navigation_lookup_once() {
        let mut author = field("AuthorId", "Guid");
        author.navigation = Some("Acme.BookStore.Authors.Author".into());
        author.navigation_display = Some("Title".into());
        author.filterable = true;
        author.show_in_ui = true;

        let blocks = angular_blocks("Book", &[author]);

        assert!(blocks.nav_option_arrays.contains("authorOptions"));
        assert_eq!(blocks.nav_load_calls.matches("onAuthorSearch").count(), 1);
        assert_eq!(
            blocks
                .nav_search_bindings
                .matches("getAuthorLookup")
                .count(),
            1
        );
        assert!(blocks.nav_search_bindings.contains("debounceTime(300)"));
        assert!(blocks.nav_search_bindings.contains("res.items ?? []"));
        assert!(blocks.proxy_with_nav_fields.contains("author?: AuthorDto"));
        assert!(blocks.proxy_nav_models.contains("title?: string | null"));
        assert!(blocks.datatable_columns.contains("row.author?.title"));
        assert!(
            blocks
                .proxy_lookup_methods
                .contains("/api/app/books/author-lookup")
        );
        assert!(
            blocks
                .modal_form_fields
                .contains("formControlName=\"authorId\"")
        );
    }

    #[test]
    fn renders_the_complete_angular_feature_manifest() {
        let fixture = GenerationFixture::new();
        let context = fixture.context();
        let mapping = fixture.mapping();
        let app_dir = fixture.root().join("angular/src/app");
        fs::create_dir_all(&app_dir).expect("Angular fixture");
        fs::write(
            app_dir.join("app.config.ts"),
            "export const appConfig = { providers: [] };\n",
        )
        .expect("Angular startup fixture");
        let mut messages = Vec::new();

        generate_angular_ui(&context, &mapping, &mut |message| {
            messages.push(message.to_owned());
        })
        .expect("Angular generation");

        let output = fixture.root().join("angular/src/app/book");
        for file in [
            "book.component.html",
            "book.component.ts",
            "book.component.scss",
            "book.component.spec.ts",
            "book.routes.ts",
        ] {
            assert!(
                output.join(file).is_file(),
                "missing Angular output: {file}"
            );
        }

        let proxy_output = fixture.root().join("angular/src/app/proxy/books");
        for file in ["models.ts", "books.service.ts", "index.ts"] {
            assert!(
                proxy_output.join(file).is_file(),
                "missing Angular proxy output: {file}"
            );
        }

        assert!(
            app_dir
                .join("theme-changer/theme-changer.component.ts")
                .is_file()
        );
        assert!(
            app_dir
                .join("theme-changer/theme-changer.provider.ts")
                .is_file()
        );
        let app_config = fs::read_to_string(app_dir.join("app.config.ts")).expect("app config");
        assert_eq!(app_config.matches("provideThemeChanger()").count(), 1);

        let component =
            fs::read_to_string(output.join("book.component.html")).expect("Angular component");
        assert!(component.contains("formControlName=\"title\""));
        assert!(component.contains("(click)=\"deleteSelected()\""));
        assert!(component.contains("[selectionType]=\"selectionType.checkbox\""));
        assert!(component.contains("[list]=\"list\""));
        assert!(component.contains("[externalPaging]=\"true\""));
        assert!(component.contains("[footerHeight]=\"50\""));
        assert!(component.contains("<ngx-datatable-footer>"));
        assert!(component.contains("(ngModelChange)=\"changePageSize()\""));
        assert!(component.contains("'::PagerInfo'"));
        assert!(!component.contains("${form_group_entries}"));

        let component_class =
            fs::read_to_string(output.join("book.component.ts")).expect("Angular component class");
        assert!(component_class.contains("readonly pageSizes = [10, 25, 50, 100]"));
        assert!(component_class.contains("goToPage(page: number): void"));

        let models =
            fs::read_to_string(proxy_output.join("models.ts")).expect("Angular proxy models");
        let service = fs::read_to_string(proxy_output.join("books.service.ts"))
            .expect("Angular proxy service");
        assert!(models.contains("export interface BookCreateDto"));
        assert!(models.contains("title: string;"));
        assert!(service.contains("url: '/api/app/books'"));
        assert!(!models.contains("${angular_proxy"));
        assert!(!service.contains("${angular_proxy"));
        assert_eq!(messages.len(), 11);
    }

    #[test]
    fn inserts_routes_into_current_and_legacy_app_route_names() {
        let declaration =
            Regex::new(r"export\s+const\s+(?:APP_ROUTES|appRoutes)\s*(?::[^=]+)?=\s*")
                .expect("route declaration regex");
        for source in [
            "export const APP_ROUTES: Routes = [\n  { path: '' },\n];\n",
            "export const appRoutes = [{ path: '' }];\n",
        ] {
            let updated = insert_into_array(source, &declaration, "  { path: 'books' },\n")
                .expect("route array");
            assert_eq!(updated.matches("path: 'books'").count(), 1);
            assert!(updated.ends_with("];\n"));
        }
    }

    #[test]
    fn registers_theme_changer_in_legacy_ng_module_idempotently() {
        let fixture = tempfile::tempdir().expect("fixture");
        let app_dir = fixture.path().join("src/app");
        fs::create_dir_all(&app_dir).expect("app directory");
        fs::write(
            app_dir.join("app.module.ts"),
            "@NgModule({\n  imports: [],\n  providers: [],\n})\nexport class AppModule {}\n",
        )
        .expect("legacy module");

        ensure_angular_theme_changer(&app_dir, &mut |_| {}).expect("first registration");
        ensure_angular_theme_changer(&app_dir, &mut |_| {}).expect("second registration");

        let module = fs::read_to_string(app_dir.join("app.module.ts")).expect("module");
        assert_eq!(module.matches("THEME_CHANGER_PROVIDER,").count(), 1);
        assert_eq!(
            module.matches("import { THEME_CHANGER_PROVIDER }").count(),
            1
        );
    }
}

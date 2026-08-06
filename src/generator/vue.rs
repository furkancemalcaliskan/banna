use super::GenerationContext;
use crate::helpers::last_segment;
use crate::models::Field;
use crate::templates::{embedded_walk, read_tpl_text};
use crate::utils::{pluralize, render_template, to_kebab, to_lower_camel, write_text};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/* ------------------------------ VUE HELPERS ------------------------- */

fn vue_ui_blocks(
    entity: &str,
    fields: &[Field],
    mapping: &HashMap<String, String>,
) -> HashMap<String, String> {
    use std::collections::HashMap;
    let mut out = HashMap::new();

    let entity_lower = to_lower_camel(entity);
    let entity_plural = pluralize(entity);
    let _entity_plural_lower = to_lower_camel(&entity_plural);
    let _domain_short = mapping.get("domain_short").cloned().unwrap_or_default();

    let label_key = |f: &Field| format!("{}:{}", entity, f.name);

    let mut form_create = String::new();
    let mut form_edit = String::new();
    let mut form_detail = String::new();

    let mut create_model = Vec::new();
    let mut edit_model = Vec::new();
    let mut detail_model = Vec::new();

    let mut create_cans = Vec::new();
    let mut edit_cans = Vec::new();

    let mut create_payload_pairs = Vec::new();
    let mut edit_payload_pairs = Vec::new();

    let mut create_support_refs = String::new();
    let mut edit_support_refs = String::new();
    let mut detail_support_refs = String::new();

    let mut create_lookup_fns = String::new();
    let mut edit_lookup_fns = String::new();

    let mut create_onmounted = String::new();
    let mut create_return_refs = Vec::new();
    let mut edit_return_refs = Vec::new();
    let mut detail_return_refs = Vec::new();

    let mut edit_load_map = String::new();
    let mut edit_prime_lookups = String::new();

    let mut detail_load_map = String::new();

    let mut vue_index_support_refs = String::new();
    let mut vue_index_lookup_functions = String::new();
    let mut vue_index_return_refs = Vec::new();
    let mut vue_index_onmounted_fetches = String::new();

    for f in fields {
        let label_l = if f.required {
            format!("@L[\"{}\"].Value *", label_key(f))
        } else {
            format!("@L[\"{}\"].Value", label_key(f))
        };
        let nm = to_lower_camel(&f.name);

        let (def_val, can_check) = match f.ftype.as_str() {
            "string" | "String" | "textarea" => ("\"\"".to_string(), format!("!!model.{nm}")),
            "Guid" => ("null".to_string(), format!("!!model.{nm}")),
            "int" | "Int32" | "long" | "Int64" | "decimal" => (
                "null".to_string(),
                format!("model.{nm} !== null && model.{nm} !== undefined"),
            ),
            "bool" | "Boolean" => ("false".to_string(), "true".to_string()),
            "DateTime" => ("null".to_string(), format!("!!model.{nm}")),
            _ => ("null".to_string(), format!("!!model.{nm}")),
        };

        let model_line = format!("{nm}: {def_val}");
        create_model.push(model_line.clone());
        edit_model.push(model_line.clone());
        detail_model.push(model_line.clone());

        if f.required {
            create_cans.push(can_check.clone());
            edit_cans.push(can_check);
        }

        let payload_pair = format!("{nm}: model.{nm}");
        create_payload_pairs.push(payload_pair.clone());
        edit_payload_pairs.push(payload_pair);

        let mut _mapped = false;

        match f.ftype.as_str() {
            "string" | "String" => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model="model.{nm}" type="text" class="form-control" />
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model="model."#, r#":value="model."#)
                        .replace(r#" type="text""#, "")
                        .replace(r#" />"#, r#" disabled />"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? \"\";\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? \"\";\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            "textarea" => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <textarea v-model="model.{nm}" class="form-control" rows="3"></textarea>
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(&format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <div class="form-control-plaintext">{{{{ model.{nm} }}}}</div>
</div>
"#,
                    label = label_l,
                    nm = nm
                ));
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? \"\";\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? \"\";\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            "decimal" => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model.number="model.{nm}" type="number" step="0.01" class="form-control" />
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model.number="model."#, r#":value="model."#)
                        .replace(r#"type="number""#, "")
                        .replace(r#" />"#, r#" disabled />"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            "Guid" => {
                if let Some(nav) = &f.navigation {
                    let nav_class = last_segment(nav);
                    let list_ref = format!("{}s", to_lower_camel(nav_class));
                    let rline = format!("const {list_ref} = ref([]);");
                    if !create_support_refs.contains(&rline) {
                        create_support_refs.push_str(&format!("{rline}\n"));
                    }
                    if !edit_support_refs.contains(&rline) {
                        edit_support_refs.push_str(&format!("{rline}\n"));
                    }
                    if !detail_support_refs.contains(&rline) {
                        detail_support_refs.push_str(&format!("{rline}\n"));
                    }
                    create_return_refs.push(list_ref.clone());
                    edit_return_refs.push(list_ref.clone());
                    detail_return_refs.push(list_ref.clone());

                    if !vue_index_support_refs.contains(&rline) {
                        vue_index_support_refs.push_str(&format!("{rline}\n"));
                    }
                    if !vue_index_return_refs.contains(&list_ref) {
                        vue_index_return_refs.push(list_ref.clone());
                    }

                    let fetch_fn = format!(
                        r#"let {list}RequestId = 0;
async function fetch{cls}Lookup(term) {{
  const requestId = ++{list}RequestId;
  const res = await {svc}Service.get{cls}Lookup({{ filter: term || "", maxResultCount: 10 }});
  if (requestId === {list}RequestId) {{
    {list}.value = res?.items ?? [];
  }}
}}
"#,
                        cls = nav_class,
                        list = list_ref,
                        svc = entity_lower
                    );

                    if !create_lookup_fns.contains(&format!("fetch{nav_class}Lookup")) {
                        create_lookup_fns.push_str(&fetch_fn);
                    }
                    if !edit_lookup_fns.contains(&format!("fetch{nav_class}Lookup")) {
                        edit_lookup_fns.push_str(&fetch_fn);
                    }
                    if !vue_index_lookup_functions.contains(&format!("fetch{nav_class}Lookup")) {
                        vue_index_lookup_functions.push_str(&fetch_fn);
                    }
                    let fetch_fn_name = format!("fetch{nav_class}Lookup");
                    if !create_return_refs.contains(&fetch_fn_name) {
                        create_return_refs.push(fetch_fn_name.clone());
                    }
                    if !edit_return_refs.contains(&fetch_fn_name) {
                        edit_return_refs.push(fetch_fn_name.clone());
                    }
                    if !vue_index_return_refs.contains(&fetch_fn_name) {
                        vue_index_return_refs.push(fetch_fn_name.clone());
                    }

                    let prime = format!("await fetch{nav_class}Lookup(\"\");\n");
                    if !create_onmounted.contains(&prime) {
                        create_onmounted.push_str(&prime);
                    }
                    if !vue_index_onmounted_fetches.contains(&prime) {
                        vue_index_onmounted_fetches.push_str(&prime);
                    }

                    let block = format!(
                        r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <v-select
    v-model="model.{nm}"
    :options="{list}"
    label="displayName"
    :reduce="a => a.id"
    @@search="async (term, loading) => {{ loading?.(true); try {{ await fetch{cls}Lookup(term); }} finally {{ loading?.(false); }} }}"
    :filterable="false"
    :clearable="{clr}"
    :placeholder="'@L["Search"].Value'"
  />
</div>
"#,
                        label = label_l,
                        list = list_ref,
                        cls = nav_class,
                        clr = if f.required { "false" } else { "true" },
                    );
                    form_create.push_str(&block);
                    form_edit.push_str(&block);
                    form_detail.push_str(
                        &block
                            .replace(r#":clearable="{clr}""#, r#":clearable="false""#)
                            .replace(r#"/>"#, " disabled\n  />"),
                    );

                    let nav_prop_lower = to_lower_camel(nav_class);
                    let nav_display_lower =
                        to_lower_camel(f.navigation_display.as_deref().unwrap_or("Name"));
                    edit_load_map.push_str(&format!(
                        "model.{nm} = dto.{el}.{nm} ?? null;\n",
                        el = entity_lower,
                        nm = nm
                    ));
                    edit_prime_lookups.push_str(&format!(
                        "if (dto.{nav}) {{ {list}.value = [{{ id: dto.{nav}.id, displayName: dto.{nav}.{display} }}]; }} else {{ await fetch{cls}Lookup(\"\"); }}\n",
                        nav = nav_prop_lower,
                        list = list_ref,
                        cls = nav_class,
                        display = nav_display_lower,
                    ));
                    detail_load_map.push_str(&format!(
                        "model.{nm} = dto.{el}.{nm} ?? null;\n{list}.value = dto.{nav} ? [{{ id: dto.{nav}.id, displayName: dto.{nav}.{display} }}] : [];\n",
                        el = entity_lower,
                        list = list_ref,
                        nav = nav_prop_lower,
                        nm = nm,
                        display = nav_display_lower,
                    ));
                    _mapped = true;
                } else {
                    let block = format!(
                        r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model="model.{nm}" type="text" class="form-control" />
</div>
"#,
                        label = label_l
                    );
                    form_create.push_str(&block);
                    form_edit.push_str(&block);
                    form_detail.push_str(
                        &block
                            .replace(r#"v-model="model."#, r#":value="model."#)
                            .replace(r#"type="text""#, "")
                            .replace(r#" />"#, r#" disabled />"#),
                    );
                    edit_load_map.push_str(&format!(
                        "model.{nm} = dto.{el}.{nm} ?? null;\n",
                        el = entity_lower,
                        nm = nm
                    ));
                    detail_load_map.push_str(&format!(
                        "model.{nm} = dto.{el}.{nm} ?? null;\n",
                        el = entity_lower,
                        nm = nm
                    ));
                    _mapped = true;
                }
            }

            "int" | "Int32" | "long" | "Int64" => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model.number="model.{nm}" type="number" class="form-control" />
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model.number="model."#, r#":value="model."#)
                        .replace(r#"type="number""#, "")
                        .replace(r#" />"#, r#" disabled />"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            "bool" | "Boolean" => {
                let block = format!(
                    r#"<div class="form-check mb-3">
  <input class="form-check-input" type="checkbox" v-model="model.{nm}" id="{nm}_chk" />
  <label class="form-check-label" for="{nm}_chk">{label}</label>
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model="model."#, r#":checked="model."#)
                        .replace(r#"type="checkbox""#, r#"type="checkbox" disabled"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = !!dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = !!dto.{el}.{nm};\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            "DateTime" => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model="model.{nm}" type="datetime-local" class="form-control" />
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model="model."#, r#":value="model."#)
                        .replace(r#"type="datetime-local""#, "")
                        .replace(r#" />"#, r#" disabled />"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? null;\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? null;\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }

            _ => {
                let block = format!(
                    r#"<div class="mb-3">
  <label class="form-label">{label}</label>
  <input v-model="model.{nm}" type="text" class="form-control" />
</div>
"#,
                    label = label_l
                );
                form_create.push_str(&block);
                form_edit.push_str(&block);
                form_detail.push_str(
                    &block
                        .replace(r#"v-model="model."#, r#":value="model."#)
                        .replace(r#"type="text""#, "")
                        .replace(r#" />"#, r#" disabled />"#),
                );
                edit_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? null;\n",
                    el = entity_lower,
                    nm = nm
                ));
                detail_load_map.push_str(&format!(
                    "model.{nm} = dto.{el}.{nm} ?? null;\n",
                    el = entity_lower,
                    nm = nm
                ));
                _mapped = true;
            }
        }

        if !_mapped && f.navigation.is_none() {
            edit_load_map.push_str(&format!(
                "model.{nm} = dto.{el}.{nm};\n",
                el = entity_lower,
                nm = nm
            ));
            detail_load_map.push_str(&format!(
                "model.{nm} = dto.{el}.{nm};\n",
                el = entity_lower,
                nm = nm
            ));
        }
    }

    edit_model.push("concurrencyStamp: null".into());
    edit_payload_pairs.push("concurrencyStamp: model.concurrencyStamp".into());
    edit_load_map.push_str(&format!(
        "model.concurrencyStamp = dto.{el}.concurrencyStamp ?? null;\n",
        el = entity_lower
    ));

    let can_expr = |xs: &Vec<String>| {
        if xs.is_empty() {
            "true".into()
        } else {
            xs.join(" && ")
        }
    };
    let can_create = can_expr(&create_cans);
    let can_edit = can_expr(&edit_cans);

    let join_refs = |v: &Vec<String>| {
        if v.is_empty() {
            "".into()
        } else {
            format!(", {}", v.join(", "))
        }
    };

    let mut vue_index_filter_fields = Vec::new();
    let mut vue_index_clear_defaults = vec![r#"filterText: """#.to_string()];
    let mut vue_index_watch_filters = String::new();

    let mut index_js_string_query_pairs = String::new();
    let mut index_js_nav_query_pairs = String::new();

    let mut web_datatable_columns = String::new();
    let mut thead_headers = String::new();

    for f in fields {
        if f.show_in_ui {
            let th = if let Some(navigation) = f.navigation.as_deref().filter(|_| f.ftype == "Guid")
            {
                let nav_cls = last_segment(navigation);
                format!(r#"<th>@L["{}"].Value</th>"#, nav_cls)
            } else {
                format!(r#"<th>@L["{}"].Value</th>"#, label_key(f))
            };
            thead_headers.push_str(&format!("            {}\n", th));
        }

        if f.show_in_ui {
            if let Some(navigation) = f.navigation.as_deref().filter(|_| f.ftype == "Guid") {
                let nav_lower = to_lower_camel(last_segment(navigation));
                let display_lower =
                    to_lower_camel(f.navigation_display.as_deref().unwrap_or("Name"));
                web_datatable_columns.push_str(&format!(
                    "        {{ data: \"{nav_lower}.{display_lower}\", defaultContent: \"\", render: DataTable.render.text() }},\n"
                ));
            } else {
                web_datatable_columns.push_str(&format!(
                    "        {{ data: \"{}.{}\", defaultContent: \"\", render: DataTable.render.text() }},\n",
                    entity_lower,
                    to_lower_camel(&f.name)
                ));
            }
        }

        if f.filterable {
            let key = to_lower_camel(&f.name);
            match f.ftype.as_str() {
                "string" | "String" | "textarea" => {
                    vue_index_filter_fields.push(format!("{}: null", key));
                    vue_index_clear_defaults.push(format!("{}: null", key));
                    vue_index_watch_filters.push_str(&format!(
                        "    watch(() => filter.value.{k}, scheduleFilterReload);\n",
                        k = key
                    ));
                    index_js_string_query_pairs.push_str(&format!(
                        "            {{ name: \"{k}\", value: filter.value.{k} }},\n",
                        k = key
                    ));
                }
                "Guid" => {
                    vue_index_filter_fields.push(format!("{}: null", key));
                    vue_index_clear_defaults.push(format!("{}: null", key));
                    vue_index_watch_filters.push_str(&format!(
                        "    watch(() => filter.value.{k}, scheduleFilterReload);\n",
                        k = key
                    ));
                    index_js_nav_query_pairs.push_str(&format!(
                        "            {{ name: \"{k}\", value: filter.value.{k} }},\n",
                        k = key
                    ));
                    if let Some(nav) = &f.navigation {
                        let cls = last_segment(nav);
                        let prime = format!("await fetch{cls}Lookup(\"\");\n");
                        if !vue_index_onmounted_fetches.contains(&prime) {
                            vue_index_onmounted_fetches.push_str(&prime);
                        }
                    }
                }
                "int" | "Int32" | "long" | "Int64" | "decimal" | "bool" | "Boolean"
                | "DateTime" => {
                    vue_index_filter_fields.push(format!("{}: null", key));
                    vue_index_clear_defaults.push(format!("{}: null", key));
                    vue_index_watch_filters.push_str(&format!(
                        "    watch(() => filter.value.{k}, scheduleFilterReload);\n",
                        k = key
                    ));
                    index_js_string_query_pairs.push_str(&format!(
                        "            {{ name: \"{k}\", value: filter.value.{k} }},\n",
                        k = key
                    ));
                }
                _ => {}
            }
        }
    }

    let vue_index_bulk_delete_block = format!(
        r#"
      dataTable.value.on("xhr", function() {{
        const table = document.getElementById("{plural}Table");
        const selectAll = document.getElementById("select_all");
        if (selectAll) {{ selectAll.indeterminate = false; selectAll.checked = false; }}
      }});"#,
        plural = entity_plural
    );

    let mut adv_inputs = String::new();
    for f in fields {
        if !f.filterable {
            continue;
        }
        let key = to_lower_camel(&f.name);
        match f.ftype.as_str() {
            "Guid" if f.navigation.is_some() => {
                let Some(nav_type) = f.navigation.as_deref().map(last_segment) else {
                    continue;
                };
                let list_ref = format!("{}s", to_lower_camel(nav_type));
                let label = format!(r#"@L["{}"]"#, label_key(f));
                adv_inputs.push_str(&format!(
r#"
          <div class="col-3">
            <div class="mb-3">
              <label class="form-label">{label}.Value</label>
              <v-select
                v-model="filter.{key}"
                :options="{list}"
                :reduce="it => it.id"
                label="displayName"
                id="{key}"
                @@search="async (term, loading) => {{ loading?.(true); try {{ await fetch{cls}Lookup(term); }} finally {{ loading?.(false); }} }}"
                :filterable="false"
              />
            </div>
          </div>
"#, label = label, key = key, list = list_ref, cls = nav_type));
            }
            "decimal" => {
                let label = format!(r#"@L["{}"]"#, label_key(f));
                adv_inputs.push_str(&format!(
r#"
          <div class="col-3">
            <div class="mb-3">
              <label class="form-label">{label}.Value</label>
              <input id="{key}" type="number" step="0.01" class="form-control" v-model.number="filter.{key}" @@keyup.enter="onGetFilteredData" />
            </div>
          </div>
"#, label = label, key = key));
            }
            "string" | "String" | "textarea" => {
                let label = format!(r#"@L["{}"]"#, label_key(f));
                adv_inputs.push_str(&format!(
r#"
          <div class="col-3">
            <div class="mb-3">
              <label class="form-label">{label}.Value</label>
              <input id="{key}" type="text" class="form-control" v-model="filter.{key}" @@keyup.enter="onGetFilteredData" />
            </div>
          </div>
"#, label = label, key = key));
            }
            _ => {
                let label = format!(r#"@L["{}"]"#, label_key(f));
                adv_inputs.push_str(&format!(
r#"
          <div class="col-3">
            <div class="mb-3">
              <label class="form-label">{label}.Value</label>
              <input id="{key}" type="text" class="form-control" v-model="filter.{key}" @@keyup.enter="onGetFilteredData" />
            </div>
          </div>
"#, label = label, key = key));
            }
        }
    }

    out.insert("web_form_fields_create_vue".into(), form_create);
    out.insert("web_form_fields_edit_vue".into(), form_edit);
    out.insert("web_form_fields_detail_vue".into(), form_detail);

    out.insert(
        "vue_create_model_fields".into(),
        create_model.join(",\n      "),
    );
    out.insert("vue_edit_model_fields".into(), edit_model.join(",\n      "));
    out.insert(
        "vue_detail_model_fields".into(),
        detail_model.join(",\n      "),
    );

    out.insert(
        "vue_create_cansave_logic".into(),
        format!("return {};", can_create),
    );
    out.insert("vue_create_cansave_expression".into(), can_create);
    out.insert(
        "vue_edit_cansave_logic".into(),
        format!("return {};", can_edit),
    );
    out.insert("vue_edit_cansave_expression".into(), can_edit);

    out.insert(
        "vue_create_payload".into(),
        format!("{{ {} }}", create_payload_pairs.join(", ")),
    );
    out.insert(
        "vue_edit_payload".into(),
        format!("{{ {} }}", edit_payload_pairs.join(", ")),
    );

    out.insert("vue_create_support_refs".into(), create_support_refs);
    out.insert("vue_edit_support_refs".into(), edit_support_refs);
    out.insert("vue_detail_support_refs".into(), detail_support_refs);

    out.insert("vue_create_lookup_functions".into(), create_lookup_fns);
    out.insert("vue_edit_lookup_functions".into(), edit_lookup_fns);

    out.insert("vue_create_onmounted".into(), create_onmounted);
    out.insert(
        "vue_create_return_refs".into(),
        join_refs(&create_return_refs),
    );
    out.insert("vue_edit_return_refs".into(), join_refs(&edit_return_refs));
    out.insert(
        "vue_detail_return_refs".into(),
        join_refs(&detail_return_refs),
    );

    out.insert("vue_edit_load_mapping".into(), edit_load_map);
    out.insert(
        "vue_edit_prime_lookups_if_needed".into(),
        edit_prime_lookups,
    );
    out.insert("vue_detail_load_mapping".into(), detail_load_map);

    out.insert(
        "vue_index_filter_fields".into(),
        vue_index_filter_fields.join(",\n      "),
    );
    out.insert(
        "vue_index_clear_filter_defaults".into(),
        vue_index_clear_defaults.join(",\n        "),
    );
    out.insert("vue_index_watch_filters".into(), vue_index_watch_filters);

    out.insert(
        "web_datatable_columns".into(),
        web_datatable_columns.trim_end().into(),
    );
    out.insert(
        "vue_index_table_headers".into(),
        thead_headers.trim_end().into(),
    );

    out.insert("vue_index_default_order_col".into(), "1".into());
    out.insert(
        "vue_index_bulk_delete_block".into(),
        vue_index_bulk_delete_block,
    );

    out.insert(
        "index_js_string_query_pairs".into(),
        index_js_string_query_pairs.trim_end().into(),
    );
    out.insert(
        "index_js_nav_query_pairs".into(),
        index_js_nav_query_pairs.trim_end().into(),
    );

    out.insert("vue_index_support_refs".into(), vue_index_support_refs);
    out.insert(
        "vue_index_lookup_functions".into(),
        vue_index_lookup_functions,
    );
    out.insert(
        "vue_index_onmounted_fetches".into(),
        vue_index_onmounted_fetches,
    );
    out.insert(
        "vue_index_return_refs".into(),
        join_refs(&vue_index_return_refs),
    );

    out.insert(
        "vue_index_advanced_filter_inputs".into(),
        adv_inputs.trim_end().into(),
    );

    out
}

pub(super) fn generate_mvc_vue(
    context: &GenerationContext<'_>,
    mapping: &HashMap<String, String>,
) -> Result<()> {
    let out_root = &context.out_root;
    let domain = context.domain;
    let entity = context.entity;
    let fields = context.fields;
    let web_files = embedded_walk("Web/MVC/VueComponent");
    let out_dir = out_root.join(format!("{}.Web", domain)).join("Pages").join(
        mapping
            .get("entity_plural")
            .cloned()
            .unwrap_or_else(|| pluralize(entity)),
    );
    fs::create_dir_all(&out_dir)?;

    let mut local_map = mapping.clone();

    if !local_map.contains_key("entity_plural_kebab") {
        let ep = local_map.get("entity_plural").cloned().unwrap_or_default();
        let kebab = to_kebab(&ep);
        local_map.insert("entity_plural_kebab".into(), kebab);
    }

    let vue_map = vue_ui_blocks(entity, fields, &local_map);
    for (k, v) in vue_map {
        local_map.insert(k, v);
    }

    let rename = std::collections::HashMap::from([
        ("Index.cshtml", "Index.cshtml".to_string()),
        ("Index.cshtml.cs", "Index.cshtml.cs".to_string()),
        ("index.js", "index.js".to_string()),
        ("Index.css", "Index.css".to_string()),
        ("CreateModal.cshtml", "CreateModal.cshtml".to_string()),
        ("CreateModal.cshtml.cs", "CreateModal.cshtml.cs".to_string()),
        ("createModal.js", "createModal.js".to_string()),
        ("CreateModal.css", "CreateModal.css".to_string()),
        ("EditModal.cshtml", "EditModal.cshtml".to_string()),
        ("EditModal.cshtml.cs", "EditModal.cshtml.cs".to_string()),
        ("editModal.js", "editModal.js".to_string()),
        ("EditModal.css", "EditModal.css".to_string()),
        ("DetailModal.cshtml", "DetailModal.cshtml".to_string()),
        ("DetailModal.cshtml.cs", "DetailModal.cshtml.cs".to_string()),
        ("detailModal.js", "detailModal.js".to_string()),
        ("DetailModal.css", "DetailModal.css".to_string()),
    ]);

    for rel in web_files {
        let stem = Path::new(&rel)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let out_name = rename
            .get(stem)
            .cloned()
            .unwrap_or_else(|| stem.to_string());

        let tpl = read_tpl_text(&rel)?;
        let content = render_template(&tpl, &local_map);
        write_text(&out_dir.join(out_name), &content)?;
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
    fn builds_required_string_form_and_payload() {
        let mut title = field("Title", "string");
        title.required = true;
        title.show_in_ui = true;

        let blocks = vue_ui_blocks("Book", &[title], &HashMap::new());

        assert!(blocks["web_form_fields_create_vue"].contains("v-model=\"model.title\""));
        assert_eq!(blocks["vue_create_cansave_logic"], "return !!model.title;");
        assert!(blocks["vue_create_payload"].contains("title: model.title"));
        assert!(blocks["web_datatable_columns"].contains("book.title"));
    }

    #[test]
    fn builds_navigation_lookup_and_filter_fragments() {
        let mut author = field("AuthorId", "Guid");
        author.navigation = Some("Acme.BookStore.Authors.Author".into());
        author.navigation_display = Some("Title".into());
        author.filterable = true;
        author.show_in_ui = true;

        let blocks = vue_ui_blocks("Book", &[author], &HashMap::new());

        assert!(blocks["vue_create_lookup_functions"].contains("fetchAuthorLookup"));
        assert!(blocks["vue_create_lookup_functions"].contains("bookService.getAuthorLookup"));
        assert!(blocks["vue_index_lookup_functions"].contains("fetchAuthorLookup"));
        assert!(blocks["index_js_nav_query_pairs"].contains("authorId"));
        assert!(blocks["vue_index_advanced_filter_inputs"].contains("filter.authorId"));
        assert!(blocks["vue_edit_prime_lookups_if_needed"].contains("dto.author.title"));
        assert!(blocks["web_datatable_columns"].contains("author.title"));
    }

    #[test]
    fn builds_typed_filter_defaults() {
        let mut price = field("Price", "decimal");
        price.filterable = true;
        let mut active = field("Active", "bool");
        active.filterable = true;

        let blocks = vue_ui_blocks("Book", &[price, active], &HashMap::new());

        assert!(blocks["vue_index_filter_fields"].contains("price: null"));
        assert!(blocks["vue_index_filter_fields"].contains("active: null"));
        assert!(!blocks["vue_index_filter_fields"].contains("filterText"));
        assert!(blocks["vue_index_watch_filters"].contains("scheduleFilterReload"));
        assert!(blocks["index_js_string_query_pairs"].contains("price"));
        assert!(blocks["index_js_string_query_pairs"].contains("active"));
    }

    #[test]
    fn renders_vue_components_with_field_fragments() {
        let fixture = GenerationFixture::new();
        let context = fixture.context();
        let mapping = fixture.mapping();

        generate_mvc_vue(&context, &mapping).expect("Vue generation");

        let output = context.out_root.join("Acme.BookStore.Web/Pages/Books");
        for file in [
            "Index.cshtml",
            "index.js",
            "CreateModal.cshtml",
            "createModal.js",
            "EditModal.cshtml",
            "editModal.js",
            "DetailModal.cshtml",
            "detailModal.js",
        ] {
            assert!(output.join(file).is_file(), "missing Vue output: {file}");
        }

        let create = fs::read_to_string(output.join("createModal.js")).expect("Vue create JS");
        assert!(create.contains("title: \"\""));
        assert!(create.contains("!isSaving.value && (!!model.title)"));
        assert!(!create.contains("close,  }"));
        assert!(!create.contains("${vue_"));

        let index = fs::read_to_string(output.join("index.js")).expect("Vue index JS");
        assert_eq!(index.matches("filterText: \"\"").count(), 2);
        assert!(index.contains("render: DataTable.render.text()"));
        assert!(index.contains("setTimeout(reloadTable, 300)"));
    }
}

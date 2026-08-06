use super::GenerationContext;
use crate::helpers::last_segment;
use crate::models::Field;
use crate::templates::read_tpl_text;
use crate::utils::{pluralize, render_template, to_kebab, to_lower_camel, write_text};
use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(super) fn generate_mobile_react_native(
    context: &GenerationContext<'_>,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let project_root = context.project_root;
    let fields = context.fields;
    let rn_root = project_root.join("react-native");
    if !rn_root.exists() {
        log(&format!(
            "React Native directory not found at {}, skipping mobile UI generation",
            rn_root.display()
        ));
        return Ok(());
    }

    let entity = mapping
        .get("entity_name")
        .cloned()
        .or_else(|| mapping.get("entity").cloned())
        .unwrap_or_default();
    let entity_plural = mapping
        .get("entity_plural")
        .cloned()
        .unwrap_or_else(|| pluralize(&entity));
    let entity_name_lower = mapping
        .get("entity_name_lower")
        .cloned()
        .unwrap_or_else(|| to_lower_camel(&entity));
    let entity_plural_lower = mapping
        .get("entity_plural_lower")
        .cloned()
        .unwrap_or_else(|| to_lower_camel(&entity_plural));
    let entity_plural_kebab = mapping
        .get("entity_plural_kebab")
        .cloned()
        .unwrap_or_else(|| to_kebab(&entity_plural));
    let domain_short = mapping
        .get("domain_short")
        .cloned()
        .unwrap_or_else(|| "App".into());

    let mut mobile_map = mapping.clone();
    mobile_map.insert("entity_name".into(), entity.clone());
    mobile_map.insert("entity_name_lower".into(), entity_name_lower);
    mobile_map.insert("entity_plural".into(), entity_plural.clone());
    mobile_map.insert("entity_plural_lower".into(), entity_plural_lower);
    mobile_map.insert("entity_plural_kebab".into(), entity_plural_kebab);
    mobile_map.insert("domain_short".into(), domain_short);
    mobile_map
        .entry("mobile_list_title".into())
        .or_insert_with(|| "item.id".into());
    mobile_map
        .entry("mobile_list_description".into())
        .or_insert_with(|| "null".into());
    mobile_map
        .entry("mobile_lookup_methods".into())
        .or_default();
    mobile_map.entry("mobile_tab_scenes".into()).or_default();
    mobile_map
        .entry("mobile_tab_route_builder".into())
        .or_default();
    mobile_map
        .entry("mobile_screen_extra_imports".into())
        .or_default();
    mobile_map.entry("mobile_screen_props".into()).or_default();
    mobile_map.entry("mobile_screen_state".into()).or_default();
    mobile_map
        .entry("mobile_screen_effects".into())
        .or_default();
    mobile_map
        .entry("mobile_screen_form_props".into())
        .or_default();
    mobile_map
        .entry("mobile_form_validations".into())
        .or_default();
    mobile_map
        .entry("mobile_form_extra_imports".into())
        .or_default();
    mobile_map
        .entry("mobile_form_external_imports".into())
        .or_default();
    mobile_map.entry("mobile_form_props".into()).or_default();
    mobile_map.entry("mobile_form_state".into()).or_default();
    mobile_map.entry("mobile_form_refs".into()).or_default();
    mobile_map
        .entry("mobile_form_initial_values".into())
        .or_default();
    mobile_map.entry("mobile_form_modals".into()).or_default();
    mobile_map.entry("mobile_form_inputs".into()).or_default();
    mobile_map.entry("mobile_form_styles".into()).or_default();
    mobile_map
        .entry("mobile_form_prop_types".into())
        .or_default();
    mobile_map.entry("mobile_list_fields".into()).or_default();
    mobile_map
        .entry("mobile_lookup_methods".into())
        .or_default();

    let files = vec![
        (
            "Mobile/ReactNative/api/Api.ts.tpl",
            rn_root
                .join("src")
                .join("api")
                .join(format!("{}API.ts", entity)),
        ),
        (
            "Mobile/ReactNative/navigators/Navigator.tsx.tpl",
            rn_root
                .join("src")
                .join("navigators")
                .join(format!("{}Navigator.tsx", entity_plural)),
        ),
        (
            "Mobile/ReactNative/screens/ContainerScreen.tsx.tpl",
            rn_root
                .join("src")
                .join("screens")
                .join(&entity_plural)
                .join(format!("{}MainScreen.tsx", entity_plural)),
        ),
        (
            "Mobile/ReactNative/screens/ListScreen.tsx.tpl",
            rn_root
                .join("src")
                .join("screens")
                .join(&entity_plural)
                .join(format!("{}Screen.tsx", entity_plural)),
        ),
        (
            "Mobile/ReactNative/screens/CreateUpdate/CreateUpdateScreen.tsx.tpl",
            rn_root
                .join("src")
                .join("screens")
                .join(&entity_plural)
                .join(format!(
                    "CreateUpdate{}/CreateUpdate{}Screen.tsx",
                    entity, entity
                )),
        ),
        (
            "Mobile/ReactNative/screens/CreateUpdate/CreateUpdateForm.tsx.tpl",
            rn_root
                .join("src")
                .join("screens")
                .join(&entity_plural)
                .join(format!(
                    "CreateUpdate{}/CreateUpdate{}Form.tsx",
                    entity, entity
                )),
        ),
    ];

    for (tpl, out_path) in files {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = render_template(&read_tpl_text(tpl)?, &mobile_map);
        write_text(&out_path, &content)?;
        log(&format!("created mobile: {}", out_path.display()));
    }

    ensure_mobile_drawer_entries(project_root, mapping, log)?;
    ensure_mobile_localization(project_root, mapping, fields, log)?;

    Ok(())
}

fn ensure_mobile_drawer_entries(
    project_root: &Path,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let rn_root = project_root.join("react-native");
    let drawer_nav = rn_root.join("src/navigators/DrawerNavigator.tsx");
    let drawer_content = rn_root.join("src/components/DrawerContent/DrawerContent.tsx");

    let entity = mapping
        .get("entity_name")
        .cloned()
        .or_else(|| mapping.get("entity").cloned())
        .unwrap_or_default();
    let entity_plural = mapping
        .get("entity_plural")
        .cloned()
        .or_else(|| mapping.get("entity_plural_kebab").cloned())
        .unwrap_or_else(|| pluralize(&entity));
    let domain_short = mapping
        .get("domain_short")
        .cloned()
        .unwrap_or_else(|| "App".into());

    if drawer_nav.exists() {
        let mut text = fs::read_to_string(&drawer_nav)?;
        let import_line = format!(
            "import {entity}StackNavigator from './{plural}Navigator';",
            entity = entity,
            plural = entity_plural
        );
        if !text.contains(&import_line) {
            let insert_pos = text
                .find("import HeaderBackground")
                .unwrap_or_else(|| text.find("const Drawer").unwrap_or(0));
            text.insert_str(insert_pos, &format!("{import_line}\n"));
            log(&format!("added mobile drawer import for {}", entity_plural));
        }

        let screen_block = format!(
            "      <Drawer.Screen\n        name=\"{plural}Stack\"\n        component={{{entity}StackNavigator}}\n        options={{{{ header: () => null }}}}\n      />\n",
            plural = entity_plural,
            entity = entity
        );
        if !text.contains(&format!("name=\"{plural}Stack\"", plural = entity_plural)) {
            if let Some(pos) = text.find("      <Drawer.Screen\n        name=\"TenantsStack\"") {
                text.insert_str(pos, &screen_block);
            } else if let Some(pos) = text.find("      <Drawer.Screen") {
                text.insert_str(pos, &screen_block);
            } else {
                text.push_str(&screen_block);
            }
            log(&format!("added mobile drawer screen for {}", entity_plural));
        } else {
            // Normalize options braces if the screen already exists
            let needle = format!("name=\"{plural}Stack\"", plural = entity_plural);
            if let Some(start) = text.find(&needle) {
                let slice = &text[start..];
                if let Some(end) = slice.find("/>") {
                    let block = slice[..=end].to_string();
                    if block.contains("options={ header: () => null }") {
                        let fixed = block.replace(
                            "options={ header: () => null }",
                            "options={{ header: () => null }}",
                        );
                        text = text.replace(&block, &fixed);
                        log(&format!(
                            "normalized drawer options braces for {}",
                            entity_plural
                        ));
                    }
                }
            }
        }

        // Safety: normalize any stray single-brace header options globally
        text = text.replace(
            "options={ header: () => null }",
            "options={{ header: () => null }}",
        );

        write_text(&drawer_nav, &text)?;
    } else {
        log("DrawerNavigator.js not found, skipping drawer injection");
    }

    if drawer_content.exists() {
        let mut text = fs::read_to_string(&drawer_content)?;
        let screen_entry = format!(
            "  {plural}Stack: {{ label: '{domain}::Menu:{plural}', iconName: 'list', requiredPolicy: '{domain}.{plural}' }},\n",
            plural = entity_plural,
            domain = domain_short
        );
        if !text.contains(&format!("{plural}Stack", plural = entity_plural)) {
            if let Some(pos) = text.find("  UsersStack") {
                text.insert_str(pos, &screen_entry);
            } else if let Some(pos) = text.find("const screens = {\n") {
                text.insert_str(pos + "const screens = {\n".len(), &screen_entry);
            } else {
                text.push_str(&format!("\nconst screens = {{\n{screen_entry}}};\n"));
            }
            log(&format!(
                "added mobile drawer content entry for {}",
                entity_plural
            ));
        }
        write_text(&drawer_content, &text)?;
    } else {
        log("DrawerContent.js not found, skipping drawer menu injection");
    }

    Ok(())
}

fn ensure_mobile_localization(
    project_root: &Path,
    mapping: &HashMap<String, String>,
    fields: &[Field],
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let domain_short = match mapping.get("domain_short") {
        Some(s) if !s.is_empty() => s.clone(),
        _ => {
            log("domain_short not present in mapping; skipping mobile localization");
            return Ok(());
        }
    };

    let loc_path = project_root
        .join("react-native")
        .join("src")
        .join("localization")
        .join(&domain_short)
        .join("en.json");

    if !loc_path.exists() {
        log(&format!(
            "Mobile localization file not found at {}, skipping localization merge",
            loc_path.display()
        ));
        return Ok(());
    }

    let text = fs::read_to_string(&loc_path)
        .with_context(|| format!("failed to read {}", loc_path.display()))?;
    let mut root: Value = serde_json::from_str(&text).unwrap_or_else(|_| Value::Object(Map::new()));

    if !root.is_object() {
        root = Value::Object(Map::new());
    }
    let root_obj = root.as_object_mut().unwrap();

    let domain_block = root_obj
        .entry(domain_short.clone())
        .or_insert_with(|| Value::Object(Map::new()));
    let domain_map = if let Some(map) = domain_block.as_object_mut() {
        map
    } else {
        *domain_block = Value::Object(Map::new());
        domain_block.as_object_mut().unwrap()
    };

    let entity_name = mapping
        .get("entity_name")
        .cloned()
        .unwrap_or_else(|| mapping.get("entity").cloned().unwrap_or_default());
    let entity_plural = mapping
        .get("entity_plural")
        .cloned()
        .unwrap_or_else(|| pluralize(&entity_name));

    let mut ensure = |k: &str, v: &str| {
        domain_map
            .entry(k.to_string())
            .or_insert_with(|| Value::String(v.to_string()));
    };

    ensure(&format!("Menu:{}", entity_plural), &entity_plural);
    ensure(
        &format!("New{}", entity_name),
        &format!("New {}", entity_name),
    );
    ensure(&format!("Permission:{}", entity_plural), &entity_plural);
    ensure(&entity_plural, &entity_plural);
    ensure(
        "AreYouSureToDelete",
        "Are you sure you want to delete this item?",
    );

    for f in fields {
        let key = format!("{}:{}", entity_name, f.name);
        let value = if f.ftype.eq_ignore_ascii_case("Guid") {
            if let Some(nav_full) = &f.navigation {
                last_segment(nav_full).to_string()
            } else {
                f.name.clone()
            }
        } else {
            f.name.clone()
        };
        ensure(&key, &value);
    }

    fs::write(&loc_path, serde_json::to_string_pretty(&root)?)?;
    log(&format!(
        "Merged mobile localization entries into {}",
        loc_path.display()
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_support::GenerationFixture;

    #[test]
    fn renders_the_complete_react_native_feature_manifest() {
        let fixture = GenerationFixture::new();
        let context = fixture.context();
        let mapping = fixture.mapping();
        fs::create_dir_all(fixture.root().join("react-native")).expect("React Native fixture");
        let mut messages = Vec::new();

        generate_mobile_react_native(&context, &mapping, &mut |message| {
            messages.push(message.to_owned());
        })
        .expect("React Native generation");

        let root = fixture.root().join("react-native/src");
        for file in [
            "api/BookAPI.ts",
            "navigators/BooksNavigator.tsx",
            "screens/Books/BooksMainScreen.tsx",
            "screens/Books/BooksScreen.tsx",
            "screens/Books/CreateUpdateBook/CreateUpdateBookScreen.tsx",
            "screens/Books/CreateUpdateBook/CreateUpdateBookForm.tsx",
        ] {
            assert!(root.join(file).is_file(), "missing mobile output: {file}");
        }

        let api = fs::read_to_string(root.join("api/BookAPI.ts")).expect("mobile API");
        assert!(api.contains("/api/app/books"));
        assert!(!api.contains("__entity_"));
        assert!(
            messages
                .iter()
                .any(|message| message.contains("created mobile"))
        );
    }
}

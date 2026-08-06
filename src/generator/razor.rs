use super::GenerationContext;
use crate::templates::{embedded_walk, read_tpl_text};
use crate::utils::{pluralize, render_template, write_text};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(super) fn generate_mvc_razor(
    context: &GenerationContext<'_>,
    mapping: &HashMap<String, String>,
) -> Result<()> {
    let out_root = &context.out_root;
    let domain = context.domain;
    let entity = context.entity;
    let web_files = embedded_walk("Web/MVC/RazorPage");
    let entity_plural = mapping
        .get("entity_plural")
        .cloned()
        .or_else(|| mapping.get("entity").map(|e| pluralize(e)))
        .unwrap_or_else(|| pluralize(entity));
    let out_dir = out_root
        .join(format!("{}.Web", domain))
        .join("Pages")
        .join(entity_plural);
    fs::create_dir_all(&out_dir)?;
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
        let content = render_template(&read_tpl_text(&rel)?, mapping);
        write_text(&out_dir.join(out_name), &content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::test_support::GenerationFixture;

    #[test]
    fn renders_the_complete_razor_file_manifest() {
        let fixture = GenerationFixture::new();
        let context = fixture.context();
        let mapping = fixture.mapping();

        generate_mvc_razor(&context, &mapping).expect("Razor generation");

        let output = context.out_root.join("Acme.BookStore.Web/Pages/Books");
        let expected = [
            "Index.cshtml",
            "Index.cshtml.cs",
            "index.js",
            "Index.css",
            "CreateModal.cshtml",
            "CreateModal.cshtml.cs",
            "createModal.js",
            "CreateModal.css",
            "EditModal.cshtml",
            "EditModal.cshtml.cs",
            "editModal.js",
            "EditModal.css",
            "DetailModal.cshtml",
            "DetailModal.cshtml.cs",
            "detailModal.js",
            "DetailModal.css",
        ];
        for file in expected {
            assert!(output.join(file).is_file(), "missing Razor output: {file}");
        }

        let index = fs::read_to_string(output.join("Index.cshtml")).expect("Razor index");
        assert!(index.contains("Acme.BookStore.Web.Pages.Books"));
        assert!(index.contains("BookStoreMenus.Books"));
        assert!(!index.contains("${entity_"));
    }
}

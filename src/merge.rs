mod application;
mod ef_core;
mod permissions;
mod web;

use anyhow::{Result, anyhow};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

#[cfg(test)]
use std::fs;

/* ------------------------------ MERGE HELPERS ------------------------- */

fn insert_before_last_close_brace_of_method(
    src: &str,
    method_sig_rx: &Regex,
    insertion: &str,
) -> String {
    if let Some(m) = method_sig_rx.find(src)
        && let Some(start) = src[m.end() - 1..].find('{')
    {
        let mut depth = 0i32;
        let mut i = m.end() - 1 + start;
        while i < src.len() {
            let ch = src.as_bytes()[i] as char;
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let (head, tail) = src.split_at(i);
                    return format!("{head}{ins}{tail}", ins = insertion);
                }
            }
            i += 1;
        }
    }
    src.into()
}

fn insert_inside_class(
    src: &str,
    class_decl_rx: &Regex,
    insertion: &str,
    before_final: bool,
) -> String {
    if let Some(m) = class_decl_rx.find(src)
        && let Some(start_rel) = src[m.end() - 1..].find('{')
    {
        let start = m.end() - 1 + start_rel;
        let mut depth = 0i32;
        let mut i = start;
        while i < src.len() {
            let ch = src.as_bytes()[i] as char;
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    if before_final {
                        let (head, tail) = src.split_at(i);
                        return format!("{head}{insertion}{tail}");
                    } else {
                        let (head, tail) = src.split_at(start + 1);
                        return format!("{head}{insertion}{tail}");
                    }
                }
            }
            i += 1;
        }
    }
    src.into()
}

fn ensure_using(src: &str, using_stmt: &str) -> String {
    if src.contains(using_stmt.trim()) {
        return src.into();
    }
    let mut lines: Vec<&str> = src.lines().collect();
    let mut last_using_idx: isize = -1;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("using ") {
            last_using_idx = idx as isize;
        }
    }
    let ins = format!("{}\n", using_stmt.trim_end());
    if last_using_idx >= 0 {
        lines.insert((last_using_idx as usize) + 1, ins.as_str());
        lines.join("\n")
    } else {
        format!("{ins}{}", src)
    }
}

struct MergeContext<'a> {
    domain: &'a str,
    short: &'a str,
    entity: &'a str,
    entity_plural: &'a str,
    entity_lower: &'a str,
    namespace: &'a str,
    web_namespace: &'a str,
    is_mvc: bool,
    is_razor: bool,
}

impl<'a> MergeContext<'a> {
    fn from_mapping(mapping: &'a HashMap<String, String>) -> Result<Self> {
        let required = |key: &str| {
            mapping
                .get(key)
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow!("merge mapping is missing required key '{key}'"))
        };
        let ui_target = mapping
            .get("ui_target")
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_default();
        let is_mvc = matches!(ui_target.as_str(), "vue" | "razor");
        let is_razor = ui_target == "razor";
        let web_namespace = if is_razor {
            required("web_namespace")?
        } else {
            mapping
                .get("web_namespace")
                .map(String::as_str)
                .unwrap_or_default()
        };

        Ok(Self {
            domain: required("domain_name")?,
            short: required("domain_short")?,
            entity: required("entity_name")?,
            entity_plural: required("entity_plural")?,
            entity_lower: required("entity_name_lower")?,
            namespace: required("namespace")?,
            web_namespace,
            is_mvc,
            is_razor,
        })
    }
}

/* ------------------------------ MERGE CORE ---------------------------- */

pub(crate) fn merge_into_existing_files(
    out_root: &Path,
    mapping: &HashMap<String, String>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let reporter = RefCell::new(log);
    let context = MergeContext::from_mapping(mapping)?;

    application::merge_application(out_root, &context, mapping, &reporter)?;
    permissions::merge_permissions(out_root, &context, &reporter)?;
    ef_core::merge_ef_core(out_root, &context, &reporter)?;
    web::merge_web(out_root, &context, &reporter)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping() -> HashMap<String, String> {
        HashMap::from([
            ("domain_name".into(), "Acme.BookStore".into()),
            ("domain_short".into(), "BookStore".into()),
            ("entity_name".into(), "Book".into()),
            ("entity_plural".into(), "Books".into()),
            ("entity_name_lower".into(), "book".into()),
            ("namespace".into(), "Acme.BookStore.Books".into()),
            (
                "web_namespace".into(),
                "Acme.BookStore.Web.Pages.Books".into(),
            ),
            ("ui_target".into(), "none".into()),
        ])
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture directory");
        }
        fs::write(path, content).expect("fixture file");
    }

    #[test]
    fn inserts_inside_nested_method_and_class_bodies() {
        let method = Regex::new(r"void\s+Configure\s*\(\s*\)\s*").expect("method regex");
        let source = "class Sample\n{\n    void Configure()\n    {\n        if (true) { Run(); }\n    }\n}\n";
        let merged =
            insert_before_last_close_brace_of_method(source, &method, "\n        Added();\n");
        let run_position = merged.find("Run();").expect("existing method body");
        let added_position = merged.find("Added();").expect("inserted method body");
        assert!(run_position < added_position);
        assert_eq!(merged.matches("Added();").count(), 1);

        let class = Regex::new(r"class\s+Sample\s*").expect("class regex");
        let merged = insert_inside_class(&merged, &class, "\n    int Value;\n", true);
        assert!(merged.contains("    int Value;\n}"));
    }

    #[test]
    fn ensures_using_once_after_existing_usings() {
        let source = "using System;\n\nnamespace Demo;\n";
        let once = ensure_using(source, "using Demo.Books;");
        let twice = ensure_using(&once, "using Demo.Books;");

        assert_eq!(once, twice);
        assert_eq!(once.matches("using Demo.Books;").count(), 1);
        assert!(once.starts_with("using System;\nusing Demo.Books;"));
    }

    #[test]
    fn reports_missing_mapping_keys_without_panicking() {
        let fixture = tempfile::tempdir().expect("merge fixture");
        let error = merge_into_existing_files(fixture.path(), &HashMap::new(), &mut |_| {})
            .expect_err("invalid mapping should fail");

        assert!(error.to_string().contains("domain_name"));
    }

    #[test]
    fn core_csharp_merge_is_idempotent() {
        let fixture = tempfile::tempdir().expect("merge fixture");
        let root = fixture.path();
        let app_profile =
            root.join("Acme.BookStore.Application/BookStoreApplicationAutoMapperProfile.cs");
        let permissions =
            root.join("Acme.BookStore.Application.Contracts/Permissions/BookStorePermissions.cs");
        let provider = root.join(
            "Acme.BookStore.Application.Contracts/Permissions/BookStorePermissionDefinitionProvider.cs",
        );
        let db_context = root
            .join("Acme.BookStore.EntityFrameworkCore/EntityFrameworkCore/BookStoreDbContext.cs");

        write(
            &app_profile,
            "namespace Acme.BookStore;\npublic class BookStoreApplicationAutoMapperProfile\n{\n    public BookStoreApplicationAutoMapperProfile()\n    {\n    }\n}\n",
        );
        write(
            &permissions,
            "namespace Acme.BookStore.Permissions;\npublic static class BookStorePermissions\n{\n    public const string GroupName = \"BookStore\";\n}\n",
        );
        write(
            &provider,
            "public class BookStorePermissionDefinitionProvider\n{\n    public override void Define(IPermissionDefinitionContext context)\n    {\n        var myGroup = context.AddGroup(\"BookStore\");\n    }\n}\n",
        );
        write(
            &db_context,
            "using Microsoft.EntityFrameworkCore;\npublic class BookStoreDbContext : DbContext\n{\n    protected override void OnModelCreating(ModelBuilder builder)\n    {\n        base.OnModelCreating(builder);\n    }\n}\n",
        );

        let map = mapping();
        let mut messages: Vec<String> = Vec::new();
        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("first merge");
        let first = [
            fs::read_to_string(&app_profile).expect("profile after merge"),
            fs::read_to_string(&permissions).expect("permissions after merge"),
            fs::read_to_string(&provider).expect("provider after merge"),
            fs::read_to_string(&db_context).expect("db context after merge"),
        ];

        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("second merge");
        let second = [
            fs::read_to_string(&app_profile).expect("profile after second merge"),
            fs::read_to_string(&permissions).expect("permissions after second merge"),
            fs::read_to_string(&provider).expect("provider after second merge"),
            fs::read_to_string(&db_context).expect("db context after second merge"),
        ];

        assert_eq!(first, second);
        assert_eq!(first[0].matches("CreateMap<Book, BookDto>").count(), 1);
        assert_eq!(first[1].matches("class Books").count(), 1);
        assert_eq!(first[2].matches("var bookPermission").count(), 1);
        assert_eq!(first[3].matches("DbSet<Book> Books").count(), 1);
        assert_eq!(first[3].matches("builder.ConfigureBook();").count(), 1);
    }

    #[test]
    fn razor_web_and_mapperly_fallback_merge_are_idempotent() {
        let fixture = tempfile::tempdir().expect("merge fixture");
        let root = fixture.path();
        let app_mappers = root.join("Acme.BookStore.Application/BookStoreApplicationMappers.cs");
        let menus = root.join("Acme.BookStore.Web/Menus/BookStoreMenus.cs");
        let contributor = root.join("Acme.BookStore.Web/Menus/BookStoreMenuContributor.cs");
        let web_profile =
            root.join("Acme.BookStore.Web/AutoMapper/BookStoreWebAutoMapperProfile.cs");
        let global_styles = root.join("Acme.BookStore.Web/wwwroot/global-styles.css");

        write(&app_mappers, "namespace Acme.BookStore;\n");
        write(
            &menus,
            "public class BookStoreMenus\n{\n    public const string Prefix = \"BookStore\";\n}\n",
        );
        write(
            &contributor,
            "using System.Threading.Tasks;\npublic class BookStoreMenuContributor\n{\n    private static Task ConfigureMainMenuAsync(MenuConfigurationContext context)\n    {\n        return Task.CompletedTask;\n    }\n}\n",
        );
        write(
            &web_profile,
            "public class BookStoreWebAutoMapperProfile : Profile\n{\n    public BookStoreWebAutoMapperProfile()\n    {\n    }\n}\n",
        );

        let mut map = mapping();
        map.insert("ui_target".into(), "razor".into());
        let mut messages: Vec<String> = Vec::new();
        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("first Razor merge");
        let first = [
            fs::read_to_string(&app_mappers).expect("application mappers"),
            fs::read_to_string(&menus).expect("web menus"),
            fs::read_to_string(&contributor).expect("menu contributor"),
            fs::read_to_string(&web_profile).expect("web profile"),
            fs::read_to_string(&global_styles).expect("global styles"),
        ];

        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("second Razor merge");
        let second = [
            fs::read_to_string(&app_mappers).expect("application mappers after second merge"),
            fs::read_to_string(&menus).expect("web menus after second merge"),
            fs::read_to_string(&contributor).expect("menu contributor after second merge"),
            fs::read_to_string(&web_profile).expect("web profile after second merge"),
            fs::read_to_string(&global_styles).expect("global styles after second merge"),
        ];

        assert_eq!(first, second);
        assert_eq!(first[0].matches("class BookToBookDtoMapper").count(), 1);
        assert_eq!(first[1].matches("public const string Books").count(), 1);
        assert_eq!(first[2].matches("BookStoreMenus.Books").count(), 1);
        assert_eq!(
            first[3]
                .matches("CreateMap<BookDto, BookUpdateViewModel>")
                .count(),
            1
        );
        assert!(first[4].contains(".select2"));
    }

    #[test]
    fn switching_from_razor_to_vue_removes_only_obsolete_viewmodel_mappings() {
        let fixture = tempfile::tempdir().expect("merge fixture");
        let root = fixture.path();
        let menus = root.join("Acme.BookStore.Web/Menus/BookStoreMenus.cs");
        let contributor = root.join("Acme.BookStore.Web/Menus/BookStoreMenuContributor.cs");
        let mapperly = root.join("Acme.BookStore.Web/BookStoreWebMappers.cs");
        let automapper =
            root.join("Acme.BookStore.Web/AutoMapper/BookStoreWebAutoMapperProfile.cs");

        write(
            &menus,
            "public class BookStoreMenus\n{\n    public const string Prefix = \"BookStore\";\n    public const string Books = Prefix + \".Books\";\n}\n",
        );
        write(
            &contributor,
            "using System.Threading.Tasks;\npublic class BookStoreMenuContributor\n{\n    private static Task ConfigureMainMenuAsync(MenuConfigurationContext context)\n    {\n        var item = BookStoreMenus.Books;\n        return Task.CompletedTask;\n    }\n}\n",
        );
        write(
            &mapperly,
            r#"using Riok.Mapperly.Abstractions;
using Volo.Abp.Mapperly;
namespace Acme.BookStore.Web;

[Mapper]
public partial class CustomMapper : MapperBase<CustomSource, CustomTarget>
{
    public override partial CustomTarget Map(CustomSource source);
    public override partial void Map(CustomSource source, CustomTarget destination);
}

[Mapper]
public partial class BookDtoToBookUpdateViewModelMapper : MapperBase<BookDto, BookUpdateViewModel>
{
    public override partial BookUpdateViewModel Map(BookDto source);
    public override partial void Map(BookDto source, BookUpdateViewModel destination);
}

[Mapper]
public partial class BookUpdateViewModelToBookUpdateDtoMapper : MapperBase<BookUpdateViewModel, BookUpdateDto>
{
    public override partial BookUpdateDto Map(BookUpdateViewModel source);
    public override partial void Map(BookUpdateViewModel source, BookUpdateDto destination);
}

[Mapper]
public partial class BookCreateViewModelToBookCreateDtoMapper : MapperBase<BookCreateViewModel, BookCreateDto>
{
    public override partial BookCreateDto Map(BookCreateViewModel source);
    public override partial void Map(BookCreateViewModel source, BookCreateDto destination);
}

[Mapper]
public partial class BookDtoToBookDetailViewModelMapper : MapperBase<BookDto, BookDetailViewModel>
{
    public override partial BookDetailViewModel Map(BookDto source);
    public override partial void Map(BookDto source, BookDetailViewModel destination);
}
"#,
        );
        write(
            &automapper,
            "public class BookStoreWebAutoMapperProfile : Profile\n{\n    public BookStoreWebAutoMapperProfile()\n    {\n        CreateMap<BookDto, BookUpdateViewModel>();\n        CreateMap<BookDto, BookDetailViewModel>();\n        CreateMap<CustomSource, CustomTarget>();\n    }\n}\n",
        );

        let mut map = mapping();
        map.insert("ui_target".into(), "vue".into());
        let mut messages = Vec::new();
        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("Vue migration cleanup");
        merge_into_existing_files(root, &map, &mut |message| messages.push(message.to_owned()))
            .expect("idempotent Vue migration cleanup");

        let mapperly = fs::read_to_string(mapperly).expect("cleaned Mapperly file");
        let automapper = fs::read_to_string(automapper).expect("cleaned AutoMapper file");
        assert!(!mapperly.contains("BookUpdateViewModel"));
        assert!(!mapperly.contains("BookCreateViewModel"));
        assert!(!mapperly.contains("BookDetailViewModel"));
        assert!(mapperly.contains("class CustomMapper"));
        assert!(!automapper.contains("BookUpdateViewModel"));
        assert!(!automapper.contains("BookDetailViewModel"));
        assert!(automapper.contains("CreateMap<CustomSource, CustomTarget>()"));
    }
}

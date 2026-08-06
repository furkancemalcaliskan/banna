use super::{
    GenerationContext, compile_fragments, ensure_domain_shared_localization,
    ensure_mini_excel_package, ensure_web_module_toolbars,
};
use crate::models::Field;
use crate::ports::{CommandOutput, CommandRunner, CommandSpec};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

pub(super) struct GenerationFixture {
    directory: tempfile::TempDir,
    fields: Vec<Field>,
}

#[derive(Default)]
struct RecordingRunner(Mutex<Option<CommandSpec>>);

impl CommandRunner for RecordingRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        *self.0.lock().expect("recording lock") = Some(spec.clone());
        Ok(CommandOutput {
            status_code: Some(0),
            success: true,
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }
}

#[test]
fn adds_mini_excel_with_dotnet_supported_arguments() {
    let directory = tempfile::tempdir().expect("project fixture");
    let app_dir = directory.path().join("src/Acme.BookStore.Application");
    fs::create_dir_all(&app_dir).expect("application directory");
    fs::write(
        app_dir.join("Acme.BookStore.Application.csproj"),
        "<Project />",
    )
    .expect("application project");
    let runner = RecordingRunner::default();

    ensure_mini_excel_package(&runner, directory.path(), "Acme.BookStore", &mut |_| {})
        .expect("package installation should succeed");

    let command = runner
        .0
        .lock()
        .expect("recording lock")
        .clone()
        .expect("command should be recorded");
    assert_eq!(command.program, "dotnet");
    assert_eq!(command.args, ["add", "package", "MiniExcel"]);
    assert_eq!(
        command.env.get("DOTNET_NOLOGO").map(String::as_str),
        Some("1")
    );
}

#[test]
fn toolbar_patch_preserves_method_indentation_and_is_idempotent() {
    let directory = tempfile::tempdir().expect("project fixture");
    let web_dir = directory.path().join("src/Acme.BookStore.Web");
    fs::create_dir_all(&web_dir).expect("web directory");
    let module = web_dir.join("BookStoreWebModule.cs");
    fs::write(
        &module,
        "public class BookStoreWebModule\n{\n    public override void ConfigureServices(ServiceConfigurationContext context)\n    {\n        ConfigureBundles(hostingEnvironment);\n    }\n}\n",
    )
    .expect("web module");

    ensure_web_module_toolbars(directory.path(), "Acme.BookStore", &mut |_| {})
        .expect("first toolbar patch");
    ensure_web_module_toolbars(directory.path(), "Acme.BookStore", &mut |_| {})
        .expect("second toolbar patch");

    let output = fs::read_to_string(module).expect("patched module");
    assert_eq!(output.matches("ConfigureToolbars();").count(), 1);
    assert!(
        output.contains(
            "        ConfigureBundles(hostingEnvironment);\n        ConfigureToolbars();\n"
        ),
        "unexpected method indentation:\n{output}"
    );
    assert!(output.contains("    private void ConfigureToolbars()"));
}

#[test]
fn localization_adds_missing_angular_pager_keys_without_overwriting_existing_values() {
    let fixture = GenerationFixture::new();
    let localization_dir = fixture
        .root()
        .join("src/Acme.BookStore.Domain.Shared/Localization/BookStore");
    fs::create_dir_all(&localization_dir).expect("localization directory");
    let en_json = localization_dir.join("en.json");
    fs::write(
        &en_json,
        r#"{
  "Culture": "en",
  "Texts": {
    "PagerShow": "Keep my custom translation"
  }
}"#,
    )
    .expect("localization fixture");

    let context = fixture.context();
    let mapping = fixture.mapping();
    ensure_domain_shared_localization(
        &context.out_root,
        context.domain,
        &mapping,
        context.fields,
        &mut |_| {},
    )
    .expect("localization generation");

    let localization: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(en_json).expect("generated localization"))
            .expect("valid localization json");
    let texts = localization["Texts"]
        .as_object()
        .expect("localization texts");

    assert_eq!(
        texts["PagerShow"],
        serde_json::Value::String("Keep my custom translation".into())
    );
    for key in [
        "PagerPageSize",
        "PagerEntries",
        "PagerInfo",
        "PagerFirst",
        "PagerPrevious",
        "PagerNext",
        "PagerLast",
    ] {
        assert!(texts.contains_key(key), "missing generated key: {key}");
    }
}

impl GenerationFixture {
    pub(super) fn new() -> Self {
        let directory = tempfile::tempdir().expect("generation fixture");
        fs::create_dir_all(directory.path().join("src")).expect("fixture src");
        Self {
            directory,
            fields: vec![Field {
                name: "Title".into(),
                ftype: "string".into(),
                required: true,
                navigation_display: None,
                max_length: Some(200),
                navigation: None,
                filterable: true,
                show_in_ui: true,
            }],
        }
    }

    pub(super) fn root(&self) -> &std::path::Path {
        self.directory.path()
    }

    pub(super) fn context(&self) -> GenerationContext<'_> {
        GenerationContext {
            project_root: self.root(),
            out_root: self.root().join("src"),
            domain: "Acme.BookStore",
            namespace: "Acme.BookStore.Books",
            entity: "Book",
            fields: &self.fields,
        }
    }

    pub(super) fn mapping(&self) -> HashMap<String, String> {
        compile_fragments(&self.context())
    }
}

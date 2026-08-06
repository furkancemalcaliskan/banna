use crate::application::{
    ConfigureProjectOutcome, ConfigureProjectRequest, GenerateEntityRequest, GenerationOptions,
    GenerationOutcome, RegenerateEntityRequest, add_project, configure_project, generate_entity,
    inspect_project, list_history, list_projects, regenerate_entity, remove_history_entity,
    remove_project,
};
use crate::models::{AbpTheme, BootstrapOverride, Field, MobileUi, UiTarget};
use crate::state::JsonProjectIndexStore;
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "banna-cli",
    version,
    about = "Headless ABP code generator by Furkan Cemal Caliskan"
)]
struct HeadlessCli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Manage and generate entity code.
    Entity(EntityArgs),
    /// Manage Banna's registered projects.
    Project(ProjectArgs),
    /// Inspect and maintain per-project generation history.
    History(HistoryArgs),
}

#[derive(Debug, Args)]
struct EntityArgs {
    #[command(subcommand)]
    command: EntityCommand,
}

#[derive(Debug, Subcommand)]
enum EntityCommand {
    /// Generate an entity from a JSON field definition.
    Generate(GenerateArgs),
    /// Regenerate an entity using its saved project history definition.
    Regenerate(RegenerateArgs),
}

#[derive(Debug, Args)]
struct RegenerateArgs {
    /// Root of the project containing generation history.
    #[arg(long)]
    project: PathBuf,

    /// Exact entity name in generation history.
    #[arg(long)]
    entity: String,

    /// Do not merge registrations into existing source files.
    #[arg(long)]
    no_merge: bool,

    /// Preview regeneration without changing files, history, or external systems.
    #[arg(long)]
    dry_run: bool,

    /// Add an EF Core migration and update the database after regeneration.
    #[arg(long)]
    run_migration: bool,

    /// Emit a machine-readable completion object on stdout.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// Compatibility flag; external tool execution is already non-interactive.
    #[arg(long)]
    non_interactive: bool,
}

#[derive(Debug, Args)]
struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommand,
}

#[derive(Debug, Subcommand)]
enum ProjectCommand {
    /// List registered projects.
    List(OutputArgs),
    /// Register an existing ABP project.
    Add(AddProjectArgs),
    /// Remove a project from Banna's index without deleting its files.
    Remove(RemoveProjectArgs),
    /// Inspect detected project capabilities and saved generation metadata.
    Inspect(InspectProjectArgs),
    /// Apply theme, Bootstrap override, or mobile UI configuration.
    Configure(ConfigureProjectArgs),
}

#[derive(Debug, Args)]
struct ConfigureProjectArgs {
    /// Root of the project containing generation history.
    #[arg(long)]
    path: PathBuf,

    /// ABP theme to apply; omitted values retain their current setting.
    #[arg(long, value_enum)]
    theme: Option<CliTheme>,

    /// Bootstrap style override to apply.
    #[arg(long, value_enum)]
    bootstrap_override: Option<CliBootstrapOverride>,

    /// Mobile UI scaffold selection.
    #[arg(long, value_enum)]
    mobile_ui: Option<CliMobileUi>,

    /// Compatibility flag; external tool execution is already non-interactive.
    #[arg(long)]
    non_interactive: bool,

    /// Preview project configuration without changing files or metadata.
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct InspectProjectArgs {
    /// Root of the ABP project containing its src directory.
    #[arg(long)]
    path: PathBuf,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct HistoryArgs {
    #[command(subcommand)]
    command: HistoryCommand,
}

#[derive(Debug, Subcommand)]
enum HistoryCommand {
    /// List entities in a project's generation history.
    List(HistoryListArgs),
    /// Remove an entity from history without deleting generated files.
    Remove(HistoryRemoveArgs),
}

#[derive(Debug, Args)]
struct HistoryListArgs {
    /// Root of the project containing `.history/codegen_history.json`.
    #[arg(long)]
    project: PathBuf,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct HistoryRemoveArgs {
    /// Root of the project containing `.history/codegen_history.json`.
    #[arg(long)]
    project: PathBuf,

    /// Exact entity name to remove from history.
    #[arg(long)]
    entity: String,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct OutputArgs {
    /// Select human-readable or JSON output.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct AddProjectArgs {
    /// Root of the ABP project containing its src directory.
    #[arg(long)]
    path: PathBuf,

    /// Display name; defaults to the project directory name.
    #[arg(long)]
    name: Option<String>,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct RemoveProjectArgs {
    /// Registered project name.
    name: String,

    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Debug, Args)]
struct GenerateArgs {
    /// Root of the ABP project containing its src directory.
    #[arg(long)]
    project: PathBuf,

    /// Root ABP namespace, for example Acme.Billing.
    #[arg(long)]
    domain: String,

    /// Entity namespace, for example Acme.Billing.Invoices.
    #[arg(long)]
    namespace: String,

    /// Entity class name, for example Invoice.
    #[arg(long)]
    entity: String,

    /// JSON file containing an array of field definitions.
    #[arg(long)]
    fields: PathBuf,

    /// UI target to generate.
    #[arg(long, value_enum, default_value_t = CliUiTarget::None)]
    ui: CliUiTarget,

    /// Optional mobile UI target.
    #[arg(long, value_enum, default_value_t = CliMobileUi::None)]
    mobile_ui: CliMobileUi,

    /// Do not merge registrations into existing source files.
    #[arg(long)]
    no_merge: bool,

    /// Preview generation without changing files or external systems.
    #[arg(long)]
    dry_run: bool,

    /// Add an EF Core migration and update the database after generation.
    #[arg(long)]
    run_migration: bool,

    /// Emit a machine-readable completion object on stdout.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// Compatibility flag; external tool execution is already non-interactive.
    #[arg(long)]
    non_interactive: bool,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum CliUiTarget {
    #[default]
    None,
    Razor,
    Vue,
    Angular,
}

impl From<CliUiTarget> for UiTarget {
    fn from(value: CliUiTarget) -> Self {
        match value {
            CliUiTarget::None => Self::None,
            CliUiTarget::Razor => Self::Razor,
            CliUiTarget::Vue => Self::Vue,
            CliUiTarget::Angular => Self::Angular,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum CliMobileUi {
    #[default]
    None,
    ReactNative,
}

impl From<CliMobileUi> for MobileUi {
    fn from(value: CliMobileUi) -> Self {
        match value {
            CliMobileUi::None => Self::None,
            CliMobileUi::ReactNative => Self::ReactNative,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliTheme {
    Basic,
    LeptonX,
}

impl From<CliTheme> for AbpTheme {
    fn from(value: CliTheme) -> Self {
        match value {
            CliTheme::Basic => Self::Basic,
            CliTheme::LeptonX => Self::LeptonX,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliBootstrapOverride {
    None,
    Modern,
    Nord,
    Solarized,
}

impl From<CliBootstrapOverride> for BootstrapOverride {
    fn from(value: CliBootstrapOverride) -> Self {
        match value {
            CliBootstrapOverride::None => Self::None,
            CliBootstrapOverride::Modern => Self::Modern,
            CliBootstrapOverride::Nord => Self::Nord,
            CliBootstrapOverride::Solarized => Self::Solarized,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

/// Runs the headless Banna command-line interface.
pub fn run_cli() -> Result<()> {
    execute(HeadlessCli::parse())
}

fn execute(cli: HeadlessCli) -> Result<()> {
    match cli.command {
        Command::Entity(EntityArgs { command }) => match command {
            EntityCommand::Generate(args) => generate(args),
            EntityCommand::Regenerate(args) => regenerate(args),
        },
        Command::Project(ProjectArgs { command }) => project(command),
        Command::History(HistoryArgs { command }) => history(command),
    }
}

fn project(command: ProjectCommand) -> Result<()> {
    let store = JsonProjectIndexStore::platform_default();
    match command {
        ProjectCommand::List(output) => {
            let projects = list_projects(&store)?;
            match output.format {
                OutputFormat::Human if projects.is_empty() => println!("No projects registered."),
                OutputFormat::Human => {
                    for project in projects {
                        println!("{}\t{}", project.project_name, project.project_dir);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&projects)?),
            }
        }
        ProjectCommand::Add(args) => {
            let project = add_project(&store, args.name, args.path)?;
            match args.output.format {
                OutputFormat::Human => println!(
                    "Registered {} at {}",
                    project.project_name, project.project_dir
                ),
                OutputFormat::Json => println!("{}", serde_json::to_string(&project)?),
            }
        }
        ProjectCommand::Remove(args) => {
            let project = remove_project(&store, &args.name)?;
            match args.output.format {
                OutputFormat::Human => {
                    println!("Removed {} from the project index", project.project_name)
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&project)?),
            }
        }
        ProjectCommand::Inspect(args) => {
            let inspection = inspect_project(&args.path)?;
            match args.output.format {
                OutputFormat::Human => {
                    println!("Project: {}", inspection.project_root.display());
                    println!(
                        "Domain: {}",
                        inspection.domain.as_deref().unwrap_or("not detected")
                    );
                    println!("MVC: {}", inspection.has_mvc);
                    println!("Angular: {}", inspection.has_angular);
                    println!("History: {}", inspection.has_history);
                    println!("Entities: {}", inspection.entity_count);
                    println!(
                        "Theme: {}",
                        match inspection.theme {
                            Some(AbpTheme::Basic) => "Basic",
                            Some(AbpTheme::LeptonX) => "LeptonX",
                            None => "not detected",
                        }
                    );
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&inspection)?),
            }
        }
        ProjectCommand::Configure(args) => {
            let mut logger = |message: &str| eprintln!("{message}");
            // Compatibility flag: external tool execution is always non-interactive.
            let _non_interactive = args.non_interactive;
            let outcome = configure_project(
                ConfigureProjectRequest {
                    project_root: args.path,
                    theme: args.theme.map(Into::into),
                    bootstrap_override: args.bootstrap_override.map(Into::into),
                    mobile_ui: args.mobile_ui.map(Into::into),
                    dry_run: args.dry_run,
                },
                Some(&mut logger),
            )?;
            print_configuration_outcome(&outcome, args.output.format)?;
        }
    }
    Ok(())
}

fn history(command: HistoryCommand) -> Result<()> {
    match command {
        HistoryCommand::List(args) => {
            let entities = list_history(&args.project)?;
            match args.output.format {
                OutputFormat::Human if entities.is_empty() => println!("No history entries."),
                OutputFormat::Human => {
                    for entity in entities {
                        println!(
                            "{}\t{}\t{}",
                            entity.name, entity.namespace, entity.generated_at
                        );
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string(&entities)?),
            }
        }
        HistoryCommand::Remove(args) => {
            let removed = remove_history_entity(&args.project, &args.entity)?;
            match args.output.format {
                OutputFormat::Human => println!(
                    "Removed {} from history; generated files were not deleted",
                    removed.name
                ),
                OutputFormat::Json => println!("{}", serde_json::to_string(&removed)?),
            }
        }
    }
    Ok(())
}

fn generate(args: GenerateArgs) -> Result<()> {
    let fields = load_fields(&args.fields)?;
    let request = GenerateEntityRequest {
        project_root: args.project,
        domain: args.domain,
        namespace: args.namespace,
        entity: args.entity,
        fields,
        ui_target: args.ui.into(),
        mobile_ui: args.mobile_ui.into(),
        options: GenerationOptions {
            no_merge: args.no_merge,
            run_migration: args.run_migration,
            dry_run: args.dry_run,
        },
    };

    let mut logger = |message: &str| eprintln!("{message}");
    // Compatibility flag: external tool execution is always non-interactive.
    let _non_interactive = args.non_interactive;
    let outcome = generate_entity(request, Some(&mut logger))?;
    print_generation_outcome(&outcome, args.format)
}

fn regenerate(args: RegenerateArgs) -> Result<()> {
    let mut logger = |message: &str| eprintln!("{message}");
    // Compatibility flag: external tool execution is always non-interactive.
    let _non_interactive = args.non_interactive;
    let outcome = regenerate_entity(
        RegenerateEntityRequest {
            project_root: args.project,
            entity: args.entity,
            options: GenerationOptions {
                no_merge: args.no_merge,
                run_migration: args.run_migration,
                dry_run: args.dry_run,
            },
        },
        Some(&mut logger),
    )?;
    print_generation_outcome(&outcome, args.format)
}

fn print_generation_outcome(outcome: &GenerationOutcome, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Human if outcome.dry_run => println!(
            "Previewed generation of {} in {}; no changes were made",
            outcome.entity,
            outcome.project_root.display()
        ),
        OutputFormat::Human => println!(
            "Generated {} in {}",
            outcome.entity,
            outcome.project_root.display()
        ),
        OutputFormat::Json => println!("{}", serde_json::to_string(outcome)?),
    }
    Ok(())
}

fn print_configuration_outcome(
    outcome: &ConfigureProjectOutcome,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Human if outcome.changed && outcome.dry_run => {
            println!(
                "Previewed project configuration in {}; no changes were made",
                outcome.project_root.display()
            )
        }
        OutputFormat::Human if outcome.changed => {
            println!(
                "Applied project configuration in {}",
                outcome.project_root.display()
            )
        }
        OutputFormat::Human => println!("Project configuration is already up to date"),
        OutputFormat::Json => println!("{}", serde_json::to_string(outcome)?),
    }
    Ok(())
}

fn load_fields(path: &Path) -> Result<Vec<Field>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read field definition: {}", path.display()))?;
    parse_fields(&contents).with_context(|| format!("invalid field definition: {}", path.display()))
}

fn parse_fields(contents: &str) -> Result<Vec<Field>> {
    let fields: Vec<Field> = serde_json::from_str(contents)?;
    if fields.is_empty() {
        bail!("field definition must contain at least one field");
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_entity_generate_contract() {
        let cli = HeadlessCli::try_parse_from([
            "banna-cli",
            "entity",
            "generate",
            "--project",
            "/tmp/demo",
            "--domain",
            "Acme.Billing",
            "--namespace",
            "Acme.Billing.Invoices",
            "--entity",
            "Invoice",
            "--fields",
            "fields.json",
            "--ui",
            "razor",
            "--no-merge",
            "--dry-run",
            "--non-interactive",
        ])
        .expect("valid CLI arguments should parse");

        let Command::Entity(EntityArgs {
            command: EntityCommand::Generate(args),
        }) = cli.command
        else {
            panic!("entity command should parse");
        };
        assert_eq!(args.entity, "Invoice");
        assert!(args.no_merge);
        assert!(args.dry_run);
        assert!(args.non_interactive);
        assert!(matches!(args.ui, CliUiTarget::Razor));
    }

    #[test]
    fn parses_the_entity_regenerate_contract() {
        let cli = HeadlessCli::try_parse_from([
            "banna-cli",
            "entity",
            "regenerate",
            "--project",
            "/tmp/demo",
            "--entity",
            "Invoice",
            "--no-merge",
            "--dry-run",
            "--non-interactive",
            "--format",
            "json",
        ])
        .expect("valid regenerate arguments should parse");

        let Command::Entity(EntityArgs {
            command: EntityCommand::Regenerate(args),
        }) = cli.command
        else {
            panic!("entity regenerate command should parse");
        };
        assert_eq!(args.entity, "Invoice");
        assert!(args.no_merge);
        assert!(args.dry_run);
        assert!(args.non_interactive);
        assert!(matches!(args.format, OutputFormat::Json));
    }

    #[test]
    fn parses_project_management_commands() {
        let cli = HeadlessCli::try_parse_from([
            "banna-cli",
            "project",
            "add",
            "--path",
            "/tmp/demo",
            "--name",
            "Demo",
            "--format",
            "json",
        ])
        .expect("project add should parse");

        let Command::Project(ProjectArgs {
            command: ProjectCommand::Add(args),
        }) = cli.command
        else {
            panic!("project command should parse");
        };
        assert_eq!(args.name.as_deref(), Some("Demo"));
        assert!(matches!(args.output.format, OutputFormat::Json));
    }

    #[test]
    fn parses_project_configuration() {
        let cli = HeadlessCli::try_parse_from([
            "banna-cli",
            "project",
            "configure",
            "--path",
            "/tmp/demo",
            "--theme",
            "lepton-x",
            "--bootstrap-override",
            "nord",
            "--mobile-ui",
            "react-native",
            "--non-interactive",
            "--dry-run",
        ])
        .expect("project configure should parse");

        let Command::Project(ProjectArgs {
            command: ProjectCommand::Configure(args),
        }) = cli.command
        else {
            panic!("project configure command should parse");
        };
        assert!(matches!(args.theme, Some(CliTheme::LeptonX)));
        assert!(matches!(
            args.bootstrap_override,
            Some(CliBootstrapOverride::Nord)
        ));
        assert!(matches!(args.mobile_ui, Some(CliMobileUi::ReactNative)));
        assert!(args.dry_run);
    }

    #[test]
    fn parses_history_commands() {
        let cli = HeadlessCli::try_parse_from([
            "banna-cli",
            "history",
            "remove",
            "--project",
            "/tmp/demo",
            "--entity",
            "Invoice",
            "--format",
            "json",
        ])
        .expect("history remove should parse");

        let Command::History(HistoryArgs {
            command: HistoryCommand::Remove(args),
        }) = cli.command
        else {
            panic!("history command should parse");
        };
        assert_eq!(args.entity, "Invoice");
        assert!(matches!(args.output.format, OutputFormat::Json));
    }

    #[test]
    fn parses_field_definitions() {
        let fields = parse_fields(
            r#"[{
                "name": "Number",
                "type": "string",
                "required": true,
                "max_length": 64,
                "filterable": true,
                "show_in_ui": true
            }]"#,
        )
        .expect("valid fields should parse");

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "Number");
        assert_eq!(fields[0].max_length, Some(64));
    }

    #[test]
    fn rejects_an_empty_field_definition() {
        let error = parse_fields("[]").expect_err("empty fields should fail");
        assert_eq!(
            error.to_string(),
            "field definition must contain at least one field"
        );
    }
}

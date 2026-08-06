use super::console::{ConsolePrompt, InputHint, prompt, prompt_with_hint};
use super::jobs::PendingGeneration;
use super::workflow::WorkflowState;
use crate::generator;
use crate::helpers::infer_domain_from_src;
use crate::models::{Field, UiTarget};
use crate::utils::pluralize;
use std::path::PathBuf;

fn pop_meta_flags(
    workflow: &WorkflowState,
    default_filter: bool,
    default_show: bool,
) -> (bool, bool) {
    workflow.take_wizard_field_meta((default_filter, default_show))
}

#[derive(Debug)]
enum FieldPhase {
    Name,
    Type {
        name: String,
    },
    Required {
        name: String,
        ftype: String,
    },
    Filterable {
        name: String,
        ftype: String,
        required: bool,
    },
    ShowUi {
        name: String,
        ftype: String,
        required: bool,
        filterable: bool,
    },
    StringMaxLen {
        name: String,
        ftype: String,
        required: bool,
    },
    GuidNavChoice {
        name: String,
        ftype: String,
        required: bool,
    },
    GuidNavName {
        name: String,
        _ftype: String,
        required: bool,
        _will_nav: bool,
    },
    GuidNavNamespace {
        name: String,
        class_name: String,
        required: bool,
        namespace: String,
    },
    GuidNavDisplay {
        name: String,
        class_name: String,
        required: bool,
        namespace: String,
    },
    EnumName {
        name: String,
        required: bool,
    },
    EnumNamespace {
        name: String,
        class_name: String,
        required: bool,
    },
}

#[derive(Debug)]
enum WizardPhase {
    AskEntity,
    AskNamespace,
    Fields(FieldPhase, Vec<Field>),
    AskUi { fields: Vec<Field> },
    AskMigration { fields: Vec<Field> },
    Done,
}

struct WizardState {
    phase: WizardPhase,
    domain: String,
    namespace: String,
    entity: String,
    ui_target: UiTarget,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            phase: WizardPhase::AskEntity,
            domain: String::new(),
            namespace: String::new(),
            entity: String::new(),
            ui_target: UiTarget::None,
        }
    }
}

pub(super) struct EntityWizard {
    root: PathBuf,
    workflow: WorkflowState,
    state: WizardState,
}

impl EntityWizard {
    pub(super) fn new(root: PathBuf, workflow: WorkflowState) -> Self {
        Self {
            root,
            workflow,
            state: WizardState::default(),
        }
    }

    pub(super) fn submit(&mut self, line: String) -> Option<ConsolePrompt> {
        let mut next_prompt: Option<ConsolePrompt> = None;
        let mut done = false;

        let trim_lower = |s: &str| s.trim().to_lowercase();

        match &mut self.state.phase {
            WizardPhase::AskEntity => {
                self.state.entity = line.trim().to_string();
                if self.state.entity.is_empty() {
                    next_prompt = prompt_with_hint(
                        "Entity name cannot be empty. Enter entity name (e.g., Book)",
                        InputHint::Free,
                    );
                } else {
                    let src = self.root.join("src");
                    if let Some(dom) = infer_domain_from_src(&src) {
                        self.state.domain = dom;
                    } else {
                        return prompt(format!(
                            "Could not infer domain from {}. Ensure your DDD layers exist (e.g., *Domain, *Application...)",
                            src.display()
                        ));
                    }

                    let suggested_ns =
                        format!("{}.{}", self.state.domain, pluralize(&self.state.entity));

                    self.state.phase = WizardPhase::AskNamespace;
                    next_prompt = prompt_with_hint(
                        format!("Enter namespace [{suggested_ns}]"),
                        InputHint::Free,
                    );
                }
            }
            WizardPhase::AskNamespace => {
                let ns_input = line.trim();
                let suggested_ns =
                    format!("{}.{}", self.state.domain, pluralize(&self.state.entity));

                self.state.namespace = if ns_input.is_empty() {
                    suggested_ns
                } else {
                    ns_input.to_string()
                };

                self.state.phase = WizardPhase::Fields(FieldPhase::Name, Vec::new());
                next_prompt = prompt_with_hint("Field name (empty to finish)", InputHint::Free);
            }

            WizardPhase::Fields(sub, fields) => match sub {
                FieldPhase::Name => {
                    let name = line.trim().to_string();
                    if name.is_empty() {
                        if fields.is_empty() {
                            return prompt_with_hint(
                                "No fields entered. Field name (empty to cancel)",
                                InputHint::Free,
                            );
                        }
                        let carried = std::mem::take(fields);
                        self.state.phase = WizardPhase::AskUi { fields: carried };

                        let (has_mvc, has_angular) =
                            generator::detect_ui(&self.root, &self.state.domain);
                        if has_mvc && has_angular {
                            next_prompt = prompt_with_hint(
                                "Select UI target: [1] Razor Page, [2] Vue Component, [3] Angular (or leave empty for None)",
                                InputHint::DigitChoice(vec!['1', '2', '3']),
                            );
                        } else if has_mvc {
                            next_prompt = prompt_with_hint(
                                "Select UI target: [1] Razor Page, [2] Vue Component (or leave empty for None)",
                                InputHint::DigitChoice(vec!['1', '2']),
                            );
                        } else if has_angular {
                            next_prompt = prompt_with_hint(
                                "Select UI target: [1] Angular (or leave empty for None)",
                                InputHint::DigitChoice(vec!['1']),
                            );
                        } else {
                            next_prompt = prompt_with_hint(
                                "No UI project found. Continue without UI? (y/n)",
                                InputHint::YesNo,
                            );
                        };
                    } else {
                        *sub = FieldPhase::Type { name };
                        next_prompt = prompt_with_hint(
                            "Field type (1:string, 2:int, L:long, 3:Guid, 4:bool, 5:DateTime, 6:textarea, 7:decimal, 8:DateOnly, 9:TimeOnly, 0:Enum)",
                            InputHint::DigitChoice(vec![
                                '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'l', 'L',
                            ]),
                        );
                    }
                }
                FieldPhase::Type { name } => {
                    let inp = line.trim().to_lowercase();
                    let ftype = match inp.as_str() {
                        "1" | "string" => "string".to_string(),
                        "2" | "int" => "int".to_string(),
                        "l" | "long" => "long".to_string(),
                        "3" | "guid" => "Guid".to_string(),
                        "4" | "bool" => "bool".to_string(),
                        "5" | "datetime" => "DateTime".to_string(),
                        "6" | "textarea" => "textarea".to_string(),
                        "7" | "decimal" => "decimal".to_string(),
                        "8" | "dateonly" => "DateOnly".to_string(),
                        "9" | "timeonly" => "TimeOnly".to_string(),
                        "0" | "enum" => "enum".to_string(),
                        _ => {
                            return prompt_with_hint(
                                "Invalid choice. Pick 1-0/L (1:string, 2:int, L:long, 3:Guid ... 8:DateOnly, 9:TimeOnly, 0:Enum)",
                                InputHint::DigitChoice(vec![
                                    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'l', 'L',
                                ]),
                            );
                        }
                    };

                    *sub = FieldPhase::Required {
                        name: name.clone(),
                        ftype,
                    };
                    next_prompt = prompt_with_hint("Required? (y/n)", InputHint::YesNo);
                }
                FieldPhase::Required { name, ftype } => {
                    let yn = trim_lower(&line);
                    let required = matches!(yn.as_str(), "y" | "yes" | "true" | "1");

                    *sub = FieldPhase::Filterable {
                        name: name.clone(),
                        ftype: ftype.clone(),
                        required,
                    };
                    next_prompt = prompt_with_hint("Filterable? (y/n)", InputHint::YesNo);
                }

                FieldPhase::Filterable {
                    name,
                    ftype,
                    required,
                } => {
                    let yn = trim_lower(&line);
                    let filterable = matches!(yn.as_str(), "y" | "yes" | "true" | "1");
                    *sub = FieldPhase::ShowUi {
                        name: name.clone(),
                        ftype: ftype.clone(),
                        required: *required,
                        filterable,
                    };
                    next_prompt = prompt_with_hint("Show in UI? (y/n)", InputHint::YesNo);
                }
                FieldPhase::ShowUi {
                    name,
                    ftype,
                    required,
                    filterable,
                } => {
                    let yn = trim_lower(&line);
                    let show_ui = matches!(yn.as_str(), "y" | "yes" | "true" | "1");

                    let filt_val: bool = *filterable;

                    if ftype.eq_ignore_ascii_case("string")
                        || ftype.eq_ignore_ascii_case("textarea")
                    {
                        self.workflow.set_wizard_field_meta(filt_val, show_ui);

                        *sub = FieldPhase::StringMaxLen {
                            name: name.clone(),
                            ftype: ftype.clone(),
                            required: *required,
                        };
                        next_prompt =
                            prompt_with_hint("Max length (empty to skip)", InputHint::Free);
                    } else if ftype == "Guid" {
                        self.workflow.set_wizard_field_meta(filt_val, show_ui);

                        *sub = FieldPhase::GuidNavChoice {
                            name: name.clone(),
                            ftype: ftype.clone(),
                            required: *required,
                        };
                        next_prompt =
                            prompt_with_hint("Add as navigation property? (y/n)", InputHint::YesNo);
                    } else if ftype == "enum" {
                        *sub = FieldPhase::EnumName {
                            name: name.clone(),
                            required: *required,
                        };
                        next_prompt = prompt_with_hint("Enum class name", InputHint::Free);
                    } else {
                        fields.push(Field {
                            name: name.clone(),
                            ftype: if ftype == "String" {
                                "string".into()
                            } else {
                                ftype.clone()
                            },
                            required: *required,
                            max_length: None,
                            navigation_display: None,
                            navigation: None,
                            filterable: filt_val,
                            show_in_ui: show_ui,
                        });
                        *sub = FieldPhase::Name;
                        next_prompt =
                            prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                    }
                }
                FieldPhase::StringMaxLen {
                    name,
                    ftype,
                    required,
                } => {
                    let ml = line.trim();
                    let mut max_len: Option<i32> = None;
                    if !ml.is_empty() {
                        if let Ok(n) = ml.parse::<i32>() {
                            max_len = Some(n);
                        } else {
                            next_prompt = prompt_with_hint(
                                "Max length must be an integer (or empty to skip)",
                                InputHint::Free,
                            );
                            return next_prompt;
                        }
                    }
                    let (filt_meta, show_meta) = pop_meta_flags(&self.workflow, false, false);
                    fields.push(Field {
                        name: name.clone(),
                        ftype: ftype.clone(),
                        required: *required,
                        max_length: max_len,
                        navigation_display: None,
                        navigation: None,
                        filterable: filt_meta,
                        show_in_ui: show_meta,
                    });
                    *sub = FieldPhase::Name;
                    next_prompt = prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                }
                FieldPhase::GuidNavChoice {
                    name,
                    ftype,
                    required,
                } => {
                    let yn = trim_lower(&line);
                    let will_nav = matches!(yn.as_str(), "y" | "yes" | "true" | "1");
                    if will_nav {
                        let name_clone = name.clone();
                        let default_name = if name_clone.ends_with("Id") {
                            name_clone[..name_clone.len() - 2].to_string()
                        } else {
                            String::new()
                        };

                        *sub = FieldPhase::GuidNavName {
                            name: name_clone.clone(),
                            _ftype: ftype.clone(),
                            required: *required,
                            _will_nav: will_nav,
                        };
                        if default_name.is_empty() {
                            next_prompt =
                                prompt_with_hint("Navigation class name", InputHint::Free);
                        } else {
                            next_prompt = prompt_with_hint(
                                format!("Navigation class name [{default_name}]"),
                                InputHint::Free,
                            );
                        }
                    } else {
                        let (filt_meta, show_meta) = pop_meta_flags(&self.workflow, false, false);
                        fields.push(Field {
                            name: name.clone(),
                            ftype: "Guid".into(),
                            required: *required,
                            max_length: None,
                            navigation_display: None,
                            navigation: None,
                            filterable: filt_meta,
                            show_in_ui: show_meta,
                        });
                        *sub = FieldPhase::Name;
                        next_prompt =
                            prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                    }
                }
                FieldPhase::GuidNavName {
                    name,
                    _ftype: _,
                    required,
                    ..
                } => {
                    let entered = line.trim();
                    let class_name = if !entered.is_empty() {
                        entered.to_string()
                    } else if name.ends_with("Id") {
                        name[..name.len() - 2].to_string()
                    } else {
                        let (filt_meta, show_meta) = pop_meta_flags(&self.workflow, false, false);
                        fields.push(Field {
                            name: name.clone(),
                            ftype: "Guid".into(),
                            required: *required,
                            max_length: None,
                            navigation_display: None,
                            navigation: None,
                            filterable: filt_meta,
                            show_in_ui: show_meta,
                        });
                        *sub = FieldPhase::Name;
                        return prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                    };
                    *sub = FieldPhase::GuidNavNamespace {
                        name: name.clone(),
                        class_name: class_name.clone(),
                        required: *required,
                        namespace: format!("{}.{}", self.state.domain, pluralize(&class_name)),
                    };
                    let default_ns = format!("{}.{}", self.state.domain, pluralize(&class_name));
                    next_prompt = prompt_with_hint(
                        format!("Namespace for {class} [{default_ns}]", class = class_name),
                        InputHint::Free,
                    );
                }
                FieldPhase::GuidNavNamespace {
                    name,
                    class_name,
                    required,
                    namespace,
                } => {
                    let entered_ns = line.trim();
                    let default_ns = if namespace.is_empty() {
                        format!("{}.{}", self.state.domain, pluralize(class_name))
                    } else {
                        namespace.clone()
                    };
                    let chosen_ns = if entered_ns.is_empty() {
                        default_ns
                    } else {
                        entered_ns.to_string()
                    };

                    let class_name_clone = class_name.clone();
                    *sub = FieldPhase::GuidNavDisplay {
                        name: name.clone(),
                        class_name: class_name_clone.clone(),
                        required: *required,
                        namespace: chosen_ns.clone(),
                    };
                    next_prompt = prompt_with_hint(
                        format!(
                            "Display property for {cls} (empty = Name)",
                            cls = class_name_clone
                        ),
                        InputHint::Free,
                    );
                }
                FieldPhase::GuidNavDisplay {
                    name,
                    class_name,
                    required,
                    namespace,
                } => {
                    let disp_raw = line.trim();
                    let nav_display = if disp_raw.is_empty() {
                        None
                    } else {
                        Some(disp_raw.to_string())
                    };

                    let mut full_nav = format!("{}.{}", namespace, class_name);
                    if let Some(d) = nav_display.as_ref()
                        && !d.is_empty()
                    {
                        full_nav.push('#');
                        full_nav.push_str(d);
                    }

                    let (filt_meta, show_meta) = pop_meta_flags(&self.workflow, false, false);

                    fields.push(Field {
                        name: name.clone(),
                        ftype: "Guid".into(),
                        required: *required,
                        max_length: None,
                        navigation_display: nav_display,
                        navigation: Some(full_nav),
                        filterable: filt_meta,
                        show_in_ui: show_meta,
                    });
                    *sub = FieldPhase::Name;
                    next_prompt = prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                }
                FieldPhase::EnumName { name, required } => {
                    let entered = line.trim();
                    if entered.is_empty() {
                        return prompt_with_hint(
                            "Enum class name cannot be empty",
                            InputHint::Free,
                        );
                    }
                    let class_name = entered.to_string();
                    *sub = FieldPhase::EnumNamespace {
                        name: name.clone(),
                        class_name: class_name.clone(),
                        required: *required,
                    };
                    let default_ns = self.state.namespace.clone();
                    next_prompt = prompt_with_hint(
                        format!("Namespace for {class} [{default_ns}]", class = class_name),
                        InputHint::Free,
                    );
                }
                FieldPhase::EnumNamespace {
                    name,
                    class_name,
                    required,
                } => {
                    let entered_ns = line.trim();
                    let default_ns = self.state.namespace.clone();
                    let chosen_ns = if entered_ns.is_empty() {
                        default_ns
                    } else {
                        entered_ns.to_string()
                    };

                    let full_nav = format!("{}.{}", chosen_ns, class_name);
                    let (filt_meta, show_meta) = pop_meta_flags(&self.workflow, false, false);

                    fields.push(Field {
                        name: name.clone(),
                        ftype: "enum".into(),
                        required: *required,
                        max_length: None,
                        navigation_display: None,
                        navigation: Some(full_nav),
                        filterable: filt_meta,
                        show_in_ui: show_meta,
                    });
                    *sub = FieldPhase::Name;
                    next_prompt = prompt_with_hint("Field name (empty to finish)", InputHint::Free);
                }
            },
            WizardPhase::AskUi { fields } => {
                let (has_mvc, has_angular) = generator::detect_ui(&self.root, &self.state.domain);
                let ans = line.trim().to_lowercase();

                if has_mvc || has_angular {
                    match ans.as_str() {
                        "1" if has_mvc => self.state.ui_target = UiTarget::Razor,
                        "2" if has_mvc => self.state.ui_target = UiTarget::Vue,
                        "3" if has_mvc && has_angular => self.state.ui_target = UiTarget::Angular,
                        "1" | "2" if has_angular && !has_mvc => {
                            self.state.ui_target = UiTarget::Angular
                        }
                        "" => self.state.ui_target = UiTarget::None,
                        _ => {
                            next_prompt = if has_mvc && has_angular {
                                prompt_with_hint(
                                    "Select UI target: [1] MVC, [2] Angular (or leave empty for None)",
                                    InputHint::DigitChoice(vec!['1', '2', '3']),
                                )
                            } else if has_mvc {
                                prompt_with_hint(
                                    "Select UI target: [1] MVC (or leave empty for None)",
                                    InputHint::DigitChoice(vec!['1']),
                                )
                            } else {
                                prompt_with_hint(
                                    "Select UI target: [2] Angular (or leave empty for None)",
                                    InputHint::DigitChoice(vec!['1']),
                                )
                            };
                            return next_prompt;
                        }
                    }
                } else if matches!(ans.as_str(), "y" | "yes") {
                    self.state.ui_target = UiTarget::None;
                } else {
                    return prompt_with_hint(
                        "No UI project found. Continue without UI? (y/n)",
                        InputHint::YesNo,
                    );
                }

                let carried = fields.clone();
                self.state.phase = WizardPhase::AskMigration { fields: carried };
                next_prompt =
                    prompt_with_hint("Run database migration now? (y/n)", InputHint::YesNo);
            }
            WizardPhase::AskMigration { fields } => {
                let ans = line.trim().to_lowercase();
                let run_migration = matches!(ans.as_str(), "y" | "yes");

                let fields_cloned = fields.clone();
                let ui_target = self.state.ui_target;

                self.workflow.complete_generation(PendingGeneration {
                    root: self.root.clone(),
                    domain: self.state.domain.clone(),
                    namespace: self.state.namespace.clone(),
                    entity: self.state.entity.clone(),
                    fields: fields_cloned,
                    ui_target,
                    run_migration,
                });

                self.state.phase = WizardPhase::Done;
                done = true;
            }
            WizardPhase::Done => {
                done = true;
            }
        }

        if done {
            return None;
        }
        next_prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (tempfile::TempDir, WorkflowState, EntityWizard) {
        let directory = tempfile::tempdir().expect("wizard fixture");
        fs::create_dir_all(directory.path().join("src/Acme.BookStore.Domain"))
            .expect("domain fixture");
        let workflow = WorkflowState::default();
        let wizard = EntityWizard::new(directory.path().to_path_buf(), workflow.clone());
        (directory, workflow, wizard)
    }

    #[test]
    fn rejects_an_empty_entity_without_advancing() {
        let (_directory, _workflow, mut wizard) = fixture();

        let next = wizard.submit("   ".into()).expect("validation prompt");

        assert!(next.text.contains("cannot be empty"));
        assert!(matches!(next.hint, InputHint::Free));
        assert!(matches!(wizard.state.phase, WizardPhase::AskEntity));
    }

    #[test]
    fn accepts_the_default_namespace_and_starts_field_collection() {
        let (_directory, _workflow, mut wizard) = fixture();

        let namespace = wizard.submit("Book".into()).expect("namespace prompt");
        assert_eq!(namespace.text, "Enter namespace [Acme.BookStore.Books]");

        let field = wizard.submit(String::new()).expect("field prompt");
        assert_eq!(field.text, "Field name (empty to finish)");
        assert_eq!(wizard.state.namespace, "Acme.BookStore.Books");
        assert!(matches!(
            wizard.state.phase,
            WizardPhase::Fields(FieldPhase::Name, _)
        ));
    }

    #[test]
    fn completes_a_minimal_headless_generation_request() {
        let (directory, workflow, mut wizard) = fixture();
        for answer in ["Book", "", "Title", "1", "y", "n", "y", "100", ""] {
            assert!(wizard.submit(answer.into()).is_some());
        }

        let migration = wizard.submit("y".into()).expect("migration prompt");
        assert_eq!(migration.text, "Run database migration now? (y/n)");
        assert!(wizard.submit("n".into()).is_none());

        let request = workflow.take_generation().expect("generation request");
        assert_eq!(request.root, directory.path());
        assert_eq!(request.domain, "Acme.BookStore");
        assert_eq!(request.namespace, "Acme.BookStore.Books");
        assert_eq!(request.entity, "Book");
        assert_eq!(request.fields.len(), 1);
        assert_eq!(request.fields[0].name, "Title");
        assert_eq!(request.fields[0].max_length, Some(100));
        assert!(request.fields[0].required);
        assert!(request.fields[0].show_in_ui);
        assert!(!request.run_migration);
    }
}

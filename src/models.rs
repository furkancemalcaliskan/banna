use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ftype: String,
    pub required: bool,
    #[serde(default)]
    pub navigation_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<String>,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub show_in_ui: bool,
}

impl Field {
    pub fn nullable_filter_type(&self) -> String {
        match self.ftype.as_str() {
            "textarea" => "string?".into(),
            "decimal" => "decimal?".into(),
            "string" => "string?".into(),
            "int" | "long" | "bool" | "Guid" | "DateTime" | "DateOnly" | "TimeOnly" => {
                format!("{}?", self.ftype)
            }
            "enum" => {
                if let Some(nav) = &self.navigation {
                    let cls = nav.rsplit('.').next().unwrap_or(nav);
                    format!("{}?", cls)
                } else {
                    "Enum?".into()
                }
            }
            _ => format!("{}?", self.ftype),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum UiTarget {
    #[default]
    None,
    Razor,
    Vue,
    Angular,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AbpTheme {
    #[default]
    Basic,
    LeptonX,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BootstrapOverride {
    #[default]
    None,
    Modern,
    Nord,
    Solarized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub domain: String,
    pub name: String,
    pub namespace: String,
    pub fields: Vec<Field>,
    #[serde(default)]
    pub ui_target: UiTarget,
    pub generated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub project_name: String,
    pub project_dir: String,
    #[serde(default)]
    pub theme: AbpTheme,
    #[serde(default)]
    pub bootstrap_override: BootstrapOverride,
    #[serde(default)]
    pub mobile_ui: MobileUi,
    pub entities: Vec<EntityRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MobileUi {
    #[default]
    None,
    ReactNative,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalIndex {
    pub projects: Vec<ProjectRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectRef {
    pub project_name: String,
    pub project_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_metadata_defaults_remain_backward_compatible() {
        let project: ProjectRecord = serde_json::from_str(
            r#"{
                "project_name": "Demo",
                "project_dir": "/tmp/demo",
                "entities": []
            }"#,
        )
        .expect("minimal project record should deserialize");

        assert_eq!(project.theme, AbpTheme::Basic);
        assert_eq!(project.bootstrap_override, BootstrapOverride::None);
        assert_eq!(project.mobile_ui, MobileUi::None);
    }

    #[test]
    fn mobile_ui_uses_kebab_case_in_persisted_data() {
        let value =
            serde_json::to_string(&MobileUi::ReactNative).expect("mobile UI should serialize");
        assert_eq!(value, r#""react-native""#);
    }
}

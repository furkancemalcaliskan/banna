use crate::models::{AbpTheme, BootstrapOverride, MobileUi, UiTarget};

pub(super) fn sanitize_ui_choice(choice: &str, has_mvc: bool, has_angular: bool) -> UiTarget {
    let choice = choice.to_ascii_lowercase();
    match choice.as_str() {
        "razor" | "1" if has_mvc => UiTarget::Razor,
        "vue" | "2" if has_mvc => UiTarget::Vue,
        "angular" | "3" if has_angular && has_mvc => UiTarget::Angular,
        "angular" | "2" if has_angular && !has_mvc => UiTarget::Angular,
        _ => UiTarget::None,
    }
}

pub(super) fn cycle_next_ui(current: UiTarget, has_mvc: bool, has_angular: bool) -> UiTarget {
    let mut allowed = vec![UiTarget::None];
    if has_mvc {
        allowed.push(UiTarget::Razor);
        allowed.push(UiTarget::Vue);
    }
    if has_angular {
        allowed.push(UiTarget::Angular);
    }
    let index = allowed
        .iter()
        .position(|target| *target == current)
        .unwrap_or(0);
    allowed[(index + 1) % allowed.len()]
}

pub(super) fn sanitize_theme_choice(choice: &str) -> AbpTheme {
    match choice.to_ascii_lowercase().as_str() {
        "leptonx" | "leptonx lite" | "leptonxlite" | "lepton" | "2" => AbpTheme::LeptonX,
        _ => AbpTheme::Basic,
    }
}

pub(super) fn cycle_next_theme(current: AbpTheme) -> AbpTheme {
    match current {
        AbpTheme::Basic => AbpTheme::LeptonX,
        AbpTheme::LeptonX => AbpTheme::Basic,
    }
}

pub(super) fn theme_label(theme: AbpTheme) -> &'static str {
    match theme {
        AbpTheme::Basic => "Basic Theme",
        AbpTheme::LeptonX => "LeptonX",
    }
}

pub(super) fn override_label(override_css: BootstrapOverride) -> &'static str {
    match override_css {
        BootstrapOverride::None => "None",
        BootstrapOverride::Modern => "Modern",
        BootstrapOverride::Nord => "Nord",
        BootstrapOverride::Solarized => "Solarized",
    }
}

pub(super) fn sanitize_override_choice(choice: &str) -> BootstrapOverride {
    match choice.to_ascii_lowercase().as_str() {
        "modern" | "1" => BootstrapOverride::Modern,
        "nord" | "2" => BootstrapOverride::Nord,
        "solarized" | "3" => BootstrapOverride::Solarized,
        _ => BootstrapOverride::None,
    }
}

pub(super) fn cycle_next_override(current: BootstrapOverride) -> BootstrapOverride {
    match current {
        BootstrapOverride::None => BootstrapOverride::Modern,
        BootstrapOverride::Modern => BootstrapOverride::Nord,
        BootstrapOverride::Nord => BootstrapOverride::Solarized,
        BootstrapOverride::Solarized => BootstrapOverride::None,
    }
}

pub(super) fn mobile_ui_label(mobile_ui: MobileUi) -> &'static str {
    match mobile_ui {
        MobileUi::None => "None",
        MobileUi::ReactNative => "React Native",
    }
}

pub(super) fn sanitize_mobile_ui_choice(choice: &str) -> MobileUi {
    match choice.to_ascii_lowercase().as_str() {
        "react native" | "react-native" | "rn" | "1" => MobileUi::ReactNative,
        _ => MobileUi::None,
    }
}

pub(super) fn cycle_next_mobile_ui(current: MobileUi) -> MobileUi {
    match current {
        MobileUi::None => MobileUi::ReactNative,
        MobileUi::ReactNative => MobileUi::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_ui_choices_against_detected_hosts() {
        assert_eq!(sanitize_ui_choice("razor", true, false), UiTarget::Razor);
        assert_eq!(sanitize_ui_choice("vue", true, false), UiTarget::Vue);
        assert_eq!(
            sanitize_ui_choice("angular", false, true),
            UiTarget::Angular
        );
        assert_eq!(sanitize_ui_choice("angular", true, false), UiTarget::None);
        assert_eq!(sanitize_ui_choice("razor", false, true), UiTarget::None);
    }

    #[test]
    fn cycles_only_through_available_ui_targets() {
        assert_eq!(cycle_next_ui(UiTarget::None, true, false), UiTarget::Razor);
        assert_eq!(cycle_next_ui(UiTarget::Razor, true, false), UiTarget::Vue);
        assert_eq!(cycle_next_ui(UiTarget::Vue, true, false), UiTarget::None);
        assert_eq!(
            cycle_next_ui(UiTarget::None, false, true),
            UiTarget::Angular
        );
    }

    #[test]
    fn preserves_theme_and_mobile_choice_aliases() {
        assert_eq!(sanitize_theme_choice("lepton"), AbpTheme::LeptonX);
        assert_eq!(sanitize_override_choice("2"), BootstrapOverride::Nord);
        assert_eq!(sanitize_mobile_ui_choice("rn"), MobileUi::ReactNative);
    }
}

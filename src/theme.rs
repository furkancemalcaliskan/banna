use once_cell::sync::Lazy;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

#[derive(Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub surface2: Color,
    pub fg: Color,
    pub muted: Color,
    pub border: Color,
    pub _border_sel: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub success: Color,
    pub _warning: Color,
    pub danger: Color,
    pub info: Color,
}

pub static THEME: Lazy<Theme> = Lazy::new(|| Theme {
    bg: Color::Rgb(7, 10, 18),
    surface: Color::Rgb(16, 21, 36),
    surface2: Color::Rgb(22, 30, 48),
    fg: Color::Rgb(231, 242, 255),
    muted: Color::Rgb(134, 152, 184),
    border: Color::Rgb(55, 86, 150),
    _border_sel: Color::Rgb(136, 196, 255),
    accent: Color::Rgb(116, 190, 255),
    accent_alt: Color::Rgb(255, 141, 207),
    success: Color::Rgb(108, 255, 207),
    _warning: Color::Rgb(255, 212, 136),
    danger: Color::Rgb(255, 110, 132),
    info: Color::Rgb(144, 215, 255),
});

pub fn block<T: Into<String>>(title: T) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title.into()),
            Style::default()
                .fg(THEME.accent)
                .bg(THEME.bg)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.border))
        .style(Style::default().bg(THEME.surface).fg(THEME.fg))
}

pub fn block_muted<T: Into<String>>(title: T) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title.into()),
            Style::default()
                .fg(THEME.accent_alt)
                .bg(THEME.bg)
                .add_modifier(Modifier::ITALIC),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.border))
        .style(Style::default().bg(THEME.surface2).fg(THEME.fg))
}

pub fn list_highlight() -> Style {
    Style::default()
        .bg(THEME.accent)
        .fg(THEME.bg)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC)
}

pub fn faint() -> Style {
    Style::default()
        .fg(THEME.muted)
        .add_modifier(Modifier::ITALIC)
}

pub fn accent() -> Style {
    Style::default()
        .fg(THEME.accent)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC)
}

pub fn danger() -> Style {
    Style::default()
        .fg(THEME.danger)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC)
}

pub fn success() -> Style {
    Style::default()
        .fg(THEME.success)
        .add_modifier(Modifier::BOLD | Modifier::ITALIC)
}

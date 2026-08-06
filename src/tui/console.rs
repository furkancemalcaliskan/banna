#[derive(Clone, Debug, Default)]
pub(super) enum InputHint {
    #[default]
    Free,
    YesNo,
    DigitChoice(Vec<char>),
}

pub(super) struct ConsolePrompt {
    pub(super) text: String,
    pub(super) hint: InputHint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConsoleMode {
    Wizard,
    Logs,
}

pub(super) struct InlineConsole {
    pub(super) title: String,
    pub(super) lines: Vec<String>,
    pub(super) input: String,
    pub(super) on_submit: Option<Box<dyn FnMut(String) -> Option<ConsolePrompt> + Send>>,
    pub(super) on_cancel: Option<Box<dyn FnMut() + Send>>,
    pub(super) visible: bool,
    pub(super) input_visible: bool,
    pub(super) input_mask: bool,
    pub(super) input_label: Option<String>,
    pub(super) scroll: u16,
    pub(super) auto_scroll: bool,
    pub(super) view_height: u16,
    pub(super) mode: ConsoleMode,
    pub(super) input_hint: InputHint,
}

impl InlineConsole {
    pub(super) fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            input: String::new(),
            on_submit: None,
            on_cancel: None,
            visible: false,
            input_visible: true,
            input_mask: false,
            input_label: None,
            scroll: 0,
            auto_scroll: true,
            view_height: 0,
            mode: ConsoleMode::Wizard,
            input_hint: InputHint::default(),
        }
    }

    pub(super) fn new_logs<T: Into<String>>(title: T) -> Self {
        Self {
            mode: ConsoleMode::Logs,
            input_visible: false,
            ..Self::new(title)
        }
    }

    pub(super) fn open(&mut self) {
        self.lines.clear();
        self.input.clear();
        self.visible = true;
        self.input_visible = matches!(self.mode, ConsoleMode::Wizard);
        self.input_mask = false;
        self.input_label = None;
        self.scroll = 0;
        self.auto_scroll = true;
        self.input_hint = InputHint::default();
    }

    pub(super) fn close(&mut self) {
        self.visible = false;
        self.lines.clear();
        self.input.clear();
        self.input_visible = matches!(self.mode, ConsoleMode::Wizard);
        self.input_mask = false;
        self.input_label = None;
        self.scroll = 0;
        self.auto_scroll = true;
        self.view_height = 0;
        self.input_hint = InputHint::default();
    }

    pub(super) fn println<S: Into<String>>(&mut self, line: S) {
        self.lines.push(line.into());
        self.auto_scroll = true;
    }

    pub(super) fn max_offset(&self) -> u16 {
        if self.view_height == 0 {
            return 0;
        }
        self.lines
            .len()
            .saturating_sub(usize::from(self.view_height))
            .try_into()
            .unwrap_or(u16::MAX)
    }
}

pub(super) fn prompt<S: Into<String>>(message: S) -> Option<ConsolePrompt> {
    prompt_with_hint(message, InputHint::Free)
}

pub(super) fn prompt_with_hint<S: Into<String>>(
    message: S,
    hint: InputHint,
) -> Option<ConsolePrompt> {
    Some(ConsolePrompt {
        text: message.into(),
        hint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_scroll_bounds_without_underflow() {
        let mut console = InlineConsole::new_logs("Logs");
        console.view_height = 2;
        console.println("one");
        assert_eq!(console.max_offset(), 0);
        console.println("two");
        console.println("three");
        assert_eq!(console.max_offset(), 1);
    }

    #[test]
    fn closing_resets_transient_console_state() {
        let mut console = InlineConsole::new("Wizard");
        console.open();
        console.println("line");
        console.input.push_str("answer");
        console.close();

        assert!(!console.visible);
        assert!(console.lines.is_empty());
        assert!(console.input.is_empty());
    }

    #[test]
    fn prompt_keeps_text_and_input_hint_together() {
        let next = prompt_with_hint("Continue?", InputHint::YesNo).expect("prompt");

        assert_eq!(next.text, "Continue?");
        assert!(matches!(next.hint, InputHint::YesNo));
    }
}

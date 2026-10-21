use super::cli_styles::{BOLD_CYAN, BOLD_GREEN, Colorize};
use super::output;

pub struct HelpBuilder {
    title: String,
    description: Option<String>,
    sections: Vec<Section>,
    tip: Option<String>,
}

pub struct HelpSection<'a> {
    help: &'a mut HelpBuilder,
    index: usize,
}
struct Section {
    title: String,
    entries: Vec<(String, String)>,
}

impl HelpBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            sections: Vec::new(),
            tip: None,
        }
    }
    pub fn description(&mut self, text: impl Into<String>) -> &mut Self {
        self.description = Some(text.into());
        self
    }

    /// Add an example to a dedicated `EXAMPLES` section.
    pub fn example(&mut self, text: impl Into<String>) -> &mut Self {
        let text = text.into();
        if let Some(section) = self.sections.iter_mut().find(|s| s.title == "EXAMPLES") {
            section.entries.push((text, String::new()));
        } else {
            self.sections.push(Section {
                title: "EXAMPLES".to_owned(),
                entries: vec![(text, String::new())],
            });
        }
        self
    }
    pub fn note(&mut self, text: impl Into<String>) -> &mut Self {
        let text = text.into();
        match &mut self.description {
            Some(description) if !description.is_empty() => {
                description.push('\n');
                description.push_str(&text);
            }
            Some(description) => description.push_str(&text),
            None => self.description = Some(text),
        }
        self
    }
    pub fn section(&mut self, title: impl Into<String>) -> HelpSection<'_> {
        self.sections.push(Section {
            title: title.into(),
            entries: Vec::new(),
        });
        HelpSection {
            index: self.sections.len() - 1,
            help: self,
        }
    }
    pub fn tip(&mut self, text: impl Into<String>) -> &mut Self {
        self.tip = Some(text.into());
        self
    }

    pub fn render(&self) {
        if output::mode() == output::Mode::Json {
            let sections: Vec<_> = self.sections.iter().map(|s| serde_json::json!({"title": s.title, "items": s.entries.iter().map(|(n,d)| serde_json::json!({"name":n,"description":d})).collect::<Vec<_>>() })).collect();
            let event = serde_json::json!({
                "schema_version": 1,
                "event": "help",
                "title": self.title,
                "description": self.description,
                "sections": sections,
                "tip": self.tip
            });
            output::write_stdout_raw(&format!("{event}\n"));
            return;
        }
        let mut rendered = String::new();
        rendered.push_str(&format!("{}\n", self.title.style(BOLD_GREEN)));
        if let Some(d) = &self.description {
            rendered.push_str(&format!("{}\n\n", d));
        }
        for s in &self.sections {
            rendered.push_str(&format!("{}:\n", s.title.style(BOLD_GREEN)));
            let width = s.entries.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
            for (n, d) in &s.entries {
                if d.is_empty() {
                    rendered.push_str(&format!("    {}\n", n.style(BOLD_CYAN)));
                } else {
                    rendered.push_str(&format!(
                        "    {}  {d}\n",
                        format!("{n:<width$}").style(BOLD_CYAN)
                    ));
                }
            }
            rendered.push('\n');
        }
        if let Some(t) = &self.tip {
            rendered.push_str(&format!("{}:\n    {t}\n", "TIP".style(BOLD_GREEN)));
        }
        output::write_stdout(&rendered);
    }
}
impl HelpSection<'_> {
    pub fn item(self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.help.sections[self.index]
            .entries
            .push((name.into(), description.into()));
        self
    }
    pub fn example(self, command: impl Into<String>) -> Self {
        self.item(command, "")
    }
}

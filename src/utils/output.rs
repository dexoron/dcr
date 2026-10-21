//! Central rendering policy for user-facing CLI events.
use std::io::{self, IsTerminal, Write};
use std::sync::OnceLock;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Mode {
    #[default]
    Human,
    Plain,
    Json,
}

#[derive(Default)]
pub struct Options {
    pub mode: Mode,
    pub quiet: bool,
    pub verbose: bool,
    pub color: Option<bool>,
}

static OPTIONS: OnceLock<Options> = OnceLock::new();

/// Consume global options without touching arguments after `--`.
pub fn configure(args: Vec<String>) -> Result<Vec<String>, String> {
    let mut options = Options::default();
    let mut remaining = Vec::new();
    let mut args = args.into_iter();
    if let Some(program) = args.next() {
        remaining.push(program);
    }
    while let Some(arg) = args.next() {
        if arg == "--" {
            remaining.push(arg);
            remaining.extend(args);
            break;
        }
        match arg.as_str() {
            "--quiet" => options.quiet = true,
            "--verbose" => options.verbose = true,
            "--output" | "--color" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                set_option(&mut options, &arg, &value)?;
            }
            _ if arg.starts_with("--output=") || arg.starts_with("--color=") => {
                let (key, value) = arg.split_once('=').unwrap();
                set_option(&mut options, key, value)?;
            }
            _ => {
                remaining.push(arg);
                remaining.extend(args);
                break;
            }
        }
    }
    OPTIONS
        .set(options)
        .map_err(|_| "output already configured".to_owned())?;
    // Help is styled before it reaches the output writer.  Keep owo-colors'
    // global decision in sync with DCR's explicit color policy; otherwise
    // `--color always` can color the DCR banner but leave help sections plain.
    let configured = OPTIONS.get().expect("output options were just configured");
    let force_color = configured.color.or_else(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            Some(false)
        } else {
            None
        }
    });
    if let Some(enabled) = force_color {
        owo_colors::set_override(enabled);
    }
    Ok(remaining)
}

fn set_option(options: &mut Options, key: &str, value: &str) -> Result<(), String> {
    match (key, value) {
        ("--output", "human") => options.mode = Mode::Human,
        ("--output", "plain") => options.mode = Mode::Plain,
        ("--output", "json") => options.mode = Mode::Json,
        ("--color", "auto") => options.color = None,
        ("--color", "always") => options.color = Some(true),
        ("--color", "never") => options.color = Some(false),
        _ => return Err(format!("invalid value `{value}` for {key}")),
    }
    Ok(())
}

pub fn emit(kind: &str, message: &str) {
    let defaults = Options::default();
    let options = OPTIONS.get().unwrap_or(&defaults);
    if options.quiet && !is_quiet_visible(kind) {
        return;
    }
    if matches!(kind, "debug" | "trace") && !options.verbose {
        return;
    }
    let mut stderr = io::stderr().lock();
    let _ = render(
        &mut stderr,
        options,
        kind,
        message,
        io::stderr().is_terminal(),
    );
}

/// Emit a structured event while retaining the human-readable diagnostic form.
pub fn emit_event(event: &str, fields: serde_json::Value, human: &str) {
    let defaults = Options::default();
    let options = OPTIONS.get().unwrap_or(&defaults);
    if options.quiet && !is_quiet_visible(event) {
        return;
    }
    if options.mode == Mode::Json {
        let mut object = match fields {
            serde_json::Value::Object(object) => object,
            _ => serde_json::Map::new(),
        };
        object.insert("schema_version".into(), serde_json::json!(1));
        object.insert("event".into(), serde_json::json!(event));
        let mut stderr = io::stderr().lock();
        let _ = writeln!(stderr, "{}", serde_json::Value::Object(object));
    } else if event == "compiler_output" {
        emit("compiler_output", human);
    } else {
        emit("build", human);
    }
}

fn is_quiet_visible(kind: &str) -> bool {
    matches!(kind, "error" | "warning" | "compiler_output")
}

pub fn mode() -> Mode {
    OPTIONS.get().map_or(Mode::Human, |options| options.mode)
}

pub fn verbose() -> bool {
    OPTIONS.get().is_some_and(|options| options.verbose)
}

pub fn colors_enabled(terminal: bool) -> bool {
    let defaults = Options::default();
    let options = OPTIONS.get().unwrap_or(&defaults);
    colors_enabled_for(options, terminal)
}

fn colors_enabled_for(options: &Options, terminal: bool) -> bool {
    options.mode == Mode::Human
        && options
            .color
            .unwrap_or(terminal && std::env::var_os("NO_COLOR").is_none())
}

pub fn progress_label(action: &str, subject: &str) -> String {
    let text = format!("{action:<12} {subject}");
    if !colors_enabled(true) {
        return text;
    }
    let code = match action.to_ascii_lowercase().as_str() {
        "project" | "target" | "dep" | "pack" | "ready" | "profile" => 36,
        _ => 32,
    };
    format!("\x1b[1;{code}m{action:>12}\x1b[0m {subject}")
}

/// A compact fluent writer for user-facing status output.
///
/// Commands can build a complete message without repeating the output macro:
/// `output::ui().status("Compiling", name).detail("release").finish();`
#[allow(dead_code)]
pub struct Ui {
    lines: Vec<String>,
}

#[allow(dead_code)]
pub fn ui() -> Ui {
    Ui { lines: Vec::new() }
}

pub fn say(message: impl AsRef<str>) {
    write_stdout(&format!("{}\n", message.as_ref()));
}
pub fn status(action: impl AsRef<str>, subject: impl AsRef<str>) {
    emit(
        "status",
        &format!("{} {}", action.as_ref(), subject.as_ref()),
    );
}
pub fn success(message: impl AsRef<str>) {
    emit("success", message.as_ref());
}
pub fn warning(message: impl AsRef<str>) {
    emit("warning", message.as_ref());
}
pub fn info(message: impl AsRef<str>) {
    emit("info", message.as_ref());
}
pub fn error(message: impl AsRef<str>) {
    emit("error", message.as_ref());
}
pub fn created(subject: impl AsRef<str>) {
    emit("success", &format!("Created {}", subject.as_ref()));
}
pub fn removed(subject: impl AsRef<str>) {
    emit("success", &format!("Removed {}", subject.as_ref()));
}

pub fn compiler_output(stream: &str, text: &str) {
    if mode() == Mode::Json {
        emit_event(
            "compiler_output",
            serde_json::json!({"stream": stream, "text": text}),
            "",
        );
    } else if stream == "stderr" {
        emit("compiler_output", text);
    } else {
        write_stdout(text);
    }
}

#[macro_export]
macro_rules! say { () => { $crate::utils::output::say("") }; ($($arg:tt)*) => { $crate::utils::output::say(format!($($arg)*)) }; }
#[macro_export]
macro_rules! status { ($action:expr, $($arg:tt)*) => { $crate::utils::output::status($action, format!($($arg)*)) }; }
#[macro_export]
macro_rules! success { ($($arg:tt)*) => { $crate::utils::output::success(format!($($arg)*)) }; }

impl Ui {
    #[allow(dead_code)]
    pub fn status(mut self, action: impl Into<String>, subject: impl Into<String>) -> Self {
        self.lines
            .push(format!("{} {}", action.into(), subject.into()));
        self
    }
    #[allow(dead_code)]
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        if let Some(line) = self.lines.last_mut() {
            line.push_str(&format!(" ({})", detail.into()));
        }
        self
    }
    #[allow(dead_code)]
    pub fn item(mut self, item: impl Into<String>) -> Self {
        self.lines.push(format!("  {}", item.into()));
        self
    }
    #[allow(dead_code)]
    pub fn finish(self) {
        emit("status", &self.lines.join("\n"));
    }
}

#[macro_export]
macro_rules! output_warning {
    ($($arg:tt)*) => { $crate::utils::output::emit("warning", &format!($($arg)*)) };
}
#[macro_export]
macro_rules! output_error {
    ($($arg:tt)*) => { $crate::utils::output::emit("error", &format!($($arg)*)) };
}
#[macro_export]
macro_rules! output_info {
    ($($arg:tt)*) => { $crate::utils::output::emit("info", &format!($($arg)*)) };
}
#[macro_export]
macro_rules! output_success {
    ($($arg:tt)*) => { $crate::utils::output::emit("success", &format!($($arg)*)) };
}

// Namespaced aliases avoid collisions with the technical `log` macros.
#[allow(unused_imports)]
pub use crate::{
    output_error as error, output_info as info, output_success as success,
    output_warning as warning,
};

/// User-facing stdout macros. Data/help output belongs on stdout; diagnostics
/// and build status use `emit` on stderr.
#[macro_export]
macro_rules! output_println {
    () => { $crate::utils::output::write_stdout("\n") };
    ($($arg:tt)*) => { $crate::utils::output::write_stdout(&format!("{}\n", format_args!($($arg)*))) };
}

#[macro_export]
macro_rules! output_print {
    ($($arg:tt)*) => { $crate::utils::output::write_stdout(&format!($($arg)*)) };
}

pub fn write_stdout(text: &str) {
    let defaults = Options::default();
    let options = OPTIONS.get().unwrap_or(&defaults);
    let colored = colors_enabled(io::stdout().is_terminal());
    let has_ansi = text.contains('\x1b');
    let rendered = if colored || !has_ansi {
        text.to_owned()
    } else {
        strip_ansi(text)
    };
    let mut stdout = io::stdout().lock();
    let result = if options.mode == Mode::Json {
        writeln!(
            stdout,
            "{}",
            serde_json::json!({"schema_version": 1, "event": "output", "text": rendered})
        )
    } else {
        stdout.write_all(rendered.as_bytes())
    };
    if let Err(error) = result
        && error.kind() == io::ErrorKind::BrokenPipe
    {
        std::process::exit(0);
    }
}

pub fn write_stdout_raw(text: &str) {
    let mut stdout = io::stdout().lock();
    if let Err(error) = stdout.write_all(text.as_bytes())
        && error.kind() == io::ErrorKind::BrokenPipe
    {
        std::process::exit(0);
    }
}

fn strip_ansi(text: &str) -> String {
    let mut chars = text.chars().peekable();
    let mut clean = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for ch in chars.by_ref() {
                if ('@'..='~').contains(&ch) {
                    break;
                }
            }
        } else {
            clean.push(ch);
        }
    }
    clean
}

#[macro_export]
macro_rules! output_sprintln {
    ($style:expr, $($arg:tt)*) => {{
        use $crate::utils::cli_styles::Colorize as _;
        $crate::output_println!("{}", format!($($arg)*).style($style));
    }};
}

/// Render a help heading through the same output policy as the rest of the CLI.
/// Keeping this separate from ordinary styled output makes help formatting easy
/// to change globally without touching every command.
#[macro_export]
macro_rules! output_help_heading {
    ($style:expr, $($arg:tt)*) => {{
        use $crate::utils::cli_styles::Colorize as _;
        $crate::utils::output::write_stdout(&format!("{}\n", format!($($arg)*).style($style)));
    }};
}

#[macro_export]
macro_rules! output_sprint {
    ($style:expr, $($arg:tt)*) => {{
        use $crate::utils::cli_styles::Colorize as _;
        $crate::output_print!("{}", format!($($arg)*).style($style));
    }};
}

fn render(
    writer: &mut impl Write,
    options: &Options,
    kind: &str,
    message: &str,
    terminal: bool,
) -> io::Result<()> {
    if options.mode == Mode::Json {
        return writeln!(
            writer,
            "{}",
            serde_json::json!({
                "schema_version": 1, "event": kind, "message": message
            })
        );
    }
    let color = colors_enabled_for(options, terminal);
    if kind == "compiler_output" {
        let message = strip_ansi(message);
        for line in message.lines() {
            let code = if line.contains("error:") || line.contains("fatal error") {
                31
            } else if line.contains("warning:") {
                33
            } else if line.contains("note:") {
                36
            } else {
                37
            };
            if color {
                writeln!(writer, "\x1b[{code}m{line}\x1b[0m")?;
            } else {
                writeln!(writer, "{line}")?;
            }
        }
        return Ok(());
    }
    if matches!(kind, "status" | "success" | "build") {
        let message = strip_ansi(message);
        let (action, subject) = message.split_once(' ').unwrap_or((&message, ""));
        if color {
            let code = match action.to_ascii_lowercase().as_str() {
                "project" | "target" | "dep" | "pack" | "ready" | "profile" => 36,
                "compile" | "compiling" | "created" | "creating" | "finished" | "done"
                | "removed" | "initialized" | "added" | "checked" | "skipped" | "format"
                | "formatting" | "lint" | "linting" | "run" | "up" | "loaded" => 32,
                _ if kind == "success" => 32,
                _ => 36,
            };
            return writeln!(writer, "\x1b[1;{code}m{action:>12}\x1b[0m {subject}");
        }
        return writeln!(writer, "{action:>12} {subject}");
    }
    if color {
        let code = match kind {
            "error" => 31,
            "warning" => 33,
            _ => 36,
        };
        writeln!(writer, "\x1b[1;{code}m{kind}:\x1b[0m {message}")
    } else {
        writeln!(writer, "{kind}: {}", strip_ansi(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_colors_status_action_and_diagnostics() {
        let options = Options {
            color: Some(true),
            ..Options::default()
        };
        for (kind, message, expected) in [
            (
                "status",
                "Compiling hello",
                "\x1b[1;32m   Compiling\x1b[0m hello\n",
            ),
            (
                "success",
                "Created project",
                "\x1b[1;32m     Created\x1b[0m project\n",
            ),
            ("warning", "notice", "\x1b[1;33mwarning:\x1b[0m notice\n"),
            ("error", "failed", "\x1b[1;31merror:\x1b[0m failed\n"),
        ] {
            let mut bytes = Vec::new();
            render(&mut bytes, &options, kind, message, false).unwrap();
            assert_eq!(String::from_utf8(bytes).unwrap(), expected);
        }
    }

    #[test]
    fn disabled_color_strips_existing_styles() {
        let options = Options {
            color: Some(false),
            ..Options::default()
        };
        let mut bytes = Vec::new();
        render(
            &mut bytes,
            &options,
            "warning",
            "\x1b[32mnotice\x1b[0m",
            true,
        )
        .unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "warning: notice\n");
    }

    #[test]
    fn human_colors_compiler_diagnostics_by_severity() {
        let options = Options {
            color: Some(true),
            ..Options::default()
        };
        let mut bytes = Vec::new();
        render(
            &mut bytes,
            &options,
            "compiler_output",
            "file.c:1:1: warning: unused value\nfile.c:2:1: error: invalid token",
            true,
        )
        .unwrap();
        let output = String::from_utf8(bytes).unwrap();
        assert!(output.contains("\x1b[33mfile.c:1:1: warning: unused value\x1b[0m"));
        assert!(output.contains("\x1b[31mfile.c:2:1: error: invalid token\x1b[0m"));
    }

    #[test]
    fn plain_never_contains_ansi() {
        let mut bytes = Vec::new();
        let options = Options {
            mode: Mode::Plain,
            color: Some(true),
            ..Options::default()
        };
        render(&mut bytes, &options, "error", "compiler missing", true).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "error: compiler missing\n"
        );
    }

    #[test]
    fn json_is_versioned_and_escapes_multiline_messages() {
        let mut bytes = Vec::new();
        let options = Options {
            mode: Mode::Json,
            ..Options::default()
        };
        render(&mut bytes, &options, "warning", "first\nsecond", false).unwrap();
        let event: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(event["schema_version"], 1);
        assert_eq!(event["message"], "first\nsecond");
    }
}

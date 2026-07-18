use std::{error::Error as StdError, fmt, result::Result as StdResult, sync::LazyLock};

use color_eyre::Report;
use regex::Regex;
use thiserror::Error;

use crate::scripting::typescript::TranspileError;

pub type Result<T> = StdResult<T, ScriptError>;

pub type UnhandledException = (String, Vec<CallStackFrame>);

static CALLSTACK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*at(?: (?P<func>.+?) \()?(?P<file>.+?):(?P<line>\d+):(?P<col>\d+)\)?$")
        .expect("Failed to compile regex")
});

#[derive(Debug, Error)]
pub enum ScriptError {
    /// A structured JavaScript runtime error carrying a source-mapped call stack.
    #[error(transparent)]
    Runtime(#[from] RuntimeScriptError),

    /// A TypeScript-to-JavaScript compilation failure.
    #[error(transparent)]
    Compile(#[from] TranspileError),

    // Covers both host/engine-level failures (runtime/context creation) and caught
    // engine errors (`CaughtError::Error`) that lack a JavaScript stack.
    #[error("script error: {0}")]
    QuickJs(String),

    #[error("script value: {0}")]
    Value(String),

    #[error(transparent)]
    Report(#[from] Report),
}

impl ScriptError {
    pub(super) fn quickjs(error: &impl ToString) -> Self {
        Self::QuickJs(error.to_string())
    }

    #[must_use]
    pub const fn runtime_error(&self) -> Option<&RuntimeScriptError> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Compile(_) | Self::QuickJs(_) | Self::Value(_) | Self::Report(_) => None,
        }
    }

    #[must_use]
    pub fn primary_frame(&self) -> Option<&CallStackFrame> {
        self.runtime_error()?.primary_frame()
    }

    /// Whether this error represents a cancellation (e.g. Ctrl-C), which callers
    /// typically suppress rather than display.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.runtime_error()
            .is_some_and(RuntimeScriptError::is_cancelled)
    }

    #[must_use]
    pub fn line(&self) -> Option<u32> {
        match self {
            Self::Runtime(error) => error.primary_frame().map(|frame| frame.line()),
            Self::Compile(_) | Self::QuickJs(_) | Self::Value(_) | Self::Report(_) => None,
        }
    }

    #[must_use]
    pub fn column(&self) -> Option<u32> {
        match self {
            Self::Runtime(error) => error.primary_frame().map(|frame| frame.column()),
            Self::Compile(_) | Self::QuickJs(_) | Self::Value(_) | Self::Report(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct CallStackFrame {
    function: String,
    file: String,
    line: u32,
    col: u32,
}

impl CallStackFrame {
    pub(super) fn new(
        function: impl Into<String>,
        file: impl Into<String>,
        line: u32,
        col: u32,
    ) -> Self {
        Self {
            function: function.into(),
            file: file.into(),
            line,
            col,
        }
    }

    /// Function name reported by the JavaScript stack frame, if available.
    #[must_use]
    pub fn function(&self) -> &str {
        &self.function
    }

    /// Filename reported by the JavaScript stack frame.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    pub(super) fn set_file(&mut self, file: impl Into<String>) {
        self.file = file.into();
    }

    /// One-based source line.
    #[must_use]
    pub const fn line(&self) -> u32 {
        self.line
    }

    pub(super) const fn set_line(&mut self, line: u32) {
        self.line = line;
    }

    /// One-based source column.
    #[must_use]
    pub const fn column(&self) -> u32 {
        self.col
    }

    pub(super) const fn set_column(&mut self, col: u32) {
        self.col = col;
    }
}

impl fmt::Display for CallStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.function.is_empty() {
            write!(f, "    at {}:{}:{}", self.file, self.line, self.col)
        } else {
            write!(
                f,
                "    at {} ({}:{}:{})",
                self.function, self.file, self.line, self.col
            )
        }
    }
}

#[derive(Debug)]
pub struct RuntimeScriptError {
    message: String,
    stack: Vec<CallStackFrame>,
    cancelled: bool,
}

impl RuntimeScriptError {
    pub(super) const fn new(message: String, stack: Vec<CallStackFrame>, cancelled: bool) -> Self {
        Self {
            message,
            stack,
            cancelled,
        }
    }

    /// Error message reported by the JavaScript runtime.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Whether this runtime error represents a cancellation rather than a genuine failure.
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Parsed and source-mapped JavaScript call stack.
    #[must_use]
    pub fn stack(&self) -> &[CallStackFrame] {
        &self.stack
    }

    /// The top stack frame, which is usually the primary error location.
    #[must_use]
    pub fn primary_frame(&self) -> Option<&CallStackFrame> {
        self.stack.first()
    }
}

impl fmt::Display for RuntimeScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        for frame in &self.stack {
            write!(f, "\n{frame}")?;
        }
        Ok(())
    }
}

impl StdError for RuntimeScriptError {}

/// Parses a JavaScript stack trace into frames, keeping every line that parses and
/// silently skipping any that do not (e.g. anonymous or native frames), rather than
/// discarding the whole stack when a single line is malformed.
pub(super) fn parse_callstack(stack: &str) -> Vec<CallStackFrame> {
    stack
        .lines()
        .filter_map(|line| parse_callstack_line(line.trim()))
        .collect()
}

pub(super) fn parse_callstack_line(line: &str) -> Option<CallStackFrame> {
    let caps = CALLSTACK_REGEX.captures(line)?;
    let function = caps.name("func").map_or("", |cap| cap.as_str());
    let file = caps.name("file").map_or("", |cap| cap.as_str());
    let line = caps.name("line")?.as_str().parse::<u32>().ok()?;
    let col = caps.name("col")?.as_str().parse::<u32>().ok()?;

    Some(CallStackFrame::new(function, file, line, col))
}

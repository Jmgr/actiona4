use swc_common::{
    BytePos, FileName, FilePathMapping, SourceMap, Span,
    errors::{ColorConfig, Handler},
    sync::Lrc,
};

use crate::scripting::{RuntimeScriptError, ScriptError};

const MAX_STACK_NOTES: usize = 8;

/// Attempts to emit a rich SWC-style diagnostic for a script error.
///
/// Returns `true` if the error was handled and emitted, `false` if the caller
/// should fall back to its own error display.
#[must_use]
pub fn try_emit_script_diagnostic(err: &ScriptError, source_code: &str) -> bool {
    if let ScriptError::Compile(error) = err {
        // A compile error either already printed its rich SWC diagnostic (nothing more to do)
        // or has none, in which case the caller should fall back to its own display.
        return error.diagnostics_already_emitted();
    }

    let Some(runtime_error) = err.runtime_error() else {
        return false;
    };

    if runtime_error.is_cancelled() {
        return true;
    }

    let (cm, handler) = new_tty_handler();

    let primary_span = runtime_primary_span(runtime_error, source_code, &cm);
    let mut diagnostic = primary_span.map_or_else(
        || handler.struct_err(runtime_error.message()),
        |span| handler.struct_span_err(span, runtime_error.message()),
    );

    #[allow(clippy::bool_to_int_with_if)]
    let first_note_index = if primary_span.is_some() { 1 } else { 0 };
    for frame in runtime_error
        .stack()
        .iter()
        .skip(first_note_index)
        .take(MAX_STACK_NOTES)
    {
        diagnostic.note(&frame.to_string());
    }
    if runtime_error.stack().len() > (first_note_index + MAX_STACK_NOTES) {
        diagnostic.note("...");
    }

    diagnostic.emit();
    true
}

pub(in crate::scripting) fn new_tty_handler() -> (Lrc<SourceMap>, Handler) {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    (cm, handler)
}

pub(in crate::scripting) fn runtime_primary_span(
    runtime_error: &RuntimeScriptError,
    source_code: &str,
    cm: &Lrc<SourceMap>,
) -> Option<Span> {
    let frame = runtime_error.stack().first()?;
    let line_index = usize::try_from(frame.line().checked_sub(1)?).ok()?;
    let column_index = usize::try_from(frame.column().checked_sub(1)?).ok()?;

    let source_file = cm.new_source_file(
        Lrc::new(FileName::Custom(frame.file().to_string())),
        source_code.to_string(),
    );

    let line_start = *source_file.analyze().lines.get(line_index)?;
    let line = source_file.get_line(line_index)?;

    if let Some((start_byte, end_byte)) =
        reference_error_identifier_range(runtime_error.message(), &line, frame.column())
    {
        let lo = line_start + BytePos(u32::try_from(start_byte).ok()?);
        let hi = line_start + BytePos(u32::try_from(end_byte).ok()?);
        return Some(Span::new(lo, hi));
    }

    if let Some((start_byte, end_byte)) =
        not_a_function_identifier_range(runtime_error.message(), &line, frame.column())
    {
        let lo = line_start + BytePos(u32::try_from(start_byte).ok()?);
        let hi = line_start + BytePos(u32::try_from(end_byte).ok()?);
        return Some(Span::new(lo, hi));
    }

    let column_byte = line
        .char_indices()
        .nth(column_index)
        .map_or_else(|| line.len(), |(index, _)| index);
    let column_byte = u32::try_from(column_byte).ok()?;
    let lo = line_start + BytePos(column_byte);

    Some(Span::new(lo, lo))
}

fn reference_error_identifier_range(
    message: &str,
    line: &str,
    reported_col: u32,
) -> Option<(usize, usize)> {
    let identifier = parse_reference_error_identifier(message)?;
    let reported_col = usize::try_from(reported_col.checked_sub(1)?).ok()?;
    find_closest_identifier_range(line, identifier, reported_col)
}

pub(in crate::scripting) fn parse_reference_error_identifier(message: &str) -> Option<&str> {
    let identifier = message
        .strip_prefix("ReferenceError: ")?
        .strip_suffix(" is not defined")?
        .trim();
    let identifier = strip_matching_quotes(identifier);
    is_valid_js_identifier(identifier).then_some(identifier)
}

fn not_a_function_identifier_range(
    message: &str,
    line: &str,
    reported_col: u32,
) -> Option<(usize, usize)> {
    let reported_col = usize::try_from(reported_col.checked_sub(1)?).ok()?;

    if let Some(identifier) = parse_type_error_identifier(message) {
        return find_closest_identifier_range(line, identifier, reported_col);
    }

    is_plain_not_a_function_type_error(message)
        .then(|| find_closest_call_identifier_range(line, reported_col))
        .flatten()
}

pub(in crate::scripting) fn parse_type_error_identifier(message: &str) -> Option<&str> {
    let identifier = message
        .strip_prefix("TypeError: ")?
        .strip_suffix(" is not a function")?
        .trim();
    let identifier = strip_matching_quotes(identifier);
    is_valid_js_identifier(identifier).then_some(identifier)
}

fn is_plain_not_a_function_type_error(message: &str) -> bool {
    message.trim() == "TypeError: not a function"
}

fn strip_matching_quotes(value: &str) -> &str {
    match value.as_bytes() {
        [b'\'' | b'"', .., last] if *last == value.as_bytes()[0] => &value[1..value.len() - 1],
        _ => value,
    }
}

pub(in crate::scripting) fn find_closest_identifier_range(
    line: &str,
    identifier: &str,
    reported_col: usize,
) -> Option<(usize, usize)> {
    let ident_char_len = identifier.chars().count();
    let mut best_match: Option<(usize, usize, usize)> = None;
    for (start_byte, _) in line.match_indices(identifier) {
        let end_byte = start_byte + identifier.len();
        if !is_identifier_boundary(line, start_byte, end_byte) {
            continue;
        }

        let start_col = line[..start_byte].chars().count();
        let end_col_exclusive = start_col + ident_char_len;
        let distance = column_distance_to_identifier(reported_col, start_col, end_col_exclusive);
        let candidate = (distance, start_byte, end_byte);

        if best_match.is_none_or(|current| candidate < current) {
            best_match = Some(candidate);
        }
    }

    best_match.map(|(_, start_byte, end_byte)| (start_byte, end_byte))
}

pub(in crate::scripting) fn find_closest_call_identifier_range(
    line: &str,
    reported_col: usize,
) -> Option<(usize, usize)> {
    let mut best_non_constructor: Option<(usize, usize, usize)> = None;
    let mut best_match: Option<(usize, usize, usize)> = None;
    for (start_byte, ch) in line.char_indices() {
        if !is_js_identifier_start(ch) {
            continue;
        }

        if line[..start_byte]
            .chars()
            .next_back()
            .is_some_and(is_js_identifier_continue)
        {
            continue;
        }

        let mut ident_end_byte = start_byte + ch.len_utf8();
        for next in line[ident_end_byte..].chars() {
            if is_js_identifier_continue(next) {
                ident_end_byte += next.len_utf8();
            } else {
                break;
            }
        }

        let after_ident_byte = ident_end_byte
            + line[ident_end_byte..]
                .chars()
                .take_while(|c| c.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>();

        let Some(next) = line[after_ident_byte..].chars().next() else {
            continue;
        };
        if next != '(' {
            continue;
        }

        let start_col = line[..start_byte].chars().count();
        let end_col_exclusive = start_col + line[start_byte..ident_end_byte].chars().count();
        let distance = column_distance_to_identifier(reported_col, start_col, end_col_exclusive);
        let candidate = (distance, start_byte, ident_end_byte);
        let is_constructor = is_constructor_call(line, start_byte);

        if best_match.is_none_or(|current| candidate < current) {
            best_match = Some(candidate);
        }
        if !is_constructor && best_non_constructor.is_none_or(|current| candidate < current) {
            best_non_constructor = Some(candidate);
        }
    }

    best_non_constructor
        .or(best_match)
        .map(|(_, start_byte, end_byte)| (start_byte, end_byte))
}

fn is_constructor_call(line: &str, start_byte: usize) -> bool {
    let prefix = line[..start_byte].trim_end_matches(char::is_whitespace);
    let Some(before_new) = prefix.strip_suffix("new") else {
        return false;
    };
    before_new
        .chars()
        .next_back()
        .is_none_or(|ch| !is_js_identifier_continue(ch))
}

const fn column_distance_to_identifier(
    reported_col: usize,
    start_col: usize,
    end_col_exclusive: usize,
) -> usize {
    if reported_col < start_col {
        start_col - reported_col
    } else if reported_col >= end_col_exclusive {
        reported_col - (end_col_exclusive.saturating_sub(1))
    } else {
        0
    }
}

fn is_identifier_boundary(line: &str, start_byte: usize, end_byte: usize) -> bool {
    let prev = line[..start_byte].chars().next_back();
    let next = line[end_byte..].chars().next();
    prev.is_none_or(|ch| !is_js_identifier_continue(ch))
        && next.is_none_or(|ch| !is_js_identifier_continue(ch))
}

fn is_valid_js_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    is_js_identifier_start(first) && chars.all(is_js_identifier_continue)
}

#[must_use]
pub fn is_js_identifier_start(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphabetic()
}

#[must_use]
pub fn is_js_identifier_continue(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphanumeric()
}

/// Parsed pragma directives from a script's header comments.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pragmas {
    /// When true, all Actiona API objects are placed under an `actiona` namespace
    /// instead of the global scope (e.g. `actiona.mouse` instead of `mouse`).
    pub no_globals: bool,
}

/// Parses `//@actiona` pragma directives from the beginning of a script.
///
/// Scans lines from the start of the script. Only blank lines and single-line
/// comments (`//`) are considered part of the pragma header. The scan stops at
/// the first line that is neither blank nor a `//` comment.
#[must_use]
pub fn parse_pragmas(script: &str) -> Pragmas {
    let mut pragmas = Pragmas::default();

    for line in script.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if !trimmed.starts_with("//") {
            break;
        }

        // Strip the "//" prefix and check for pragma
        let comment = trimmed.strip_prefix("//").unwrap();

        // Support both `//@actiona noglobals` and `// @actiona noglobals`
        let comment = comment.trim_start();

        if comment == "@actiona noglobals" {
            pragmas.no_globals = true;
        }
    }

    pragmas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_pragmas() {
        let pragmas = parse_pragmas("const x = 1;");
        assert_eq!(pragmas, Pragmas::default());
        assert!(!pragmas.no_globals);
    }

    #[test]
    fn no_globals_pragma() {
        let pragmas = parse_pragmas("//@actiona noglobals\nconst x = 1;");
        assert!(pragmas.no_globals);
    }

    #[test]
    fn no_globals_pragma_with_spaces() {
        let pragmas = parse_pragmas("// @actiona noglobals\nconst x = 1;");
        assert!(pragmas.no_globals);
    }

    #[test]
    fn no_globals_pragma_with_leading_whitespace() {
        let pragmas = parse_pragmas("  //@actiona noglobals\nconst x = 1;");
        assert!(pragmas.no_globals);
    }

    #[test]
    fn no_globals_pragma_after_blank_lines() {
        let pragmas = parse_pragmas("\n\n//@actiona noglobals\nconst x = 1;");
        assert!(pragmas.no_globals);
    }

    #[test]
    fn no_globals_pragma_after_other_comments() {
        let pragmas = parse_pragmas("// some comment\n//@actiona noglobals\nconst x = 1;");
        assert!(pragmas.no_globals);
    }

    #[test]
    fn pragma_after_code_is_ignored() {
        let pragmas = parse_pragmas("const x = 1;\n//@actiona noglobals");
        assert!(!pragmas.no_globals);
    }

    #[test]
    fn wrong_pragma_name_is_ignored() {
        let pragmas = parse_pragmas("//@actiona something_else\nconst x = 1;");
        assert!(!pragmas.no_globals);
    }

    #[test]
    fn empty_script() {
        let pragmas = parse_pragmas("");
        assert!(!pragmas.no_globals);
    }
}

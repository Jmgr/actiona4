use std::path::{self, Path, PathBuf};

use macros::{js_class, js_methods};
use rquickjs::{Ctx, JsLifetime, atom::PredefinedAtom, class::Trace, prelude::Rest};

use crate::api::js::classes::HostClass;

/// Utilities for manipulating file paths. All methods are static.
///
/// ```ts
/// const full = Path.join("/home/user", "documents", "file.txt");
/// const dir = Path.parent(full);   // "/home/user/documents"
/// const name = Path.filename(full); // "file.txt"
/// const ext = Path.extension(full); // "txt"
/// ```
///
/// ```ts
/// // Change a file's extension
/// const newPath = Path.setExtension("/tmp/data.csv", "json");
/// // "/tmp/data.json"
/// ```
#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[js_class]
pub struct JsPath {}

impl HostClass<'_> for JsPath {}

#[js_methods]
impl JsPath {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> rquickjs::Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Path cannot be instantiated directly",
        ))
    }

    /// Joins path segments into a single path.
    ///
    /// ```ts
    /// Path.join("/home", "user", "file.txt"); // "/home/user/file.txt"
    /// ```
    /// @rest string
    #[qjs(static)]
    #[must_use]
    pub fn join(args: Rest<String>) -> String {
        let mut path = PathBuf::new();
        for part in args.iter() {
            path.push(part);
        }
        path.to_string_lossy().into_owned()
    }

    /// Returns the file name component of a path.
    ///
    /// ```ts
    /// Path.filename("/home/user/file.txt"); // "file.txt"
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn filename(path: String) -> String {
        Path::new(&path)
            .file_name()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Alias for `filename`.
    #[qjs(static)]
    #[must_use]
    pub fn basename(path: String) -> String {
        Self::filename(path)
    }

    /// Returns the parent directory of a path.
    ///
    /// ```ts
    /// Path.parent("/home/user/file.txt"); // "/home/user"
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn parent(path: String) -> String {
        Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Alias for `parent`.
    #[qjs(static)]
    #[must_use]
    pub fn dirname(path: String) -> String {
        Self::parent(path)
    }

    /// Returns whether the path is absolute.
    ///
    /// ```ts
    /// Path.isAbsolute("/home/user"); // true
    /// Path.isAbsolute("relative/path"); // false
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn is_absolute(path: String) -> bool {
        Path::new(&path).is_absolute()
    }

    /// Returns whether the path is relative.
    ///
    /// ```ts
    /// Path.isRelative("relative/path"); // true
    /// Path.isRelative("/absolute/path"); // false
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn is_relative(path: String) -> bool {
        Path::new(&path).is_relative()
    }

    /// Returns the file extension of a path (without the leading dot).
    ///
    /// ```ts
    /// Path.extension("/home/user/file.txt"); // "txt"
    /// Path.extension("/home/user/file"); // ""
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn extension(path: String) -> String {
        Path::new(&path)
            .extension()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Alias for `extension`.
    #[qjs(static)]
    #[must_use]
    pub fn extname(path: String) -> String {
        Self::extension(path)
    }

    /// Returns the path with a different extension. Returns an empty string on failure.
    ///
    /// ```ts
    /// Path.setExtension("/tmp/data.csv", "json"); // "/tmp/data.json"
    /// Path.setExtension("/tmp/archive.tar.gz", "xz"); // "/tmp/archive.tar.xz"
    /// ```
    #[qjs(static)]
    pub fn set_extension(path: String, extension: String) -> String {
        // Avoid a panic if `extension` contains a separator
        if extension.chars().into_iter().any(path::is_separator) {
            return String::new();
        }

        let mut path = PathBuf::from(path);

        if !path.set_extension(extension) {
            return String::new();
        }

        path.to_string_lossy().into_owned()
    }

    /// Returns a string representation of this path.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Path".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::runtime::Runtime;

    #[test]
    fn test_join() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(r#"Path.join("foo", "bar")"#)
                .await
                .unwrap();
            assert_eq!(result, Path::new("foo").join("bar").to_string_lossy());
        });
    }

    #[test]
    fn test_filename() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(r#"Path.filename("/foo/bar/test.txt")"#)
                .await
                .unwrap();
            assert_eq!(result, "test.txt");

            let result = script_engine
                .eval_async::<String>(r#"Path.filename("/foo/bar")"#)
                .await
                .unwrap();
            assert_eq!(result, "bar");
        });
    }

    #[test]
    fn test_parent() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(r#"Path.parent("/foo/bar/test.txt")"#)
                .await
                .unwrap();
            assert_eq!(result, "/foo/bar");

            let result = script_engine
                .eval_async::<String>(r#"Path.parent("/foo/bar")"#)
                .await
                .unwrap();
            assert_eq!(result, "/foo");
        });
    }

    #[test]
    fn test_absolute() {
        Runtime::test_with_script_engine(|script_engine| async move {
            #[cfg(windows)]
            let absolute_path = "C:/foo/bar/test.txt";
            #[cfg(not(windows))]
            let absolute_path = "/foo/bar/test.txt";

            let result = script_engine
                .eval_async::<bool>(&format!(r#"Path.isAbsolute("{absolute_path}")"#))
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval_async::<bool>(r#"Path.isAbsolute("foo/bar")"#)
                .await
                .unwrap();
            assert!(!result);
        });
    }

    #[test]
    fn test_relative() {
        Runtime::test_with_script_engine(|script_engine| async move {
            #[cfg(windows)]
            let absolute_path = "C:/foo/bar/test.txt";
            #[cfg(not(windows))]
            let absolute_path = "/foo/bar/test.txt";

            let result = script_engine
                .eval_async::<bool>(&format!(r#"Path.isRelative("{absolute_path}")"#))
                .await
                .unwrap();
            assert!(!result);

            let result = script_engine
                .eval_async::<bool>(r#"Path.isRelative("foo/bar")"#)
                .await
                .unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_extension() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(r#"Path.extension("/foo/bar/test.txt")"#)
                .await
                .unwrap();
            assert_eq!(result, "txt");

            let result = script_engine
                .eval_async::<String>(r#"Path.extension("foo/bar")"#)
                .await
                .unwrap();
            assert_eq!(result, "");
        });
    }

    #[test]
    fn test_set_extension() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(r#"Path.setExtension("/foo/bar/test.txt", "foo")"#)
                .await
                .unwrap();
            assert_eq!(result, "/foo/bar/test.foo");

            let result = script_engine
                .eval_async::<String>(r#"Path.setExtension("/foo/bar/test", "foo")"#)
                .await
                .unwrap();
            assert_eq!(result, "/foo/bar/test.foo");

            let result = script_engine
                .eval_async::<String>(r#"Path.setExtension("/foo/bar/test", "not/valid")"#)
                .await
                .unwrap();
            assert_eq!(result, "");
        });
    }
}

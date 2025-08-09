use std::{
    fmt::Debug,
    path::{self, Path, PathBuf},
};

use rquickjs::{JsLifetime, Result, class::Trace, prelude::Rest};

use crate::core::ValueClass;

#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[rquickjs::class(rename = "Path")]
pub struct JsPath {}

impl ValueClass<'_> for JsPath {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsPath {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// @rest string
    #[qjs(static)]
    pub fn join(args: Rest<String>) -> String {
        let mut path = PathBuf::new();
        for part in args.iter() {
            path.push(part);
        }
        path.to_string_lossy().into_owned()
    }

    #[qjs(static)]
    pub fn filename(path: String) -> String {
        Path::new(&path)
            .file_name()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    #[qjs(static)]
    pub fn basename(path: String) -> String {
        Self::filename(path)
    }

    #[qjs(static)]
    pub fn parent(path: String) -> String {
        Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    #[qjs(static)]
    pub fn dirname(path: String) -> String {
        Self::parent(path)
    }

    #[qjs(static)]
    pub fn is_absolute(path: String) -> bool {
        Path::new(&path).is_absolute()
    }

    #[qjs(static)]
    pub fn is_relative(path: String) -> bool {
        Path::new(&path).is_relative()
    }

    #[qjs(static)]
    pub fn extension(path: String) -> String {
        Path::new(&path)
            .extension()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    #[qjs(static)]
    pub fn extname(path: String) -> String {
        Self::extension(path)
    }

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
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    #[test]
    fn test_join() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let result = script_engine
                .eval_async::<String>(r#"Path.join("foo", "bar")"#)
                .await
                .unwrap();
            assert_eq!(result, "foo/bar");
        });
    }

    #[test]
    fn test_filename() {
        Runtime::test_with_script_engine(async move |script_engine| {
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
        Runtime::test_with_script_engine(async move |script_engine| {
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
        Runtime::test_with_script_engine(async move |script_engine| {
            let result = script_engine
                .eval_async::<bool>(r#"Path.isAbsolute("/foo/bar/test.txt")"#)
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
        Runtime::test_with_script_engine(async move |script_engine| {
            let result = script_engine
                .eval_async::<bool>(r#"Path.isRelative("/foo/bar/test.txt")"#)
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
        Runtime::test_with_script_engine(async move |script_engine| {
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
        Runtime::test_with_script_engine(async move |script_engine| {
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

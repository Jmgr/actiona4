use rquickjs::{Ctx, JsLifetime, class::Trace};
use tokio::fs;

use crate::core::js::classes::HostClass;

/// Provides static methods for querying filesystem path types.
///
/// ```ts
/// if (await Filesystem.exists("/tmp/myfile.txt")) {
///     console.log("exists!");
/// }
///
/// if (await Filesystem.isFile("/tmp/myfile.txt")) {
///     console.log("it's a file");
/// } else if (await Filesystem.isDirectory("/tmp/myfile.txt")) {
///     console.log("it's a directory");
/// }
/// ```
#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[rquickjs::class(rename = "Filesystem")]
pub struct JsFilesystem {}

impl HostClass<'_> for JsFilesystem {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsFilesystem {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> rquickjs::Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Filesystem cannot be instantiated directly",
        ))
    }

    /// Returns `true` if a path exists on the filesystem.
    #[qjs(static)]
    pub async fn exists(path: String) -> bool {
        fs::try_exists(path).await.unwrap_or_default()
    }

    /// Returns `true` if the path points to a regular file.
    #[qjs(static)]
    pub async fn is_file(path: String) -> bool {
        fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_file())
    }

    /// Returns `true` if the path points to a directory.
    #[qjs(static)]
    pub async fn is_directory(path: String) -> bool {
        fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
    }

    /// Returns `true` if the path points to a symbolic link.
    #[qjs(static)]
    pub async fn is_symlink(path: String) -> bool {
        fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_symlink())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use tokio::fs;

    use crate::runtime::Runtime;

    #[test]
    fn test_exists() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let temp_dir = env::temp_dir();
            let path = temp_dir.to_string_lossy();

            let result = script_engine
                .eval_async::<bool>(&format!(r#"await Filesystem.exists("{path}")"#))
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval_async::<bool>(r#"await Filesystem.exists("/non/existent/path")"#)
                .await
                .unwrap();
            assert!(!result);
        });
    }

    #[test]
    fn test_is_file() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let file_path = env::temp_dir().join("test.txt");
            fs::write(&file_path, b"test").await.unwrap();
            let directory_path = env::temp_dir();

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"await Filesystem.isFile("{}")"#,
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"await Filesystem.isFile("{}")"#,
                    directory_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert!(!result);
        });
    }

    #[test]
    fn test_is_directory() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let file_path = env::temp_dir().join("test.txt");
            fs::write(&file_path, b"test").await.unwrap();
            let directory_path = env::temp_dir();

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"await Filesystem.isDirectory("{}")"#,
                    directory_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"await Filesystem.isDirectory("{}")"#,
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert!(!result);
        });
    }
}

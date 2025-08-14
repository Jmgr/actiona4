use std::fmt::Debug;

use rquickjs::{JsLifetime, Result, class::Trace};
use tokio::fs;

use crate::core::js::classes::ValueClass;

#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[rquickjs::class(rename = "Filesystem")]
pub struct JsFilesystem {}

impl ValueClass<'_> for JsFilesystem {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsFilesystem {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    #[qjs(static)]
    pub async fn exists(path: String) -> bool {
        fs::try_exists(path).await.unwrap_or_default()
    }

    #[qjs(static)]
    pub async fn is_file(path: String) -> bool {
        fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_file())
    }

    #[qjs(static)]
    pub async fn is_directory(path: String) -> bool {
        fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
    }

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
        Runtime::test_with_script_engine(async move |script_engine| {
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
        Runtime::test_with_script_engine(async move |script_engine| {
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
        Runtime::test_with_script_engine(async move |script_engine| {
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

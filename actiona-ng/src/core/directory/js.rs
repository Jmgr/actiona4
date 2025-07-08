use std::fmt::Debug;

use macros::FromJsObject;
use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::fs;

use crate::core::ValueClass;

/// Directory options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDirectoryOptions {
    /// Should the directories be created or removed recursively?
    /// @default true
    pub recursive: bool,
}

impl Default for JsDirectoryOptions {
    fn default() -> Self {
        Self { recursive: true }
    }
}

#[derive(Clone, Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "Directory")]
pub struct JsDirectory {}

impl ValueClass<'_> for JsDirectory {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDirectory {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    #[qjs(static)]
    pub async fn create(path: String, options: Opt<JsDirectoryOptions>) -> Result<()> {
        let options = options.unwrap_or_default();

        if options.recursive {
            fs::create_dir_all(&path).await?;
        } else {
            fs::create_dir(&path).await?;
        }

        Ok(())
    }

    #[qjs(static)]
    pub async fn remove(path: String, options: Opt<JsDirectoryOptions>) -> Result<()> {
        let options = options.unwrap_or_default();

        if options.recursive {
            fs::remove_dir_all(&path).await?;
        } else {
            fs::remove_dir(&path).await?;
        }

        Ok(())
    }
}

impl<'js> Trace<'js> for JsDirectory {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{core::random_name, runtime::Runtime};

    #[test]
    fn test_create_directory() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let file_path = env::temp_dir().join("test").join(random_name());

            // Try to remove a non existing directory
            let result = script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.remove("{}")
                "#,
                    file_path.display(),
                ))
                .await;
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("No such file or directory")
            );

            // Check that the directory doesn't exist
            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await Filesystem.exists("{}")
                "#,
                    file_path.display(),
                ))
                .await
                .unwrap();
            assert!(!result);

            // Create the directories
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.create("{}")
                "#,
                    file_path.display(),
                ))
                .await
                .unwrap();

            // Check that the directories exist
            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await Filesystem.exists("{}")
                "#,
                    file_path.display(),
                ))
                .await
                .unwrap();
            assert!(result);

            // Cleanup
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.remove("{}")
                "#,
                    file_path.display(),
                ))
                .await
                .unwrap();
        });
    }
}

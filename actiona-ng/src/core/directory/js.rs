use std::{fmt::Debug, path::PathBuf};

use macros::FromJsObject;
use rquickjs::{Ctx, JsLifetime, Result, class::Trace, prelude::Opt};
use tokio::fs::{self};

use crate::core::js::classes::{ValueClass, register_value_class};

/// Directory entry
#[derive(Clone, Debug, Default, Eq, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "DirectoryEntry")]
pub struct JsDirectoryEntry {
    path: String,
    file_name: String,
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    size: u64,
}

impl ValueClass<'_> for JsDirectoryEntry {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDirectoryEntry {
    /// @skip
    #[qjs(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        self.is_file
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        self.is_directory
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        self.is_symlink
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

/// Directory options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDirectoryOptions {
    /// Should the directories be created or removed recursively?
    /// @default `true`
    pub recursive: bool,
}

impl Default for JsDirectoryOptions {
    fn default() -> Self {
        Self { recursive: true }
    }
}

/// Directory list options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDirectoryListOptions {
    /// Should the entries be sorted?
    /// @default `true`
    pub sort: bool,

    /// Should each entry's absolute path be computed?
    /// @default `true`
    pub absolute_path: bool,

    /// Should each entry's size be fetched?
    /// @default `true`
    pub fetch_size: bool,
}

impl Default for JsDirectoryListOptions {
    fn default() -> Self {
        Self {
            sort: true,
            absolute_path: true,
            fetch_size: true,
        }
    }
}

#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[rquickjs::class(rename = "Directory")]
pub struct JsDirectory {}

impl ValueClass<'_> for JsDirectory {
    // TODO: should this be a HostValue?
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        register_value_class::<JsDirectoryEntry>(ctx)?;

        Ok(())
    }
}

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

    #[qjs(static)]
    pub async fn list_entries(
        path: String,
        options: Opt<JsDirectoryListOptions>,
    ) -> Result<Vec<JsDirectoryEntry>> {
        let options = options.unwrap_or_default();
        let dir_path = PathBuf::from(path);
        let mut read_dir = fs::read_dir(&dir_path).await?;
        let mut result = Vec::new();

        while let Some(entry) = read_dir.next_entry().await? {
            let (is_file, is_directory, is_symlink) =
                (entry.file_type().await).map_or((false, false, false), |file_type| {
                    (
                        file_type.is_file(),
                        file_type.is_dir(),
                        file_type.is_symlink(),
                    )
                });

            let path = if options.absolute_path {
                fs::canonicalize(entry.path()).await?
            } else {
                entry.path()
            }
            .to_string_lossy()
            .to_string();

            let size = if options.fetch_size {
                let metadata = fs::metadata(entry.path()).await?;
                metadata.len()
            } else {
                0
            };

            result.push(JsDirectoryEntry {
                path,
                file_name: entry.file_name().to_string_lossy().to_string(),
                is_file,
                is_directory,
                is_symlink,
                size,
            });
        }

        if options.sort {
            result.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use tokio::fs;

    use crate::{
        core::{directory::js::JsDirectoryEntry, test_helpers::random_name},
        runtime::Runtime,
    };

    #[test]
    fn test_create_directory() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let file_path = env::temp_dir().join("test").join(random_name());

            // Try to remove a non existing directory
            let result = script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.remove("{}")
                "#,
                    file_path.to_string_lossy(),
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
                    file_path.to_string_lossy(),
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
                    file_path.to_string_lossy(),
                ))
                .await
                .unwrap();

            // Check that the directories exist
            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await Filesystem.exists("{}")
                "#,
                    file_path.to_string_lossy(),
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
                    file_path.to_string_lossy(),
                ))
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_list() {
        Runtime::test_with_script_engine(|script_engine| async move {
            env::set_current_dir(env::temp_dir()).unwrap();
            let parent_path = Path::new(".").join(random_name());
            let directory_path = parent_path.join("a");
            fs::create_dir_all(&directory_path).await.unwrap();
            let file_path = parent_path.join("b");
            fs::write(&file_path, b"test").await.unwrap();

            // We need to manually compute the directory "size" because it can vary between OSes.
            let directory_size = fs::metadata(&directory_path).await.unwrap().len();

            let result = script_engine
                .eval_async::<Vec<JsDirectoryEntry>>(&format!(
                    r#"
                await Directory.listEntries("{}")
                "#,
                    parent_path.to_string_lossy(),
                ))
                .await
                .unwrap();

            assert_eq!(
                result,
                vec![
                    JsDirectoryEntry {
                        path: fs::canonicalize(directory_path)
                            .await
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        file_name: "a".to_string(),
                        is_file: false,
                        is_directory: true,
                        is_symlink: false,
                        size: directory_size,
                    },
                    JsDirectoryEntry {
                        path: fs::canonicalize(file_path)
                            .await
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        file_name: "b".to_string(),
                        is_file: true,
                        is_directory: false,
                        is_symlink: false,
                        size: 4,
                    }
                ]
            );
        });
    }
}

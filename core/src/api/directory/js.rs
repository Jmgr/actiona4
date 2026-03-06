use std::{fmt::Debug, path::PathBuf};

use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};
use tokio::fs::{self};

use crate::{
    api::js::classes::{HostClass, register_host_class},
    types::display::{DisplayFields, display_with_type},
};

/// An entry returned by `Directory.listEntries()`, representing a file, directory,
/// or symlink within a directory.
///
/// ```ts
/// const entries = await Directory.listEntries("/home/user");
/// for (const entry of entries) {
///     println(entry.fileName, entry.isFile, entry.size);
/// }
/// ```
#[derive(Clone, Debug, Default, Eq, JsLifetime, PartialEq, Trace)]
#[js_class]
pub struct JsDirectoryEntry {
    path: String,
    file_name: String,
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    size: u64,
}

impl HostClass<'_> for JsDirectoryEntry {}

#[js_methods]
impl JsDirectoryEntry {
    /// The full path to the entry.
    #[get]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The file name (last component of the path).
    #[get]
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Whether this entry is a regular file.
    #[get]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        self.is_file
    }

    /// Whether this entry is a directory.
    #[get]
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        self.is_directory
    }

    /// Whether this entry is a symbolic link.
    #[get]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        self.is_symlink
    }

    /// The size of the entry in bytes.
    #[get]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "DirectoryEntry",
            DisplayFields::default()
                .display("path", &self.path)
                .display("fileName", &self.file_name)
                .display("isFile", self.is_file)
                .display("isDirectory", self.is_directory)
                .display("isSymlink", self.is_symlink)
                .display("size", self.size)
                .finish_as_string(),
        )
    }
}

/// Options for `Directory.create()` and `Directory.remove()`.
///
/// ```ts
/// await Directory.create("/tmp/a/b/c", { recursive: true });
/// await Directory.remove("/tmp/a", { recursive: false });
/// ```
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDirectoryOptions {
    /// Should the directories be created or removed recursively?
    #[default(true)]
    pub recursive: bool,
}

/// Options for `Directory.listEntries()`.
///
/// ```ts
/// const entries = await Directory.listEntries("/tmp", {
///   sort: false,
///   absolutePath: false,
///   fetchSize: true,
/// });
/// ```
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDirectoryListOptions {
    /// Should the entries be sorted?
    #[default(true)]
    pub sort: bool,

    /// Should each entry's size be fetched?
    #[default(true)]
    pub fetch_size: bool,
}

/// Provides static methods for creating, removing, and listing directories.
///
/// ```ts
/// // Create a directory (recursively by default)
/// await Directory.create("/tmp/my/nested/dir");
///
/// // List entries in a directory
/// const entries = await Directory.listEntries("/tmp/my/nested/dir");
/// for (const entry of entries) {
///     println(entry.fileName, entry.isFile ? "file" : "dir");
/// }
///
/// // Remove a directory tree
/// await Directory.remove("/tmp/my");
/// ```
#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[js_class]
pub struct JsDirectory {}

impl HostClass<'_> for JsDirectory {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        register_host_class::<JsDirectoryEntry>(ctx)?;

        Ok(())
    }
}

#[js_methods]
impl JsDirectory {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Directory cannot be instantiated directly",
        ))
    }

    /// Creates a directory at the given path. By default, creates parent directories
    /// recursively.
    ///
    /// ```ts
    /// await Directory.create("/tmp/a/b/c");
    ///
    /// // Non-recursive: fails if parent doesn't exist
    /// await Directory.create("/tmp/a/b/c", { recursive: false });
    /// ```
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

    /// Removes a directory. By default, removes all contents recursively.
    ///
    /// ```ts
    /// await Directory.remove("/tmp/my/dir");
    ///
    /// // Non-recursive: fails if the directory is not empty
    /// await Directory.remove("/tmp/my/dir", { recursive: false });
    /// ```
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

    /// Lists all entries in a directory, returning an array of `DirectoryEntry`.
    ///
    /// ```ts
    /// // List with defaults (sorted, absolute paths, sizes fetched)
    /// const entries = await Directory.listEntries("/home/user/docs");
    ///
    /// // Skip size fetching for faster listing
    /// const entries = await Directory.listEntries("/home/user/docs", {
    ///     fetchSize: false,
    /// });
    /// ```
    /// @readonly
    #[qjs(static)]
    pub async fn list_entries(
        path: String,
        options: Opt<JsDirectoryListOptions>,
    ) -> Result<Vec<JsDirectoryEntry>> {
        let options = options.unwrap_or_default();
        let dir_path = fs::canonicalize(PathBuf::from(path)).await?;
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

            let path = entry.path().to_string_lossy().to_string();

            let size = if options.fetch_size {
                let metadata = fs::symlink_metadata(entry.path()).await?;
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

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Directory".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use tokio::fs;

    use crate::{
        api::{
            directory::js::JsDirectoryEntry,
            test_helpers::{js_path, random_name},
        },
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
                await Directory.remove({})
                "#,
                    js_path(&file_path),
                ))
                .await;
            let err = result.unwrap_err().to_string().to_lowercase();
            assert!(
                err.contains("no such file or directory")
                    || err.contains("cannot find the file")
                    || err.contains("cannot find the path"),
                "unexpected error: {err}"
            );

            // Check that the directory doesn't exist
            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await Filesystem.exists({})
                "#,
                    js_path(&file_path),
                ))
                .await
                .unwrap();
            assert!(!result);

            // Create the directories
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.create({})
                "#,
                    js_path(&file_path),
                ))
                .await
                .unwrap();

            // Check that the directories exist
            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await Filesystem.exists({})
                "#,
                    js_path(&file_path),
                ))
                .await
                .unwrap();
            assert!(result);

            // Cleanup
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await Directory.remove({})
                "#,
                    js_path(&file_path),
                ))
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_list() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let original_dir = env::current_dir().unwrap();
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
                await Directory.listEntries({})
                "#,
                    js_path(&parent_path),
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

            env::set_current_dir(original_dir).unwrap();
        });
    }
}

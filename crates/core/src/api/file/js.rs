#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::fs::FileTimesExt;
use std::{fmt::Debug, fs::FileTimes, io::SeekFrom, sync::Arc};

use color_eyre::eyre::eyre;
use macros::{FromJsObject, js_class, js_methods, options, platform};
use rquickjs::{
    Ctx, Exception, JsLifetime, Object, Result, TypedArray,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};
use tokio_util::task::TaskTracker;

use crate::{
    IntoJsResult,
    api::js::{
        classes::HostClass,
        date::{date_from_system_time, system_time_from_date},
    },
    error::CommonError,
    runtime::WithUserData,
    types::display::display_with_type,
};

#[derive(Clone, Debug, JsLifetime)]
struct OpenedFile {
    path: String,
    file: Arc<Mutex<fs::File>>,
}

impl PartialEq for OpenedFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

/// Options for `File.open()`.
///
/// ```ts
/// // Read-only (default)
/// const file = await File.open("data.txt");
///
/// // Create a new file for writing
/// const file = await File.open("out.txt", {
///     write: true,
///     createNew: true,
/// });
///
/// // Append to an existing file
/// const file = await File.open("log.txt", {
///     write: true,
///     append: true,
/// });
/// ```
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsOpenOptions {
    /// Should the file be opened with read access?
    #[default(true)]
    pub read: bool,

    /// Should the file be opened with write access?
    pub write: bool,

    /// Writing: open the file in append mode.
    /// Note that setting this to `true` implies setting `write` to `true`.
    pub append: bool,

    /// Writing: truncate (remove all contents of) the file.
    /// Note that this only works if `write` is `true`.
    pub truncate: bool,

    /// Writing: create a new file if it doesn't exist.
    /// Note that this only works if `write` or `append` are set to `true`.
    pub create: bool,

    /// Writing: always create a new file, even if one already exists.
    /// Note that this only works if `write` or `append` are set to `true`.
    /// Note that `create` and `truncate` are ignored if this is set to `true`.
    pub create_new: bool,
}

impl From<JsOpenOptions> for fs::OpenOptions {
    fn from(value: JsOpenOptions) -> Self {
        Self::new()
            .read(value.read)
            .write(value.write)
            .append(value.append)
            .truncate(value.truncate)
            .create(value.create)
            .create_new(value.create_new)
            .clone()
    }
}

/// A file handle for reading and writing. Also provides static utility methods
/// for common file operations without needing to open a handle.
///
/// ```ts
/// // Read a file in one shot (static)
/// const text = await File.readText("config.json");
///
/// // Write a file in one shot (static)
/// await File.writeText("output.txt", "Hello!");
///
/// // Open, read/write, then close
/// const file = await File.open("data.bin", { read: true, write: true, create: true });
/// await file.writeBytes(new Uint8Array([1, 2, 3]));
/// await file.rewind();
/// const bytes = await file.readBytes();
/// await file.close();
///
/// // File utilities
/// await File.copy("src.txt", "dst.txt");
/// await File.rename("old.txt", "new.txt");
/// const exists = await File.exists("file.txt");
/// await File.remove("file.txt");
/// ```
#[derive(Clone, Debug, Default, JsLifetime)]
#[js_class]
pub struct JsFile {
    inner: Option<OpenedFile>,
}

impl HostClass<'_> for JsFile {}

impl JsFile {
    fn opened_file(&self, ctx: &Ctx<'_>) -> Result<&OpenedFile> {
        let Some(opened_file) = &self.inner else {
            return Err(Exception::throw_message(ctx, "File is not open"));
        };
        Ok(opened_file)
    }

    fn opened_file_mut(&mut self, ctx: &Ctx<'_>) -> Result<&mut OpenedFile> {
        let Some(opened_file) = &mut self.inner else {
            return Err(Exception::throw_message(ctx, "File is not open"));
        };
        Ok(opened_file)
    }
}

#[js_methods]
impl JsFile {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(Exception::throw_message(
            &ctx,
            "File cannot be instantiated directly, use File.open() instead",
        ))
    }

    /// Opens a file.
    ///
    /// Example
    /// ```js
    /// // Open a file for reading
    /// let file = await File.open("my_file.txt", {
    ///     read: true,
    /// });
    ///
    /// // Create a new file for writing.
    /// let file = await File.open("my_file.txt", {
    ///     write: true,
    ///     createNew: true,
    /// });
    ///
    /// // Append to an existing file.
    /// let file = await File.open("my_file.txt", {
    ///     write: true,
    ///     append: true,
    /// });
    /// ```
    #[qjs(static)]
    pub async fn open(ctx: Ctx<'_>, path: String, options: Opt<JsOpenOptions>) -> Result<Self> {
        let options = options.unwrap_or_default();

        let file = fs::OpenOptions::from(options)
            .open(&path)
            .await
            .map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error opening the file: {err}"))
            })?;

        Ok(Self {
            inner: Some(OpenedFile {
                path,
                file: Arc::new(Mutex::new(file)),
            }),
        })
    }

    /// Returns true if the file is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    /// Closes this file handle.
    /// Please note that the actual file might not be closed until all other handles to it are also closed.
    /// This can happen if you cloned() this File.
    pub fn close(&mut self) {
        self.inner = None;
    }

    /// Writes bytes to this file handle.
    ///
    /// @rename writeBytes
    ///
    /// @param bytes: Uint8Array
    #[qjs(rename = "writeBytes")]
    pub async fn write_bytes_instance(
        &mut self,
        ctx: Ctx<'_>,
        bytes: TypedArray<'_, u8>,
    ) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let bytes = bytes
            .as_bytes()
            .ok_or(CommonError::DetachedArrayBuffer)
            .into_js_result(&ctx)?;

        opened_file
            .file
            .lock()
            .await
            .write(bytes)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;
        opened_file.file.lock().await.flush().await?;

        Ok(())
    }

    /// Writes bytes to a file at the given path (static).
    ///
    /// @static
    ///
    /// @param path: string
    /// @param bytes: Uint8Array
    #[qjs(static)]
    pub async fn write_bytes(ctx: Ctx<'_>, path: String, bytes: TypedArray<'_, u8>) -> Result<()> {
        let bytes = bytes
            .as_bytes()
            .ok_or(CommonError::DetachedArrayBuffer)
            .into_js_result(&ctx)?;

        fs::write(path, bytes)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;

        Ok(())
    }

    /// Writes text to this file handle.
    ///
    /// @rename writeText
    ///
    /// @param text: string
    #[qjs(rename = "writeText")]
    pub async fn write_text_instance(&mut self, ctx: Ctx<'_>, text: String) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file
            .file
            .lock()
            .await
            .write(text.as_bytes())
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;
        opened_file.file.lock().await.flush().await?;

        Ok(())
    }

    /// Writes text to a file at the given path (static).
    ///
    /// ```ts
    /// await File.writeText("hello.txt", "Hello, world!");
    /// ```
    ///
    /// @static
    ///
    /// @param path: string
    /// @param text: string
    #[qjs(static)]
    pub async fn write_text(ctx: Ctx<'_>, path: String, text: String) -> Result<()> {
        fs::write(path, text)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;

        Ok(())
    }

    /// Reads bytes from this file handle. If `amount` is given, reads exactly that many bytes;
    /// otherwise reads until EOF.
    ///
    /// @rename readBytes
    ///
    /// @param amount?: number
    ///
    /// @returns Uint8Array
    #[qjs(rename = "readBytes")]
    pub async fn read_bytes_instance<'js>(
        &mut self,
        ctx: Ctx<'js>,
        amount: Opt<u64>,
    ) -> Result<TypedArray<'js, u8>> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let mut result = Vec::new();

        if let Some(amount) = amount.0 {
            result.resize(
                usize::try_from(amount)
                    .map_err(|err| eyre!("{err}"))
                    .into_js_result(&ctx)?,
                0,
            );
            opened_file
                .file
                .lock()
                .await
                .read_exact(&mut result)
                .await?;
        } else {
            // Note that we can't just call file.metadata here as that would cause a stack overflow
            let len = fs::metadata(&opened_file.path).await?.len();
            result.reserve(
                usize::try_from(len)
                    .map_err(|err| eyre!("{err}"))
                    .into_js_result(&ctx)?,
            );
            opened_file
                .file
                .lock()
                .await
                .read_to_end(&mut result)
                .await?;
        }

        TypedArray::new(ctx, result)
    }

    /// Reads bytes from a file at the given path (static).
    #[qjs(static)]
    pub async fn read_bytes(
        ctx: Ctx<'_>,
        path: String,
        amount: Opt<u64>,
    ) -> Result<TypedArray<'_, u8>> {
        let result = if let Some(amount) = amount.0 {
            let mut result = vec![
                0;
                usize::try_from(amount)
                    .map_err(|err| eyre!("{err}"))
                    .into_js_result(&ctx)?
            ];
            let mut file = fs::File::open(path).await?;
            file.read_exact(&mut result).await?;
            result
        } else {
            fs::read(path).await.map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error reading file: {err}"))
            })?
        };

        TypedArray::new(ctx, result)
    }

    /// Reads the entire file as a UTF-8 string from this file handle.
    ///
    /// @rename readText
    #[qjs(rename = "readText")]
    pub async fn read_text_instance(&mut self, ctx: Ctx<'_>) -> Result<String> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let mut result = String::new();

        opened_file
            .file
            .lock()
            .await
            .read_to_string(&mut result)
            .await
            .map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error reading from the file: {err}"))
            })?;

        Ok(result)
    }

    /// Reads the entire file as a UTF-8 string (static).
    ///
    /// ```ts
    /// const text = await File.readText("config.json");
    /// ```
    #[qjs(static)]
    pub async fn read_text(ctx: Ctx<'_>, path: String) -> Result<String> {
        fs::read_to_string(path)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error reading file: {err}")))
    }

    /// Returns the file size in bytes.
    #[qjs(rename = "size")]
    pub async fn size(&mut self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.lock().await.metadata().await?.len())
    }

    /// Truncates or extends the file to the given size in bytes.
    pub async fn set_size(&self, ctx: Ctx<'_>, size: u64) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.lock().await.set_len(size).await?)
    }

    /// Returns whether the file is read-only.
    pub async fn readonly(&self, ctx: Ctx<'_>) -> Result<bool> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file
            .file
            .lock()
            .await
            .metadata()
            .await?
            .permissions()
            .readonly())
    }

    /// Sets whether the file is read-only.
    pub async fn set_readonly(&self, ctx: Ctx<'_>, readonly: bool) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        let file = opened_file.file.lock().await;

        let mut permissions = file.metadata().await?.permissions();

        permissions.set_readonly(readonly);

        file.set_permissions(permissions).await?;

        Ok(())
    }

    /// Returns the Unix file mode (e.g. `0o644`).
    #[platform(not = "windows")]
    pub async fn mode(&self, ctx: Ctx<'_>) -> Result<u32> {
        ctx.user_data().require_not_windows(&ctx)?;
        #[cfg(unix)]
        {
            let opened_file = self.opened_file(&ctx)?;

            return Ok(opened_file
                .file
                .lock()
                .await
                .metadata()
                .await?
                .permissions()
                .mode());
        }

        #[cfg(windows)]
        {
            _ = ctx;
            Ok(0)
        }
    }

    /// Sets the file mode.
    /// You should use the octal notation to specify the mode: `await file.setMode(0o445)`.
    #[platform(not = "windows")]
    pub async fn set_mode(&self, ctx: Ctx<'_>, mode: u32) -> Result<()> {
        ctx.user_data().require_not_windows(&ctx)?;
        #[cfg(unix)]
        return {
            let opened_file = self.opened_file(&ctx)?;
            let file = opened_file.file.lock().await;

            let mut permissions = file.metadata().await?.permissions();

            permissions.set_mode(mode);

            file.set_permissions(permissions).await?;

            Ok(())
        };

        #[cfg(windows)]
        {
            _ = ctx;
            _ = mode;
            Ok(())
        }
    }

    /// Returns the last modification time of the file.
    /// @returns Date
    pub async fn modified_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.modified()?;
        date_from_system_time(&ctx, &modified)
    }

    #[qjs(skip)]
    async fn set_times(
        ctx: &Ctx<'_>,
        opened_file: &OpenedFile,
        times: FileTimes,
        task_tracker: TaskTracker,
    ) -> Result<()> {
        let path = opened_file.path.clone();

        task_tracker
            .spawn_blocking(move || {
                let file = std::fs::File::options().write(true).open(path)?;
                file.set_times(times)?; // No implemented in tokio::fs
                Result::<_>::Ok(())
            })
            .await
            .map_err(|err| Exception::throw_message(ctx, &format!("Task join error: {err}")))?
    }

    /// Sets the last modification time of the file.
    /// @param date: Date
    pub async fn set_modified_time<'js>(&self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;
        let system_time = system_time_from_date(ctx.clone(), date)?;

        Self::set_times(
            &ctx,
            opened_file,
            FileTimes::new().set_modified(system_time),
            ctx.user_data().task_tracker(),
        )
        .await?;

        Ok(())
    }

    /// Returns the last access time of the file.
    /// @returns Date
    pub async fn accessed_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.accessed()?;
        date_from_system_time(&ctx, &modified)
    }

    /// Sets the last access time of the file.
    /// @param date: Date
    pub async fn set_accessed_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let system_time = system_time_from_date(ctx.clone(), date)?;

        Self::set_times(
            &ctx,
            opened_file,
            FileTimes::new().set_accessed(system_time),
            ctx.user_data().task_tracker(),
        )
        .await?;

        Ok(())
    }

    /// Returns the creation time of the file.
    /// @returns Date
    pub async fn creation_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.created()?;
        date_from_system_time(&ctx, &modified)
    }

    /// Sets the creation time of the file.
    /// @param date: Date
    #[platform(not = "linux")]
    pub async fn set_creation_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        ctx.user_data().require_not_linux(&ctx)?;
        #[cfg(unix)]
        {
            _ = ctx;
            _ = date;
            Ok(())
        }

        #[cfg(windows)]
        {
            let task_tracker = ctx.user_data().task_tracker();
            let system_time = system_time_from_date(ctx.clone(), date)?;
            let opened_file = self.opened_file(&ctx.clone())?;

            Self::set_times(
                &ctx,
                opened_file,
                FileTimes::new().set_created(system_time),
                task_tracker,
            )
            .await?;

            Ok(())
        }
    }

    /// Returns the current read/write position in the file.
    pub async fn position(&mut self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file_mut(&ctx)?;

        Ok(opened_file.file.lock().await.stream_position().await?)
    }

    /// Seeks to an absolute position in the file.
    pub async fn set_position(&mut self, ctx: Ctx<'_>, position: u64) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file
            .file
            .lock()
            .await
            .seek(SeekFrom::Start(position))
            .await?;

        Ok(())
    }

    /// Seeks relative to the current position (can be negative).
    pub async fn set_relative_position(&mut self, ctx: Ctx<'_>, offset: i64) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file
            .file
            .lock()
            .await
            .seek(SeekFrom::Current(offset))
            .await?;

        Ok(())
    }

    /// Rewinds the file position to the beginning.
    pub async fn rewind(&mut self, ctx: Ctx<'_>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file.file.lock().await.rewind().await?;

        Ok(())
    }

    /// The file path
    #[get]
    pub fn path(&self, ctx: Ctx<'_>) -> Result<String> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.path.to_string())
    }

    /// Returns `true` if a file exists at the given path.
    ///
    /// ```ts
    /// if (await File.exists("config.json")) {
    ///     const text = await File.readText("config.json");
    /// }
    /// ```
    #[qjs(static)]
    pub async fn exists(path: String) -> Result<bool> {
        let result = fs::try_exists(path).await?;

        Ok(result)
    }

    /// Removes a file from the filesystem.
    ///
    /// Note that there is no guarantee that the file is immediately deleted (e.g. depending on platform, other open file descriptors may prevent immediate removal).
    #[qjs(static)]
    pub async fn remove(path: String) -> Result<()> {
        fs::remove_file(path).await?;

        Ok(())
    }

    /// Copies a file from `source` to `destination`.
    #[qjs(static)]
    pub async fn copy(source: String, destination: String) -> Result<()> {
        fs::copy(source, destination).await?;

        Ok(())
    }

    /// Renames (moves) a file from `source` to `destination`. Works across filesystems.
    #[qjs(static)]
    pub async fn rename(source: String, destination: String) -> Result<()> {
        match fs::rename(&source, &destination).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::CrossesDevices => {
                fs::copy(&source, &destination).await?;
                fs::remove_file(&source).await?;
                Ok(())
            }
            Err(err) => Err(err)?,
        }
    }

    /// Alias for `rename`.
    #[qjs(static)]
    pub async fn r#move(source: String, destination: String) -> Result<()> {
        Self::rename(source, destination).await
    }

    /// Returns a clone of this file handle. Both handles share the same underlying file.
    #[qjs(rename = "clone")]
    #[must_use]
    pub fn clone_js(&self) -> Self {
        self.clone()
    }

    /// Returns `true` if both handles refer to the same file path.
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        self.inner == other.inner
    }

    /// Returns a string representation of this file.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.as_ref().map_or_else(
            || display_with_type("File", ""),
            |file| display_with_type("File", format!("path: {}", file.path)),
        )
    }
}

impl<'js> Trace<'js> for JsFile {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use std::env;

    use rquickjs::TypedArray;
    use tokio::fs;
    use tracing_test::traced_test;

    use crate::{
        api::test_helpers::{js_path, random_temp_filename},
        runtime::Runtime,
        scripting::Engine,
    };

    fn test_with_file<F, Fut>(f: F)
    where
        F: FnOnce(Engine) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Runtime::test_with_script_engine(|script_engine| async move {
            let file_path = random_temp_filename();

            script_engine
                .eval_async::<()>(&format!(
                    r#"const file = await File.open({}, {{ read: true, write: true, createNew: true }})"#,
                    js_path(&file_path)
                ))
                .await
                .unwrap();

            f(script_engine).await;

            _ = fs::remove_file(&file_path).await;
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_bytes_instance() {
        const BYTES: &[u8] = b"test";

        test_with_file(|script_engine| async move {
            script_engine
                .with(|ctx| {
                    ctx.globals()
                        .set("bytes", TypedArray::new_copy(ctx, BYTES)?)?;
                    Ok(())
                })
                .await
                .unwrap();

            script_engine
                .eval_async::<()>(
                    r#"
                await file.writeBytes(bytes);
                await file.rewind();
                var result = await file.readBytes();
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result")?;
                    Ok(result.as_bytes().unwrap().to_vec())
                })
                .await
                .unwrap();

            assert_eq!(result, BYTES);

            script_engine
                .eval_async::<()>(
                    r#"
                await file.rewind();
                result = await file.readBytes(2);
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result")?;
                    Ok(result.as_bytes().unwrap().to_vec())
                })
                .await
                .unwrap();

            assert_eq!(result, b"te");
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_bytes_static() {
        const BYTES: &[u8] = b"test";

        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .with(|ctx| {
                    ctx.globals()
                        .set("bytes", TypedArray::new_copy(ctx, BYTES)?)?;
                    Ok(())
                })
                .await
                .unwrap();

            let file_path = env::temp_dir().join("test_write_read_bytes_static.txt");
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await File.writeBytes({}, bytes);
                var result = await File.readBytes({});
                "#,
                    js_path(&file_path),
                    js_path(&file_path)
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result")?;
                    Ok(result.as_bytes().unwrap().to_vec())
                })
                .await
                .unwrap();

            assert_eq!(result, BYTES);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                var result = await File.readBytes({}, 2);
                "#,
                    js_path(&file_path)
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result")?;
                    Ok(result.as_bytes().unwrap().to_vec())
                })
                .await
                .unwrap();

            assert_eq!(result, b"te");
        });
    }
}

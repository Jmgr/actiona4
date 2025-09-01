use std::{
    fmt::Debug,
    fs::FileTimes,
    io::SeekFrom,
    os::unix::fs::PermissionsExt,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use macros::FromJsObject;
use rquickjs::{
    Ctx, Exception, Function, JsLifetime, Object, Result, TypedArray,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{Args, Constructor},
    prelude::Opt,
};
use tokio::{
    fs,
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
    task::spawn_blocking,
};

use crate::{IntoJsResult, core::js::classes::ValueClass, error::CommonError};

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

/// File open options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsOpenOptions {
    /// Should the file be opened with read access?
    /// @default true
    pub read: bool,

    /// Should the file be opened with write access?
    /// @default false
    pub write: bool,

    /// Writing: open the file in append mode.
    /// Note that setting this to `true` implies setting `write` to `true`.
    /// @default false
    pub append: bool,

    /// Writing: truncate (remove all contents of) the file.
    /// Note that this only works if `write` is `true`.
    /// @default false
    pub truncate: bool,

    /// Writing: create a new file if it doesn't exist.
    /// Note that this only works if `write` or `append` are set to `true`.
    /// @default false
    pub create: bool,

    /// Writing: always create a new file, even if one already exists.
    /// Note that this only works if `write` or `append` are set to `true`.
    /// Note that `create` and `truncate` are ignored if this is set to `true`.
    /// @default false
    pub create_new: bool,
}

impl Default for JsOpenOptions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
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

/// File represents a file handle.
///
/// @prop readonly path: string // The file path
#[derive(Clone, Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "File")]
pub struct JsFile {
    inner: Option<OpenedFile>,
}

impl ValueClass<'_> for JsFile {}

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

    fn date_from_system_time<'js>(ctx: &Ctx<'js>, system_time: &SystemTime) -> Object<'js> {
        let global = ctx.globals();
        let date_constructor: Constructor = global.get("Date").unwrap();

        let duration = system_time.duration_since(UNIX_EPOCH).unwrap();
        let millis = duration.as_millis() as f64;

        date_constructor
            .construct::<_, Object<'js>>((millis,))
            .unwrap()
    }

    fn system_time_from_date<'js>(ctx: Ctx<'js>, date: Object<'js>) -> Result<SystemTime> {
        let date_object: Object = ctx.globals().get(PredefinedAtom::Date)?;
        if !date.is_instance_of(&date_object) {
            return Err(Exception::throw_message(
                &ctx,
                &format!("Expected a Date parameter, got {}", date.type_name()),
            ));
        }

        let get_time: Function = date.get("getTime")?;
        let mut args = Args::new(ctx, 0);
        args.this(date)?;
        let time: u64 = get_time.call_arg(args)?;

        Ok(UNIX_EPOCH + Duration::from_millis(time))
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsFile {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
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
    pub const fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    /// Closes this file handle.
    /// Please note that the actual file might not be closed until all other handles to it are also closed.
    /// This can happen if you cloned() this File.
    pub fn close(&mut self) {
        self.inner = None;
    }

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

        opened_file
            .file
            .lock()
            .await
            .write(bytes.as_bytes().unwrap())
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;
        opened_file.file.lock().await.flush().await?;

        Ok(())
    }

    /// @static
    ///
    /// @param path: string
    /// @param bytes: Uint8Array
    #[qjs(static)]
    pub async fn write_bytes(ctx: Ctx<'_>, path: String, bytes: TypedArray<'_, u8>) -> Result<()> {
        let bytes = bytes
            .as_bytes()
            .ok_or(CommonError::DetachedArrayBuffer)
            .into_js(&ctx)?;

        fs::write(path, bytes)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error writing file: {err}")))?;

        Ok(())
    }

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
            result.resize(amount as usize, 0);
            opened_file
                .file
                .lock()
                .await
                .read_exact(&mut result)
                .await?;
        } else {
            // Note that we can't just call file.metadata here as that would cause a stack overflow
            let len = fs::metadata(&opened_file.path).await?.len();
            result.reserve(len as usize);
            opened_file
                .file
                .lock()
                .await
                .read_to_end(&mut result)
                .await?;
        }

        TypedArray::new(ctx, result)
    }

    #[qjs(static)]
    pub async fn read_bytes(
        ctx: Ctx<'_>,
        path: String,
        amount: Opt<u64>,
    ) -> Result<TypedArray<'_, u8>> {
        let result = if let Some(amount) = amount.0 {
            let mut result = vec![0; amount as usize];
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

    #[qjs(static)]
    pub async fn read_text(ctx: Ctx<'_>, path: String) -> Result<String> {
        fs::read_to_string(path)
            .await
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error reading file: {err}")))
    }

    #[qjs(rename = "size")]
    pub async fn size(&mut self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.lock().await.metadata().await?.len())
    }

    pub async fn set_size(&self, ctx: Ctx<'_>, size: u64) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.lock().await.set_len(size).await?)
    }

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

    pub async fn set_readonly(&self, ctx: Ctx<'_>, readonly: bool) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        let file = opened_file.file.lock().await;

        let mut permissions = file.metadata().await?.permissions();

        permissions.set_readonly(readonly);

        file.set_permissions(permissions).await?;

        Ok(())
    }

    /// @platforms -windows
    pub async fn mode(&self, ctx: Ctx<'_>) -> Result<u32> {
        let opened_file = self.opened_file(&ctx)?;

        #[cfg(unix)]
        return Ok(opened_file
            .file
            .lock()
            .await
            .metadata()
            .await?
            .permissions()
            .mode());

        #[cfg(windows)]
        return Ok(0);
    }

    /// Sets the file mode.
    /// You should use the octal notation to specify the mode: `await file.setMode(0o445)`.
    /// @platforms -windows
    pub async fn set_mode(&self, ctx: Ctx<'_>, mode: u32) -> Result<()> {
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
        return Ok(());
    }

    /// @returns Date
    pub async fn modified_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.modified()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    #[qjs(skip)]
    async fn set_times(opened_file: &OpenedFile, times: FileTimes) -> Result<()> {
        let path = opened_file.path.clone();

        spawn_blocking(move || {
            let file = std::fs::File::options().write(true).open(path)?;
            file.set_times(times)?; // No implemented in tokio::fs
            Result::<_>::Ok(())
        })
        .await
        .unwrap()
    }

    /// @param date: Date
    pub async fn set_modified_time<'js>(&self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;
        let system_time = Self::system_time_from_date(ctx, date)?;

        Self::set_times(opened_file, FileTimes::new().set_modified(system_time)).await?;

        Ok(())
    }

    /// @returns Date
    pub async fn accessed_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.accessed()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    /// @param date: Date
    pub async fn set_accessed_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let system_time = Self::system_time_from_date(ctx, date)?;

        Self::set_times(opened_file, FileTimes::new().set_accessed(system_time)).await?;

        Ok(())
    }

    /// @returns Date
    pub async fn creation_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.lock().await.metadata().await?;
        let modified = metadata.created()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    /// @param date: Date
    /// @platforms -linux
    pub async fn set_creation_time(&mut self, ctx: Ctx<'_>, date: Object<'_>) -> Result<()> {
        #[cfg(unix)]
        {
            let _ = ctx;
            let _ = date;
            Ok(())
        }

        #[cfg(windows)]
        {
            let opened_file = self.opened_file(&ctx)?;
            let system_time = Self::system_time_from_date(ctx, date)?;

            Self::set_times(&opened_file, FileTimes::new().set_created(system_time)).await?;

            Ok(())
        }
    }

    pub async fn position(&mut self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file_mut(&ctx)?;

        Ok(opened_file.file.lock().await.stream_position().await?)
    }

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

    pub async fn rewind(&mut self, ctx: Ctx<'_>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file.file.lock().await.rewind().await?;

        Ok(())
    }

    /// @skip
    #[qjs(get)]
    pub fn path(&self, ctx: Ctx<'_>) -> Result<String> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.path.to_string())
    }

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

    #[qjs(static)]
    pub async fn copy(source: String, destination: String) -> Result<()> {
        fs::copy(source, destination).await?;

        Ok(())
    }

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

    #[qjs(static)]
    pub async fn r#move(source: String, destination: String) -> Result<()> {
        Self::rename(source, destination).await
    }

    #[qjs(rename = "clone")]
    pub fn clone_js(&self) -> Self {
        self.clone()
    }

    pub fn equals(&self, other: Self) -> bool {
        self.inner == other.inner
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string_js(&self) -> String {
        if let Some(file) = &self.inner {
            format!("(path: {})", file.path)
        } else {
            "()".to_string()
        }
    }
}

impl<'js> Trace<'js> for JsFile {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use std::env::{self};

    use chrono::{DateTime, Datelike, Timelike, Utc};
    use rquickjs::{Object, TypedArray};
    use tokio::fs;
    use tracing_test::traced_test;

    use crate::{
        core::{file::js::JsFile, test_helpers::random_temp_filename},
        runtime::Runtime,
        scripting::Engine,
    };

    fn test_with_file<F>(f: F)
    where
        F: AsyncFn(&mut Engine) + Clone + 'static,
    {
        Runtime::test_with_script_engine(async move |script_engine| {
            let file_path = random_temp_filename();

            script_engine
                .eval_async::<()>(&format!(
                    r#"const file = await File.open("{}", {{ read: true, write: true, createNew: true }})"#,
                    file_path.display()
                ))
                .await
                .unwrap();

            f(script_engine).await;

            let _ = fs::remove_file(&file_path).await;
        });
    }

    #[test]
    #[traced_test]
    fn test_is_open() {
        test_with_file(async move |script_engine| {
            let is_open = script_engine
                .eval_async::<bool>("await file.isOpen()")
                .await
                .unwrap();
            assert!(is_open);
        });
    }

    #[test]
    #[traced_test]
    fn test_close() {
        test_with_file(async move |script_engine| {
            let is_open = script_engine
                .eval_async::<bool>(
                    "
                await file.close();
                await file.isOpen()
                ",
                )
                .await
                .unwrap();
            assert!(!is_open);

            let result = script_engine
                .eval_async::<()>(r#"await file.setSize(42)"#)
                .await;
            assert_eq!(result.unwrap_err().to_string(), "File is not open");
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_text_instance() {
        const TEXT: &str = "test";

        test_with_file(async move |script_engine| {
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                await file.writeText("{TEXT}");
                await file.rewind();
                await file.readText()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, TEXT);
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_text_static() {
        const TEXT: &str = "test";

        Runtime::test_with_script_engine(async move |script_engine| {
            let file_path = env::temp_dir().join("test_write_read_text_static.txt");
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                await File.writeText("{}", "{TEXT}");
                await File.readText("{}")
                "#,
                    file_path.display(),
                    file_path.display()
                ))
                .await
                .unwrap();

            assert_eq!(result, TEXT);
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_bytes_instance() {
        const BYTES: &[u8] = b"test";

        test_with_file(async move |script_engine| {
            script_engine
                .with(|ctx| {
                    ctx.globals()
                        .set("bytes", TypedArray::new_copy(ctx, BYTES).unwrap())
                        .unwrap();
                })
                .await;

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.writeBytes(bytes);
                await file.rewind();
                var result = await file.readBytes();
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result").unwrap();
                    result.as_bytes().unwrap().to_vec()
                })
                .await;

            assert_eq!(result, BYTES);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.rewind();
                result = await file.readBytes(2);
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result").unwrap();
                    result.as_bytes().unwrap().to_vec()
                })
                .await;

            assert_eq!(result, b"te");
        });
    }

    #[test]
    #[traced_test]
    fn test_write_read_bytes_static() {
        const BYTES: &[u8] = b"test";

        Runtime::test_with_script_engine(async move |script_engine| {
            script_engine
                .with(|ctx| {
                    ctx.globals()
                        .set("bytes", TypedArray::new_copy(ctx, BYTES).unwrap())
                        .unwrap();
                })
                .await;

            let file_path = env::temp_dir().join("test_write_read_bytes_static.txt");
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await File.writeBytes("{}", bytes);
                var result = await File.readBytes("{}");
                "#,
                    file_path.display(),
                    file_path.display()
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result").unwrap();
                    result.as_bytes().unwrap().to_vec()
                })
                .await;

            assert_eq!(result, BYTES);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                var result = await File.readBytes("{}", 2);
                "#,
                    file_path.display()
                ))
                .await
                .unwrap();

            let result = script_engine
                .with::<_, Vec<u8>>(|ctx| {
                    let result = ctx.globals().get::<_, TypedArray<u8>>("result").unwrap();
                    result.as_bytes().unwrap().to_vec()
                })
                .await;

            assert_eq!(result, b"te");
        });
    }

    #[test]
    #[traced_test]
    fn test_size() {
        const TEXT: &str = "test";

        test_with_file(async move |script_engine| {
            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.writeText("{TEXT}");
                await file.size()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, TEXT.len());

            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.setSize(42);
                await file.size()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 42);
        });
    }

    #[test]
    #[traced_test]
    fn test_readonly() {
        test_with_file(async move |script_engine| {
            let readonly = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await file.readonly()
                "#
                ))
                .await
                .unwrap();

            assert!(!readonly);

            let readonly = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                await file.setReadonly(true);
                await file.readonly()
                "#
                ))
                .await
                .unwrap();

            assert!(readonly);
        });
    }

    #[test]
    #[traced_test]
    fn test_mode() {
        test_with_file(async move |script_engine| {
            let mode = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.setMode(0o445);
                await file.mode()
                "#
                ))
                .await
                .unwrap();

            #[cfg(unix)]
            assert_eq!(mode & 0o777, 0o445);

            #[cfg(windows)]
            assert_eq!(mode & 0o777, 0);
        });
    }

    #[test]
    #[traced_test]
    fn test_times() {
        const YEAR: i32 = 1996;
        const MONTH: u32 = 2;
        const DAY: u32 = 10;
        const HOUR: u32 = 6;
        const MINUTE: u32 = 46;
        const SECOND: u32 = 16;
        const MILLISECOND: u32 = 468;

        test_with_file(async move |script_engine| {
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setModifiedTime(new Date({YEAR}, {MONTH}, {DAY}, {HOUR}, {MINUTE}, {SECOND}, {MILLISECOND}));
                var result = await file.modifiedTime()
                "#
                ))
                .await
                .unwrap();

            let time = script_engine
                .with::<_, DateTime<Utc>>(|ctx| {
                    let result = ctx.globals().get::<_, Object>("result").unwrap();
                    JsFile::system_time_from_date(ctx, result).unwrap().into()
                })
                .await;

            assert_eq!(time.year(), YEAR);
            assert_eq!(time.month0(), MONTH);
            assert_eq!(time.day(), DAY);
            assert_eq!(time.hour(), HOUR);
            assert_eq!(time.minute(), MINUTE);
            assert_eq!(time.second(), SECOND);
            assert_eq!(time.timestamp_subsec_millis(), MILLISECOND);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setAccessedTime(new Date({YEAR}, {MONTH}, {DAY}, {HOUR}, {MINUTE}, {SECOND}, {MILLISECOND}));
                var result = await file.accessedTime()
                "#
                ))
                .await
                .unwrap();

            let time = script_engine
                .with::<_, DateTime<Utc>>(|ctx| {
                    let result = ctx.globals().get::<_, Object>("result").unwrap();
                    JsFile::system_time_from_date(ctx, result).unwrap().into()
                })
                .await;

            assert_eq!(time.year(), YEAR);
            assert_eq!(time.month0(), MONTH);
            assert_eq!(time.day(), DAY);
            assert_eq!(time.hour(), HOUR);
            assert_eq!(time.minute(), MINUTE);
            assert_eq!(time.second(), SECOND);
            assert_eq!(time.timestamp_subsec_millis(), MILLISECOND);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setCreationTime(new Date({YEAR}, {MONTH}, {DAY}, {HOUR}, {MINUTE}, {SECOND}, {MILLISECOND}));
                var result = await file.creationTime()
                "#
                ))
                .await
                .unwrap();

            let time = script_engine
                .with::<_, DateTime<Utc>>(|ctx| {
                    let result = ctx.globals().get::<_, Object>("result").unwrap();
                    JsFile::system_time_from_date(ctx, result).unwrap().into()
                })
                .await;

            #[cfg(unix)]
            let _ = time;

            #[cfg(windows)]
            {
                assert_eq!(time.year(), YEAR);
                assert_eq!(time.month0(), MONTH);
                assert_eq!(time.day(), DAY);
                assert_eq!(time.hour(), HOUR);
                assert_eq!(time.minute(), MINUTE);
                assert_eq!(time.second(), SECOND);
                assert_eq!(time.timestamp_subsec_millis(), MILLISECOND);
            }
        });
    }

    #[test]
    #[traced_test]
    fn test_position() {
        test_with_file(async move |script_engine| {
            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.position()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 0);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setPosition(2)
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.position()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 2);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setRelativePosition(1)
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.position()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 3);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.setRelativePosition(-1)
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.position()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 2);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                await file.rewind()
                "#
                ))
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<usize>(&format!(
                    r#"
                await file.position()
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, 0);
        });
    }

    #[test]
    #[traced_test]
    fn test_path() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let file_path = env::temp_dir().join("test_with_script_engine.txt");

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                var file = await File.open("{}", {{ create: true, write: true }})
                "#,
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<String>("file.path")
                .await
                .unwrap();

            assert_eq!(result, file_path.to_string_lossy());
        });
    }

    #[test]
    #[traced_test]
    fn test_exists() {
        test_with_file(async move |script_engine| {
            let file_path = env::temp_dir().join("test_exists.txt");

            let result = script_engine
                .eval_async::<bool>(
                    r#"
                    await File.exists(file.path)
                "#,
                )
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                    await File.exists("{}")
                "#,
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert!(!result);
        });
    }

    #[test]
    #[traced_test]
    fn test_remove() {
        test_with_file(async move |script_engine| {
            let result = script_engine
                .eval_async::<bool>(
                    r#"
                    await File.exists(file.path)
                "#,
                )
                .await
                .unwrap();
            assert!(result);

            script_engine
                .eval_async::<()>(
                    r#"
                    await File.remove(file.path)
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<bool>(
                    r#"
                    await File.exists(file.path)
                "#,
                )
                .await
                .unwrap();
            assert!(!result);
        });
    }

    #[test]
    #[traced_test]
    fn test_copy() {
        const TEXT: &str = "test";

        test_with_file(async move |script_engine| {
            let file_path = env::temp_dir().join("test_copy.txt");

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                await file.writeText("{TEXT}");
                await File.copy(file.path, "{}");
                await File.readText("{}")
                "#,
                    file_path.to_string_lossy(),
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert_eq!(result, TEXT);
        });
    }

    #[test]
    #[traced_test]
    fn test_rename() {
        const TEXT: &str = "test";

        test_with_file(async move |script_engine| {
            let file_path = env::temp_dir().join("test_rename.txt");

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                await file.writeText("{TEXT}");
                await File.rename(file.path, "{}");
                await File.readText("{}")
                "#,
                    file_path.to_string_lossy(),
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
            assert_eq!(result, TEXT);

            let result = script_engine
                .eval_async::<bool>(
                    r#"
                    await File.exists(file.path)
                "#,
                )
                .await
                .unwrap();
            assert!(!result);

            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    await File.remove("{}")
                "#,
                    file_path.to_string_lossy()
                ))
                .await
                .unwrap();
        });
    }
}

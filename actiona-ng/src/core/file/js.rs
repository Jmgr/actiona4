use std::{
    fs,
    fs::FileTimes,
    io::{Read, Seek, SeekFrom, Write},
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

use crate::core::ValueClass;

/// Open options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsOpenOptions {
    /// Open the file for reading
    /// @default true
    pub read: bool,

    /// Open the file for writing
    /// @default true
    pub write: bool,

    /// When writing, create a new file if it doesn't exist already
    /// @default true
    pub create: bool,

    /// When writing, truncate the file (erase all contents)
    /// @default false
    pub truncate: bool,
}

impl Default for JsOpenOptions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            create: true,
            truncate: false,
        }
    }
}

#[derive(Clone, Debug, JsLifetime)]
struct OpenedFile {
    path: String,
    file: Arc<fs::File>,
}

impl PartialEq for OpenedFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

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

// TODO: move, copy, rename
// TODO: directory

#[rquickjs::methods(rename_all = "camelCase")]
impl JsFile {
    /// Creates a new file.
    ///
    /// Example
    /// ```js
    /// let file = new File();
    /// ```
    ///
    /// @constructor
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Opens a new file.
    ///
    /// Example
    /// ```js
    /// let file = new File.open("my_file.txt");
    /// let file = new File.open("my_file.txt", {
    ///     read: true,
    ///     write: false,
    /// });
    /// ```
    #[qjs(static)]
    pub fn open(ctx: Ctx<'_>, path: String, options: Opt<JsOpenOptions>) -> Result<Self> {
        let options = options.unwrap_or_default();

        let file = fs::OpenOptions::new()
            .read(options.read)
            .write(options.write)
            .create(options.create)
            .truncate(options.truncate)
            .open(&path)
            .map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error opening the file: {}", err))
            })?;

        Ok(Self {
            inner: Some(OpenedFile {
                path: path.clone(),
                file: Arc::new(file),
            }),
        })
    }

    /// Returns true if the file is open.
    pub fn is_open(&self) -> bool {
        self.inner.is_some()
    }

    /// Closes this file handle.
    /// Please note that the actual file might not be closed until all other handles to it are also closed.
    /// This can happen if you cloned() this File.
    pub fn close(&mut self) {
        self.inner = None;
    }

    /// @rename write
    ///
    /// @overload
    /// @param text: string
    ///
    /// @overload
    /// @param data: Uint8Array
    #[qjs(rename = "write")]
    pub fn write_instance<'js>(&mut self, ctx: Ctx<'js>, value: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        if let Some(typed_array) = value.as_typed_array::<u8>() {
            opened_file
                .file
                .write(typed_array.as_bytes().unwrap())
                .map_err(|err| {
                    Exception::throw_message(&ctx, &format!("Error writing file: {}", err))
                })?;
        } else if let Some(text) = value.as_string() {
            opened_file
                .file
                .write(text.to_string().unwrap().as_bytes())
                .map_err(|err| {
                    Exception::throw_message(&ctx, &format!("Error writing file: {}", err))
                })?;
        } else {
            return Err(Exception::throw_message(
                &ctx,
                "Expected a string or a Uint8Array",
            ));
        }

        Ok(())
    }

    /// @static
    ///
    /// @overload
    /// @param path: string
    /// @param data: Uint8Array
    ///
    /// @overload
    /// @param path: string
    /// @param text: string
    #[qjs(static)]
    pub fn write<'js>(ctx: Ctx<'js>, path: String, value: Object<'js>) -> Result<()> {
        if let Some(typed_array) = value.as_typed_array::<u8>() {
            fs::write(path, typed_array.as_bytes().unwrap()).map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error writing file: {}", err))
            })?;
        } else if let Some(text) = value.as_string() {
            fs::write(path, text.to_string().unwrap()).map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error writing file: {}", err))
            })?;
        } else {
            return Err(Exception::throw_message(
                &ctx,
                "Expected a string or a Uint8Array",
            ));
        }

        Ok(())
    }

    /// @rename readAllText
    #[qjs(rename = "readAllText")]
    pub fn read_all_text_instance(&mut self, ctx: Ctx<'_>) -> Result<String> {
        let opened_file = self.opened_file_mut(&ctx)?;

        let mut result = String::new();
        opened_file
            .file
            .read_to_string(&mut result)
            .map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error reading from the file: {}", err))
            })?;

        Ok(result)
    }

    #[qjs(static)]
    pub fn read_all_text(ctx: Ctx<'_>, path: String) -> Result<String> {
        fs::read_to_string(path)
            .map_err(|err| Exception::throw_message(&ctx, &format!("Error reading file: {}", err)))
    }

    pub fn read_bytes<'js>(
        &mut self,
        ctx: Ctx<'js>,
        amount: Opt<u64>,
    ) -> Result<TypedArray<'js, u8>> {
        let opened_file = self.opened_file_mut(&ctx)?;

        let mut result = Vec::new();

        if let Some(amount) = amount.0 {
            result.resize(amount as usize, 0);
            opened_file.file.read_exact(&mut result)?;
        } else {
            opened_file.file.read_to_end(&mut result).map_err(|err| {
                Exception::throw_message(&ctx, &format!("Error reading from the file: {}", err))
            })?;
        }

        TypedArray::new(ctx, result)
    }

    #[qjs(static)]
    pub fn read_all_bytes(ctx: Ctx<'_>, path: String) -> Result<TypedArray<'_, u8>> {
        let result = fs::read(path).map_err(|err| {
            Exception::throw_message(&ctx, &format!("Error reading file: {}", err))
        })?;

        TypedArray::new(ctx, result)
    }

    pub fn size(&self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.metadata()?.len())
    }

    pub fn set_size(&self, ctx: Ctx<'_>, size: u64) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.set_len(size)?)
    }

    pub fn readonly(&self, ctx: Ctx<'_>) -> Result<bool> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(opened_file.file.metadata()?.permissions().readonly())
    }

    pub fn set_readonly(&self, ctx: Ctx<'_>, readonly: bool) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        opened_file
            .file
            .metadata()?
            .permissions()
            .set_readonly(readonly);

        Ok(())
    }

    /// Note that this returns 0 on Windows.
    pub fn mode(&self, ctx: Ctx<'_>) -> Result<u32> {
        let opened_file = self.opened_file(&ctx)?;

        #[cfg(unix)]
        return Ok(opened_file.file.metadata()?.permissions().mode());

        #[cfg(windows)]
        return Ok(0);
    }

    /// Note that this does nothing on Windows.
    pub fn set_mode(&self, ctx: Ctx<'_>, mode: u32) -> Result<()> {
        let opened_file = self.opened_file(&ctx)?;

        #[cfg(unix)]
        return {
            opened_file.file.metadata()?.permissions().set_mode(mode);
            Ok(())
        };

        #[cfg(windows)]
        return Ok(());
    }

    /// @returns Date
    pub fn modified_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.metadata()?;
        let modified = metadata.modified()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    /// @param date: Date
    pub fn set_modified_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let system_time = Self::system_time_from_date(ctx, date)?;

        opened_file.file.set_modified(system_time)?;

        Ok(())
    }

    /// @returns Date
    pub fn accessed_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.metadata()?;
        let modified = metadata.accessed()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    /// @param date: Date
    pub fn set_accessed_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;
        let system_time = Self::system_time_from_date(ctx, date)?;

        opened_file
            .file
            .set_times(FileTimes::new().set_accessed(system_time))?;

        Ok(())
    }

    /// @returns Date
    pub fn created_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let opened_file = self.opened_file(&ctx)?;
        let metadata = opened_file.file.metadata()?;
        let modified = metadata.created()?;
        Ok(Self::date_from_system_time(&ctx, &modified))
    }

    /// @param date: Date
    /// Note that this does nothing on Linux.
    pub fn set_created_time<'js>(&mut self, ctx: Ctx<'js>, date: Object<'js>) -> Result<()> {
        #[cfg(unix)]
        {
            let _ = ctx;
            let _ = date;
            Ok(())
        }

        #[cfg(windows)]
        {
            let opened_file = self.opened_file_mut(&ctx)?;
            let system_time = Self::system_time_from_date(ctx, date)?;

            opened_file
                .file
                .set_times(FileTimes::new().set_created(system_time))?;

            Ok(())
        }
    }

    pub fn position(&mut self, ctx: Ctx<'_>) -> Result<u64> {
        let opened_file = self.opened_file_mut(&ctx)?;

        Ok(opened_file.file.stream_position()?)
    }

    pub fn set_position(&mut self, ctx: Ctx<'_>, position: u64) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file.file.seek(SeekFrom::Start(position))?;

        Ok(())
    }

    pub fn set_relative_position(&mut self, ctx: Ctx<'_>, offset: i64) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file.file.seek_relative(offset)?;

        Ok(())
    }

    pub fn rewind(&mut self, ctx: Ctx<'_>) -> Result<()> {
        let opened_file = self.opened_file_mut(&ctx)?;

        opened_file.file.rewind()?;

        Ok(())
    }

    pub fn path(&self, ctx: Ctx<'_>) -> Result<&str> {
        let opened_file = self.opened_file(&ctx)?;

        Ok(&opened_file.path)
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
        String::new()
    }
}

impl<'js> Trace<'js> for JsFile {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;
    use zbus::blocking::Connection;

    use crate::{eval, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_() {
        Runtime::test_with_js(async |js_context| {
            //eval::<()>(&js_context, "").unwrap();
        });
    }
}

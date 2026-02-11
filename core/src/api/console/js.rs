use std::{collections::HashMap, io::Write, time::Instant};

use console::{Style, Term};
use humantime::format_duration;
use itertools::Itertools;
use rquickjs::{
    Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    prelude::*,
};

use crate::api::js::classes::SingletonClass;

/// The global console singleton for printing output and basic debugging.
///
/// ```ts
/// // Print values
/// println("hello", 42, { key: "value" });
///
/// // Warnings and errors are styled
/// console.warn("this is a warning");
/// console.error("something went wrong");
///
/// // Measure elapsed time
/// console.time("fetch");
/// // ... do work ...
/// console.timeEnd("fetch"); // prints "fetch: 1s 234ms - timer ended"
///
/// // Count how many times a label is hit
/// console.count("loop");
/// console.count("loop");
/// ```
///
/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "Console")]
pub struct JsConsole {
    timers: HashMap<String, Instant>,
    counters: HashMap<String, usize>,
}

impl<'js> Trace<'js> for JsConsole {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsConsole {}

impl JsConsole {
    fn try_custom_to_string<'js>(
        ctx: &Ctx<'js>,
        obj: &rquickjs::Object<'js>,
        value: &Value<'js>,
    ) -> Option<String> {
        let global_prototype = ctx.globals().get_prototype()?;
        let default_to_string = global_prototype
            .get::<_, rquickjs::Function>("toString")
            .ok()?;
        let to_string = obj.get::<_, rquickjs::Function>("toString").ok()?;
        if to_string == default_to_string {
            return None;
        }
        let mut args = rquickjs::function::Args::new(ctx.clone(), 0);
        args.this(value.clone()).ok()?;
        to_string.call_arg::<String>(args).ok()
    }

    fn inspect_string(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len() + 2);
        for ch in value.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '\'' => escaped.push_str("\\'"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\0' => escaped.push_str("\\0"),
                c if c.is_control() => {
                    // Escape remaining control characters as \xHH
                    for byte in c.to_string().bytes() {
                        escaped.push_str(&format!("\\x{byte:02x}"));
                    }
                }
                c => escaped.push(c),
            }
        }
        format!("'{escaped}'")
    }

    fn print_value<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> String {
        use rquickjs::*;

        match value.type_of() {
            Type::Uninitialized => "uninitialized".to_string(),
            Type::Undefined => "undefined".to_string(),
            Type::Null => "null".to_string(),
            Type::Bool => format!("{}", value.as_bool().unwrap()),
            Type::Int => format!("{}", value.as_int().unwrap()),
            Type::Float => format!("{}", value.as_float().unwrap()),
            Type::String => {
                let string_value = value.as_string().unwrap().to_string().unwrap();
                Self::inspect_string(&string_value)
            }
            Type::Symbol => "[Symbol]".to_string(),
            Type::Array => {
                let arr = value.as_array().unwrap();
                let mut elems = vec![];
                for i in 0..arr.len() {
                    let item = arr.get(i).unwrap();
                    elems.push(Self::print_value(ctx, item));
                }

                if elems.is_empty() {
                    "[]".to_string()
                } else {
                    format!("[ {} ]", elems.join(", "))
                }
            }
            Type::Constructor | Type::Function => {
                let obj = value.as_object().unwrap();

                let kind = obj
                    .get::<_, Object>("constructor")
                    .ok()
                    .and_then(|constructor| constructor.get::<_, String>("name").ok())
                    .and_then(|name| name.to_string().ok())
                    .unwrap_or_else(|| "Function".to_string());

                let name = obj
                    .get::<_, String>("name")
                    .ok()
                    .and_then(|name| name.to_string().ok())
                    .unwrap_or_default();

                if name.is_empty() {
                    format!("[{kind} (anonymous)]")
                } else {
                    format!("[{kind}: {name}]")
                }
            }
            Type::Promise => "[Promise]".to_string(),
            Type::Exception | Type::Object => {
                let obj = value.as_object().unwrap();

                if let Some(s) = Self::try_custom_to_string(ctx, obj, &value) {
                    return s;
                }

                let mut fields = vec![];
                for key in obj.keys::<String>() {
                    let key = key.unwrap();
                    let val: Value = obj.get(key.clone()).unwrap();
                    let key_str = key.to_string().unwrap();
                    let val_str = Self::print_value(ctx, val);
                    fields.push(format!("{key_str}: {val_str}"));
                }
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{ {} }}", fields.join(", "))
                }
            }
            Type::Module => "[Module]".to_string(),
            Type::BigInt => value
                .get::<Coerced<String>>()
                .unwrap()
                .0
                .to_string()
                .unwrap(),
            Type::Unknown => "[Unknown]".to_string(),
            Type::Proxy => "[Proxy]".to_string(),
        }
    }

    fn print_value_pretty<'js>(ctx: &Ctx<'js>, value: Value<'js>, indent: usize) -> String {
        use rquickjs::*;

        match value.type_of() {
            Type::Array => {
                let arr = value.as_array().unwrap();
                if arr.is_empty() {
                    return "[]".to_string();
                }

                let indentation = " ".repeat(indent + 2);
                let mut elems = vec![];
                for i in 0..arr.len() {
                    let item = arr.get(i).unwrap();
                    let item_str = Self::print_value_pretty(ctx, item, indent + 2);
                    elems.push(format!("{indentation}{item_str}"));
                }

                format!("[\n{}\n{}]", elems.join(",\n"), " ".repeat(indent))
            }
            Type::Exception | Type::Object => {
                let obj = value.as_object().unwrap();

                if let Some(s) = Self::try_custom_to_string(ctx, obj, &value) {
                    return s;
                }

                let mut fields = vec![];
                let indentation = " ".repeat(indent + 2);
                for key in obj.keys::<String>() {
                    let key = key.unwrap();
                    let val: Value = obj.get(key.clone()).unwrap();
                    let key_str = key.to_string().unwrap();
                    let val_str = Self::print_value_pretty(ctx, val, indent + 2);
                    fields.push(format!("{indentation}{key_str}: {val_str}"));
                }

                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!("{{\n{}\n{}}}", fields.join(",\n"), " ".repeat(indent))
                }
            }
            _ => Self::print_value(ctx, value),
        }
    }

    pub(crate) fn args_to_string<'js>(ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> String {
        args.0
            .into_iter()
            .map(|arg| {
                use rquickjs::Type;

                match arg.type_of() {
                    // Top-level string: no quotes
                    Type::String => arg.as_string().unwrap().to_string().unwrap(),
                    // Everything else: use full inspector
                    _ => Self::print_value(ctx, arg),
                }
            })
            .join(" ")
    }

    pub(crate) fn args_to_pretty_string<'js>(ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> String {
        args.0
            .into_iter()
            .map(|arg| Self::print_value_pretty(ctx, arg, 0))
            .join("\n")
    }

    pub(crate) fn do_print<'js>(ctx: &Ctx<'js>, data: Rest<Value<'js>>) {
        print!("{}", Self::args_to_string(ctx, data));
        std::io::stdout().flush().unwrap();
    }

    pub(crate) fn do_println<'js>(ctx: &Ctx<'js>, data: Rest<Value<'js>>) {
        println!("{}", Self::args_to_string(ctx, data));
    }

    pub(crate) fn do_inspect<'js>(ctx: &Ctx<'js>, data: Rest<Value<'js>>) {
        println!("{}", Self::args_to_pretty_string(ctx, data));
    }
}

const DEFAULT_LABEL: &str = "default";

#[rquickjs::methods(rename_all = "camelCase")]
impl JsConsole {
    /// Prints values without a trailing newline.
    /// @rest
    pub fn print<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        Self::do_print(&ctx, data);
    }

    /// Prints values followed by a newline.
    /// @rest
    pub fn println<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        Self::do_println(&ctx, data);
    }

    /// Logs values to stdout. Alias for `println`.
    /// @rest
    pub fn log<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        Self::do_println(&ctx, data);
    }

    /// Logs informational values. Alias for `log`.
    /// @rest
    pub fn info<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        Self::do_println(&ctx, data);
    }

    /// Logs a warning in yellow.
    /// @rest
    pub fn warn<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        let yellow = Style::new().yellow();
        println!("{}", yellow.apply_to(Self::args_to_string(&ctx, data)));
    }

    /// Logs an error in bold red.
    /// @rest
    pub fn error<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        let red = Style::new().red().bold();
        println!("{}", red.apply_to(Self::args_to_string(&ctx, data)));
    }

    /// Pretty-prints values using an inspect-style multiline format.
    /// @rest
    pub fn inspect<'js>(&self, ctx: Ctx<'js>, data: Rest<Value<'js>>) {
        Self::do_inspect(&ctx, data);
    }

    /// Clears the terminal screen.
    pub fn clear(&self) {
        let term = Term::stdout();
        term.clear_screen().unwrap();
    }

    /// Starts a timer with the given label (defaults to `"default"`).
    ///
    /// ```ts
    /// console.time("myTimer");
    /// ```
    pub fn time(&mut self, label: Opt<String>) {
        let name = label.clone().unwrap_or_else(|| DEFAULT_LABEL.to_string());
        self.timers.insert(name, Instant::now());
    }

    /// Stops a timer and prints the elapsed time.
    ///
    /// ```ts
    /// console.time("myTimer");
    /// // ... do work ...
    /// console.timeEnd("myTimer"); // prints "myTimer: 1s 234ms - timer ended"
    /// ```
    pub fn time_end(&mut self, ctx: Ctx<'_>, label: Opt<String>) -> Result<()> {
        let label = label.clone().unwrap_or_else(|| DEFAULT_LABEL.to_string());
        if let Some(timer_start) = self.timers.remove(&label) {
            println!(
                "{label}: {} - timer ended",
                format_duration(Instant::now() - timer_start)
            );
        } else {
            return Err(Exception::throw_message(
                &ctx,
                &format!("Timer \"{label}\" doesn't exist."),
            ));
        };

        Ok(())
    }

    /// Increments and prints a counter for the given label (defaults to `"default"`).
    ///
    /// ```ts
    /// console.count("loop"); // prints "loop: 1"
    /// console.count("loop"); // prints "loop: 2"
    /// ```
    pub fn count(&mut self, label: Opt<String>) {
        let label = label.clone().unwrap_or_else(|| DEFAULT_LABEL.to_string());
        let value = self.counters.entry(label.clone()).or_default();
        *value += 1;
        println!("{label}: {value}");
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Context, Runtime as JsRuntime, prelude::Rest};

    use crate::runtime::Runtime;

    #[test]
    fn test_log() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval::<()>(
                    r#"
                let a = {
                    "foo": 10
                };
                console.log("hello", a)"#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_exception_values_use_js_to_string() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx
                .eval(
                    r#"
                    (() => {
                        try {
                            throw new Error("Failed quickly");
                        } catch (e) {
                            return e;
                        }
                    })()
                    "#,
                )
                .unwrap();

            let output = super::JsConsole::print_value(&ctx, value);
            assert_eq!(output, "Error: Failed quickly");
        });
    }

    #[test]
    fn test_array_values_use_node_style_format() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval("[4, 6]").unwrap();
            let output = super::JsConsole::print_value(&ctx, value);
            assert_eq!(output, "[ 4, 6 ]");
        });
    }

    #[test]
    fn test_object_values_use_node_style_format() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval("({ a: 1, b: 'x' })").unwrap();
            let output = super::JsConsole::print_value(&ctx, value);
            assert_eq!(output, "{ a: 1, b: 'x' }");
        });
    }

    #[test]
    fn test_named_function_values_use_node_style_format() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval("function move() {}; move").unwrap();
            let output = super::JsConsole::print_value(&ctx, value);
            assert_eq!(output, "[Function: move]");
        });
    }

    #[test]
    fn test_anonymous_function_values_use_node_style_format() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval("(function(){})").unwrap();
            let output = super::JsConsole::print_value(&ctx, value);
            assert_eq!(output, "[Function (anonymous)]");
        });
    }

    #[test]
    fn test_pretty_print_uses_multiline_format() {
        let runtime = JsRuntime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval("({ a: 1, b: [2, { c: 'x' }] })").unwrap();
            let output = super::JsConsole::args_to_pretty_string(&ctx, Rest(vec![value]));
            assert_eq!(
                output,
                "{\n  a: 1,\n  b: [\n    2,\n    {\n      c: 'x'\n    }\n  ]\n}"
            );
        });
    }
}

use std::{collections::HashMap, time::Instant};

use console::{Style, Term};
use humantime::format_duration;
use itertools::Itertools;
use rquickjs::{
    Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    function::Args,
    prelude::*,
};

use crate::core::js::classes::SingletonClass;

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
    fn print_value<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> String {
        use rquickjs::*;

        match value.type_of() {
            Type::Uninitialized => "uninitialized".to_string(),
            Type::Undefined => "undefined".to_string(),
            Type::Null => "null".to_string(),
            Type::Bool => format!("{}", value.as_bool().unwrap()),
            Type::Int => format!("{}", value.as_int().unwrap()),
            Type::Float => format!("{}", value.as_float().unwrap()),
            Type::String => format!("\"{}\"", value.as_string().unwrap().to_string().unwrap()),
            Type::Symbol => "[Symbol]".to_string(),
            Type::Array => {
                let arr = value.as_array().unwrap();
                let mut elems = vec![];
                for i in 0..arr.len() {
                    let item = arr.get(i).unwrap();
                    elems.push(Self::print_value(ctx, item));
                }
                format!("Array({}) [ {} ]", arr.len(), elems.join(", "))
            }
            Type::Constructor => "[Constructor]".to_string(),
            Type::Function => "[Function]".to_string(),
            Type::Promise => "[Promise]".to_string(),
            Type::Exception => "[Exception]".to_string(),
            Type::Object => {
                let obj = value.as_object().unwrap();

                let global_prototype = ctx.globals().get_prototype().unwrap();
                let default_to_string = global_prototype.get::<_, Function>("toString").unwrap();

                if let Ok(to_string) = obj.get::<_, Function>("toString")
                    && to_string != default_to_string
                {
                    let mut args = Args::new(ctx.clone(), 0);
                    args.this(value.clone()).unwrap();

                    if let Ok(string_val) = to_string.call_arg::<String>(args) {
                        return string_val.to_string().unwrap();
                    }
                }

                let mut fields = vec![];
                for key in obj.keys::<String>() {
                    let key = key.unwrap();
                    let val: Value = obj.get(key.clone()).unwrap();
                    let val_str = Self::print_value(ctx, val);
                    fields.push(format!("{}: {}", key.to_string().unwrap(), val_str));
                }
                format!("Object {{ {} }}", fields.join(", "))
            }
            Type::Module => "[Module]".to_string(),
            Type::BigInt => {
                format!("{}", value.as_big_int().unwrap().clone().to_i64().unwrap())
            }
            Type::Unknown => "[Unknown]".to_string(),
        }
    }

    fn args_to_string<'js>(ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> String {
        args.0
            .into_iter()
            .map(|arg| Self::print_value(ctx, arg))
            .join(" ")
    }
}

const DEFAULT_NAME: &str = "default";

#[rquickjs::methods(rename_all = "camelCase")]
impl JsConsole {
    /// @rest
    pub fn print<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        print!("{}", Self::args_to_string(&ctx, args));
    }

    /// @rest
    pub fn print_ln<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        println!("{}", Self::args_to_string(&ctx, args));
    }

    /// @rest
    pub fn log<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        println!("{}", Self::args_to_string(&ctx, args));
    }

    /// @rest
    pub fn info<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        println!("{}", Self::args_to_string(&ctx, args));
    }

    /// @rest
    pub fn warn<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        let yellow = Style::new().yellow();
        println!("{}", yellow.apply_to(Self::args_to_string(&ctx, args)));
    }

    /// @rest
    pub fn error<'js>(&self, ctx: Ctx<'js>, args: Rest<Value<'js>>) {
        let red = Style::new().red().bold();
        println!("{}", red.apply_to(Self::args_to_string(&ctx, args)));
    }

    pub fn clear(&self) {
        let term = Term::stdout();
        term.clear_screen().unwrap();
    }

    pub fn time(&mut self, name: Opt<String>) {
        let name = name.clone().unwrap_or_else(|| DEFAULT_NAME.to_string());
        self.timers.insert(name, Instant::now());
    }

    pub fn time_end(&mut self, ctx: Ctx<'_>, name: Opt<String>) -> Result<()> {
        let name = name.clone().unwrap_or_else(|| DEFAULT_NAME.to_string());
        if let Some(timer_start) = self.timers.remove(&name) {
            println!(
                "{name}: {} - timer ended",
                format_duration(Instant::now() - timer_start)
            );
        } else {
            return Err(Exception::throw_message(
                &ctx,
                &format!("Timer \"{name}\" doesn't exist."),
            ));
        };

        Ok(())
    }

    pub fn count(&mut self, name: Opt<String>) {
        let name = name.clone().unwrap_or_else(|| DEFAULT_NAME.to_string());
        let value = self.counters.entry(name.clone()).or_default();
        *value += 1;
        println!("{name}: {value}");
    }
}

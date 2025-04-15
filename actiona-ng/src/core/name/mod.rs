use js::JsWildcard;
use rquickjs::{Ctx, Function, JsLifetime, Object, class::Trace, function::Args};

pub mod js;

#[derive(Clone, Debug, JsLifetime, PartialEq, Trace)]
pub enum Name<'js> {
    String(String),
    Wildcard(JsWildcard),
    Regex(Object<'js>),
}

impl<'js> Name<'js> {
    pub fn matches(&self, ctx: &Ctx<'js>, text: &str) -> bool {
        match self {
            Name::String(name) => name == text,
            Name::Wildcard(wildcard) => wildcard.inner().matches(text),
            Name::Regex(object) => {
                let test_function: Function = object.get("test").unwrap();
                let mut args = Args::new(ctx.clone(), 1);
                args.this(object.clone()).unwrap();
                args.push_arg(text).unwrap();
                test_function.call_arg(args).unwrap()
            }
        }
    }
}

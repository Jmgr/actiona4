use js::JsWildcard;
use rquickjs::{Ctx, Function, JsLifetime, Object, Result, class::Trace, function::Args};

pub mod js;

#[derive(Clone, Debug, JsLifetime, PartialEq, Trace)]
pub enum Name<'js> {
    String(String),
    Wildcard(JsWildcard),
    Regex(Object<'js>),
}

impl<'js> Name<'js> {
    pub fn matches(&self, ctx: &Ctx<'js>, text: &str) -> Result<bool> {
        Ok(match self {
            Name::String(name) => name == text,
            Name::Wildcard(wildcard) => wildcard.inner().matches(text),
            Name::Regex(object) => {
                // `Name::Regex` always holds a JS `RegExp`; call its standard `.test()` method.
                let test_function: Function = object.get("test")?;
                let mut args = Args::new(ctx.clone(), 1);
                args.this(object.clone())?;
                args.push_arg(text)?;
                test_function.call_arg(args)?
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Context, Object, Runtime};

    use super::Name;

    #[test]
    fn regex_match_errors_are_propagated() {
        let runtime = Runtime::new().expect("runtime");
        let context = Context::full(&runtime).expect("context");

        context.with(|ctx| {
            let regex: Object = ctx
                .eval(
                    r#"
                    (() => {
                        const re = /foo/;
                        re.test = () => { throw new Error("regex test boom"); };
                        return re;
                    })()
                    "#,
                )
                .expect("regex object");

            let name = Name::Regex(regex);
            let err = name.matches(&ctx, "foo").unwrap_err();
            assert!(matches!(err, rquickjs::Error::Exception));
        });
    }
}

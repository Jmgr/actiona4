use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    function::{Constructor, FromParam, ParamRequirement, ParamsAccessor},
    prelude::Rest,
};
use wildmatch::WildMatch;

use crate::core::{ResultExt, js::classes::ValueClass};

#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "Wildcard")]
pub struct JsWildcard {
    pattern: String,
    inner: WildMatch,
}

impl ValueClass<'_> for JsWildcard {}

impl<'js> Trace<'js> for JsWildcard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl PartialEq for JsWildcard {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl JsWildcard {
    /// @skip
    #[must_use]
    pub const fn inner(&self) -> &WildMatch {
        &self.inner
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWildcard {
    /// Constructor.
    ///
    /// @constructor
    #[qjs(constructor)]
    pub fn new(pattern: String) -> Result<Self> {
        Ok(Self {
            pattern: pattern.clone(),
            inner: WildMatch::new(&pattern),
        })
    }
}

#[derive(Clone, Debug, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Name")]
pub struct JsName<'js> {
    inner: super::Name<'js>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> JsName<'js> {
    // TODO: The first part before the first overload is ignored
    /// Creates a new Name.
    ///
    /// Example
    /// ```js
    /// let name = new Name("some name"); // Fixed name
    /// let name = new Name(new Wildcard("some *")); // Wildcard
    /// let name = new Name(/^\w+$/); // Regular expression
    /// ```
    ///
    /// @constructor
    ///
    /// @overload
    /// Constructor with a fixed name.
    /// @param name: string
    ///
    /// @overload
    /// Constructor with a wildcard.
    /// @param wildcard: Wildcard
    ///
    /// @overload
    /// Constructor with a regular expression.
    /// @param regexp: RegExp
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'js>, _args: Rest<Value<'js>>) -> Result<Self> {
        // TODO: accept an object as arg

        Ok(Self {
            inner: super::Name::String("TMP".into()), // TODO
        })
    }
}

impl<'js> ValueClass<'js> for JsName<'js> {}

pub struct JsNameParam<'js>(pub super::Name<'js>);

impl<'js> FromParam<'js> for JsNameParam<'js> {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::exhaustive()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        match params.len() {
            n if n >= 1 => {
                let value = params.arg();

                if value.is_string() {
                    return Ok(Self(super::Name::String(
                        value.as_string().unwrap().to_string().unwrap(),
                    )));
                }

                if let Ok(wildcard) = value.get::<JsWildcard>() {
                    return Ok(Self(super::Name::Wildcard(wildcard)));
                }

                let object = value
                    .into_object()
                    .or_throw_message(params.ctx(), "Expected an object")?;

                let regexp_ctor: Constructor = params.ctx().globals().get("RegExp")?;
                if object.is_instance_of(regexp_ctor) {
                    return Ok(Self(super::Name::Regex(object)));
                }

                Err(Exception::throw_message(
                    params.ctx(),
                    "Unexpected object type",
                ))
            }
            n => Err(Exception::throw_message(
                params.ctx(),
                &format!("Unexpected number of parameter: {n}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Ctx, JsLifetime, class::Trace};

    use super::{JsNameParam, JsWildcard};
    use crate::{core::js::classes::SingletonClass, runtime::Runtime};

    #[derive(Clone, Default, JsLifetime, Trace)]
    #[rquickjs::class(rename = "Test")]
    struct JsTest {}

    impl<'js> SingletonClass<'js> for JsTest {}

    #[rquickjs::methods(rename_all = "camelCase")]
    impl JsTest {
        pub fn name_match<'js>(&self, ctx: Ctx<'js>, name: JsNameParam<'js>, text: String) -> bool {
            name.0.matches(&ctx, &text)
        }
    }

    #[test]
    fn test_wildcard() {
        let wildcard = JsWildcard::new("foo*".to_string()).unwrap();
        assert!(wildcard.inner().matches("football"));
        assert!(!wildcard.inner().matches("cat"));

        let wildcard = JsWildcard::new("*😄*".to_string()).unwrap();
        assert!(wildcard.inner().matches("foot 😄 ball"));
        assert!(!wildcard.inner().matches("cat"));

        let wildcard = JsWildcard::new("😄?😁".to_string()).unwrap();
        assert!(!wildcard.inner().matches("😄😁"));
        assert!(wildcard.inner().matches("😄😆😁"));
        assert!(wildcard.inner().matches("😄-😁"));
    }

    #[test]
    fn test_name() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .with(|ctx| {
                    JsTest::register(&ctx, JsTest::default()).unwrap();
                })
                .await;

            assert!(
                script_engine
                    .eval::<bool>(r#"test.nameMatch("foo", "foo")"#)
                    .await
                    .unwrap()
            );
            assert!(
                !script_engine
                    .eval::<bool>(r#"test.nameMatch("foo", "bar")"#)
                    .await
                    .unwrap()
            );

            assert!(
                script_engine
                    .eval::<bool>(r#"test.nameMatch(new Wildcard("foo*"), "football")"#)
                    .await
                    .unwrap()
            );
            assert!(
                !script_engine
                    .eval::<bool>(r#"test.nameMatch(new Wildcard("foo"), "football")"#)
                    .await
                    .unwrap()
            );

            assert!(
                script_engine
                    .eval::<bool>(r#"test.nameMatch(/^\d+$/, "123456")"#)
                    .await
                    .unwrap()
            );
            assert!(
                !script_engine
                    .eval::<bool>(r#"test.nameMatch(/^\d+$/, "abc123def")"#)
                    .await
                    .unwrap()
            );
        });
    }
}

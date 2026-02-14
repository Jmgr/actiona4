//! @verbatim /**
//! @verbatim  * NameLike
//! @verbatim  */
//! @verbatim type NameLike = string | Wildcard | RegExp;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    function::{Constructor, FromParam, ParamRequirement, ParamsAccessor},
};
use wildmatch::WildMatch;

use crate::api::{ResultExt, js::classes::ValueClass};

/// A wildcard pattern for matching strings.
///
/// Supports `*` (match any sequence) and `?` (match any single character).
///
/// ```ts
/// const pattern = new Wildcard("*.txt");
/// ```
///
/// ```ts
/// // Used in APIs that accept a NameLike parameter
/// const pattern = new Wildcard("my_app*");
/// ```
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

/*
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
*/

pub struct JsNameLike<'js>(pub super::Name<'js>);

#[derive(Clone, Debug, JsLifetime)]
pub struct JsName<'js>(pub super::Name<'js>);

impl<'js> JsName<'js> {
    #[must_use]
    pub const fn inner(&self) -> &super::Name<'js> {
        &self.0
    }
}

pub(crate) fn value_to_name_like<'js>(
    ctx: &Ctx<'js>,
    value: Value<'js>,
) -> Result<super::Name<'js>> {
    if value.is_string() {
        return Ok(super::Name::String(
            value.as_string().unwrap().to_string().unwrap(),
        ));
    }

    if let Ok(wildcard) = value.get::<JsWildcard>() {
        return Ok(super::Name::Wildcard(wildcard));
    }

    let object = value
        .into_object()
        .or_throw_message(ctx, "Expected an object")?;

    let regexp_ctor: Constructor = ctx.globals().get("RegExp")?;
    if object.is_instance_of(regexp_ctor) {
        return Ok(super::Name::Regex(object));
    }

    Err(Exception::throw_message(ctx, "Unexpected object type"))
}

impl<'js> rquickjs::FromJs<'js> for JsName<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Self(value_to_name_like(ctx, value)?))
    }
}

impl<'js> FromParam<'js> for JsNameLike<'js> {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single().combine(ParamRequirement::exhaustive()) // 1 -> 1
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let ctx = params.ctx().clone();
        let value = params.arg();
        Ok(Self(value_to_name_like(&ctx, value)?))
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Ctx, JsLifetime, class::Trace};

    use super::{JsNameLike, JsWildcard};
    use crate::{
        api::js::classes::{SingletonClass, register_singleton_class},
        runtime::Runtime,
    };

    #[derive(Clone, Default, JsLifetime, Trace)]
    #[rquickjs::class(rename = "Test")]
    struct JsTest {}

    impl<'js> SingletonClass<'js> for JsTest {}

    #[rquickjs::methods(rename_all = "camelCase")]
    impl JsTest {
        pub fn name_match<'js>(
            &self,
            ctx: Ctx<'js>,
            name: JsNameLike<'js>,
            text: String,
        ) -> rquickjs::Result<bool> {
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
                    register_singleton_class::<JsTest>(&ctx, JsTest::default())?;
                    Ok(())
                })
                .await
                .unwrap();

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

            let error = script_engine
                .eval::<bool>(
                    r#"
                    const re = /foo/;
                    re.test = () => { throw new Error("regex test boom"); };
                    test.nameMatch(re, "foo");
                "#,
                )
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains("regex test boom"));
        });
    }
}

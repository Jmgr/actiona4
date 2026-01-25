//! @verbatim /**
//! @verbatim  * NameLike
//! @verbatim  */
//! @verbatim type NameLike = string | Wildcard | RegExp;

use rquickjs::{
    Exception, JsLifetime, Result,
    class::{Trace, Tracer},
    function::{Constructor, FromParam, ParamRequirement, ParamsAccessor},
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

/*
// TODO
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

impl<'js> FromParam<'js> for JsNameLike<'js> {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single().combine(ParamRequirement::exhaustive()) // 1 -> 1
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
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
}

#[cfg(test)]
mod tests {
    use rquickjs::{Ctx, JsLifetime, class::Trace};

    use super::{JsNameLike, JsWildcard};
    use crate::{
        core::js::classes::{SingletonClass, register_singleton_class},
        runtime::Runtime,
    };

    #[derive(Clone, Default, JsLifetime, Trace)]
    #[rquickjs::class(rename = "Test")]
    struct JsTest {}

    impl<'js> SingletonClass<'js> for JsTest {}

    #[rquickjs::methods(rename_all = "camelCase")]
    impl JsTest {
        pub fn name_match<'js>(&self, ctx: Ctx<'js>, name: JsNameLike<'js>, text: String) -> bool {
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
                    register_singleton_class::<JsTest>(&ctx, JsTest::default()).unwrap();
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

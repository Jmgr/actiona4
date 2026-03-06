use macros::{js_class, js_methods};
use rquickjs::{
    JsLifetime,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};

use crate::{
    api::{js::classes::SingletonClass, standardpaths::StandardPaths},
    types::display::display_with_type,
};
/// Platform-specific standard directory paths.
///
/// All properties return the path as a string, or undefined if unavailable.
///
/// ```ts
/// println(standardPaths.home);       // e.g. "/home/user"
/// println(standardPaths.downloads);   // e.g. "/home/user/Downloads"
/// println(standardPaths.documents);   // e.g. "/home/user/Documents"
/// ```
///
/// @category StandardPaths
/// @singleton
#[derive(Clone, Debug, Default, JsLifetime)]
#[js_class]
pub struct JsStandardPaths {
    inner: StandardPaths,
}

impl<'js> Trace<'js> for JsStandardPaths {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsStandardPaths {}

#[js_methods]
impl JsStandardPaths {
    /// Home directory
    #[get]
    #[must_use]
    pub fn home(&self) -> Option<String> {
        self.inner.home().as_ref().map(|path| path.to_string())
    }

    /// Temporary directory
    #[get]
    #[must_use]
    pub fn temp(&self) -> Option<String> {
        self.inner.temp().as_ref().map(|path| path.to_string())
    }

    /// Music directory
    #[get]
    #[must_use]
    pub fn music(&self) -> Option<String> {
        self.inner.music().as_ref().map(|path| path.to_string())
    }

    /// Desktop directory
    #[get]
    #[must_use]
    pub fn desktop(&self) -> Option<String> {
        self.inner.desktop().as_ref().map(|path| path.to_string())
    }

    /// Documents directory
    #[get]
    #[must_use]
    pub fn documents(&self) -> Option<String> {
        self.inner.documents().as_ref().map(|path| path.to_string())
    }

    /// Downloads directory
    #[get]
    #[must_use]
    pub fn downloads(&self) -> Option<String> {
        self.inner.downloads().as_ref().map(|path| path.to_string())
    }

    /// Pictures directory
    #[get]
    #[must_use]
    pub fn pictures(&self) -> Option<String> {
        self.inner.pictures().as_ref().map(|path| path.to_string())
    }

    /// Public directory
    #[get]
    #[must_use]
    pub fn public(&self) -> Option<String> {
        self.inner.public().as_ref().map(|path| path.to_string())
    }

    /// Videos directory
    #[get]
    #[must_use]
    pub fn videos(&self) -> Option<String> {
        self.inner.videos().as_ref().map(|path| path.to_string())
    }

    /// Cache directory
    #[get]
    #[must_use]
    pub fn cache(&self) -> Option<String> {
        self.inner.cache().as_ref().map(|path| path.to_string())
    }

    /// Config directory
    #[get]
    #[must_use]
    pub fn config(&self) -> Option<String> {
        self.inner.config().as_ref().map(|path| path.to_string())
    }

    /// Local config directory
    #[get]
    #[must_use]
    pub fn local_config(&self) -> Option<String> {
        self.inner
            .local_config()
            .as_ref()
            .map(|path| path.to_string())
    }

    /// Returns a string representation of all standard paths.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("StandardPaths", &self.inner)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    #[ignore]
    fn test_standard_paths() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>("console.println(standardPaths);")
                .await
                .unwrap();
        });
    }
}

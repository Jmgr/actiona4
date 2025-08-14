use convert_case::{Case, Casing};
use rquickjs::{Class, Ctx, IntoJs, Object, class::JsClass};

/// Represents a JavaScript class that exists as a single instance in the global scope.
///
/// The `'js` lifetime represents the lifetime of the JavaScript context.
pub trait SingletonClass<'js>: JsClass<'js> + IntoJs<'js> {
    /// Register any dependencies required by this class.
    ///
    /// This is called before the singleton instance is registered.
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let _ = ctx; // Silence unused variable warning
        Ok(())
    }

    /// Perform extra registration steps after the singleton has been registered.
    ///
    /// This allows for additional configuration of the object in the JavaScript context.
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        let _ = object; // Silence unused variable warning
        Ok(())
    }

    /// Register this singleton instance in the JavaScript context.
    ///
    /// This creates a global variable with the snake_case version of the class name
    /// and assigns the instance to it.
    fn register(ctx: &Ctx<'js>, instance: Self) -> rquickjs::Result<()>
    where
        Self: Sized,
    {
        Self::register_dependencies(ctx)?;

        // Remove "Js" prefix if present
        let name = Self::NAME.strip_prefix("Js").unwrap_or(Self::NAME);

        let name = name.to_case(Case::Snake);

        ctx.globals().prop(&name, instance)?;

        let object = ctx.globals().get::<_, Object>(name)?;

        Self::extra_registration(&object)?;

        Ok(())
    }
}

/// Represents a JavaScript class that can be instantiated multiple times.
///
/// The `'js` lifetime represents the lifetime of the JavaScript context.
pub trait ValueClass<'js>: JsClass<'js> {
    /// Register any dependencies required by this class.
    ///
    /// This is called before the class is defined in the JavaScript context.
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let _ = ctx; // Silence unused variable warning
        Ok(())
    }

    /// Perform extra registration steps after the class has been defined.
    ///
    /// This allows for additional configuration of the class in the JavaScript context.
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        let _ = object; // Silence unused variable warning
        Ok(())
    }

    /// Register this class in the JavaScript context.
    ///
    /// This defines the class in the global scope, making it available for instantiation.
    fn register(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Self::register_dependencies(ctx)?;

        Class::<Self>::define(&ctx.globals())?;

        // Remove "Js" prefix if present
        let name = Self::NAME.strip_prefix("Js").unwrap_or(Self::NAME);

        let object = ctx.globals().get::<_, Object>(name)?;

        Self::extra_registration(&object)?;

        Ok(())
    }
}

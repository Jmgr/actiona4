use color_eyre::eyre::Context;
use convert_case::{Case, Casing};
use rquickjs::{Class, Ctx, IntoJs, Object, class::JsClass};
use serde::{Deserialize, Serialize};
use serde_name::trace_name;
use strum::IntoEnumIterator;

use crate::IntoJsResult;

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
}

/// Register this singleton instance in the JavaScript context.
///
/// This creates a global variable with the snake_case version of the class name
/// and assigns the instance to it.
/// @skip
pub fn register_singleton_class<'js, T: SingletonClass<'js> + JsClass<'js> + Sized>(
    ctx: &Ctx<'js>,
    instance: T,
) -> rquickjs::Result<()> {
    T::register_dependencies(ctx)?;

    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    let name = name.to_case(Case::Camel);

    ctx.globals().prop(&name, instance)?;

    let object = ctx.globals().get::<_, Object>(name)?;

    T::extra_registration(&object)?;

    Ok(())
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
}

/// Register this class in the JavaScript context.
///
/// This defines the class in the global scope, making it available for instantiation.
/// @skip
pub fn register_value_class<'js, T: ValueClass<'js> + JsClass<'js>>(
    ctx: &Ctx<'js>,
) -> rquickjs::Result<()> {
    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    move || -> rquickjs::Result<()> {
        T::register_dependencies(ctx)
            .wrap_err("register dependencies")
            .into_js_result(ctx)?;

        Class::<T>::define(&ctx.globals())
            .wrap_err("define constructor")
            .into_js_result(ctx)?;

        let object = ctx.globals().get::<_, Object>(name)?;

        T::extra_registration(&object)
            .wrap_err("extra registration")
            .into_js_result(ctx)?;

        Ok(())
    }()
    .wrap_err_with(|| format!("registering {name} (missing constructor?)"))
    .into_js_result(ctx)
}

/// Represents a JavaScript class that cannot be created by the user.
///
/// The `'js` lifetime represents the lifetime of the JavaScript context.
pub trait HostClass<'js>: JsClass<'js> + IntoJs<'js> {
    /// Register any dependencies required by this class.
    ///
    /// This is called before the class is defined in the JavaScript context.
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let _ = ctx; // Silence unused variable warning
        Ok(())
    }
}

/// Register this class in the JavaScript context.
///
/// This defines the class in the global scope, making it available for instantiation.
/// @skip
pub fn register_host_class<'js, T: HostClass<'js> + JsClass<'js>>(
    ctx: &Ctx<'js>,
) -> rquickjs::Result<()> {
    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    move || -> rquickjs::Result<()> {
        T::register_dependencies(ctx)
            .wrap_err("register dependencies")
            .into_js_result(ctx)?;

        Class::<T>::define(&ctx.globals())
            .wrap_err("define constructor")
            .into_js_result(ctx)?;

        Ok(())
    }()
    .wrap_err_with(|| format!("registering {name}"))
    .into_js_result(ctx)
}

/// @skip
pub fn register_enum<'js, E>(ctx: &Ctx<'js>) -> rquickjs::Result<()>
where
    E: Serialize + for<'de> Deserialize<'de> + IntoEnumIterator,
{
    let obj = Object::new(ctx.clone())?;
    for v in E::iter() {
        // Serialize variant to serde's canonical string (honors rename/rename_all)
        let key = serde_plain::to_string(&v).unwrap();

        // Set both the property name and the value to that canonical string
        obj.set(&key, key.clone())?;
    }
    ctx.globals().set(trace_name::<E>().unwrap(), obj)
}

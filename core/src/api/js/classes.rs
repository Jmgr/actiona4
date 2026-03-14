use color_eyre::eyre::Context;
use convert_case::{Case, Casing};
use rquickjs::{Class, Ctx, Exception, IntoJs, Object, class::JsClass};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tracing::instrument;

use crate::{IntoJsResult, runtime::WithUserData};

/// Returns the object where Actiona API items should be registered.
///
/// In normal mode this is `ctx.globals()`. In `no_globals` mode this is
/// the `actiona` namespace object stored on globals.
/// @skip
#[must_use]
pub fn registration_target<'js>(ctx: &Ctx<'js>) -> Object<'js> {
    if ctx.user_data().no_globals() {
        ctx.globals()
            .get::<_, Object>("actiona")
            .expect("actiona namespace should exist")
    } else {
        ctx.globals()
    }
}

/// Represents a JavaScript class that exists as a single instance in the global scope.
///
/// The `'js` lifetime represents the lifetime of the JavaScript context.
pub trait SingletonClass<'js>: JsClass<'js> + IntoJs<'js> {
    /// Register any dependencies required by this class.
    ///
    /// This is called before the singleton instance is registered.
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        _ = ctx; // Silence unused variable warning
        Ok(())
    }

    /// Perform extra registration steps after the singleton has been registered.
    ///
    /// This allows for additional configuration of the object in the JavaScript context.
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        _ = object; // Silence unused variable warning
        Ok(())
    }
}

/// Register this singleton instance in the JavaScript context.
///
/// This creates a global variable with the snake_case version of the class name
/// and assigns the instance to it.
/// @skip
#[instrument(skip_all)]
pub fn register_singleton_class<'js, T: SingletonClass<'js> + JsClass<'js> + Sized>(
    ctx: &Ctx<'js>,
    instance: T,
) -> rquickjs::Result<()> {
    T::register_dependencies(ctx)?;

    let target = registration_target(ctx);

    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    let name = name.to_case(Case::Camel);

    target.prop(&name, instance)?;

    let object = target.get::<_, Object>(name)?;

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
        _ = ctx; // Silence unused variable warning
        Ok(())
    }

    /// Perform extra registration steps after the class has been defined.
    ///
    /// This allows for additional configuration of the class in the JavaScript context.
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        _ = object; // Silence unused variable warning
        Ok(())
    }
}

/// Register this class in the JavaScript context.
///
/// This defines the class in the global scope, making it available for instantiation.
/// @skip
#[instrument(skip_all)]
pub fn register_value_class<'js, T: ValueClass<'js> + JsClass<'js>>(
    ctx: &Ctx<'js>,
) -> rquickjs::Result<()> {
    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    move || -> rquickjs::Result<()> {
        T::register_dependencies(ctx)
            .wrap_err("register dependencies")
            .into_js_result(ctx)?;

        let target = registration_target(ctx);

        Class::<T>::define(&target)
            .wrap_err("define constructor")
            .into_js_result(ctx)?;

        let object = target.get::<_, Object>(name)?;

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
        _ = ctx; // Silence unused variable warning
        Ok(())
    }
}

/// Register this class in the JavaScript context.
///
/// This defines the class in the global scope, making it available for instantiation.
/// @skip
#[instrument(skip_all)]
pub fn register_host_class<'js, T: HostClass<'js> + JsClass<'js>>(
    ctx: &Ctx<'js>,
) -> rquickjs::Result<()> {
    // Remove "Js" prefix if present
    let name = T::NAME.strip_prefix("Js").unwrap_or(T::NAME);

    move || -> rquickjs::Result<()> {
        T::register_dependencies(ctx)
            .wrap_err("register dependencies")
            .into_js_result(ctx)?;

        let target = registration_target(ctx);

        Class::<T>::define(&target)
            .wrap_err("define constructor")
            .into_js_result(ctx)?;

        Ok(())
    }()
    .wrap_err_with(|| format!("registering {name}"))
    .into_js_result(ctx)
}

/// @skip
#[instrument(skip_all)]
pub fn register_enum<'js, E>(ctx: &Ctx<'js>) -> rquickjs::Result<()>
where
    E: Serialize + for<'de> Deserialize<'de> + IntoEnumIterator,
{
    let target = registration_target(ctx);
    let obj = Object::new(ctx.clone())?;
    for v in E::iter() {
        // Serialize variant to serde's canonical string (honors rename/rename_all)
        let key = serde_plain::to_string(&v).map_err(|err| {
            Exception::throw_message(ctx, &format!("Failed to serialize enum variant: {err}"))
        })?;

        // Set both the property name and the value to that canonical string
        obj.set(&key, key.clone())?;
    }

    target.set(enum_registration_name::<E>(), obj)
}

fn enum_registration_name<E>() -> String {
    let type_name = std::any::type_name::<E>();
    let enum_name = type_name
        .rsplit("::")
        .next()
        .expect("type_name should always contain a final segment");

    enum_name
        .strip_prefix("Js")
        .unwrap_or(enum_name)
        .to_string()
}

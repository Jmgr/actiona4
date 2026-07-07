use std::collections::HashSet;

use rquickjs::{Array, Class, Function, Object, Value, class::JsClass};

use crate::api::{
    color::js::JsColor,
    image::js::{JsImage, JsMatch},
    name::js::JsWildcard,
    point::js::JsPoint,
    rect::js::JsRect,
    size::js::JsSize,
};

/// @skip
#[derive(Debug, thiserror::Error)]
pub enum DeepEqualError {
    #[error("cannot compare non-finite number {value}")]
    NonFiniteNumber { value: &'static str },
    #[error("failed to inspect JavaScript value: {0}")]
    Js(String),
}

impl From<rquickjs::Error> for DeepEqualError {
    fn from(value: rquickjs::Error) -> Self {
        Self::Js(value.to_string())
    }
}

pub trait DeepEqualClass {
    fn deep_equal_class(&self, other: &Self) -> Result<bool, DeepEqualError>;
}

pub fn deep_equal<'js>(left: &Value<'js>, right: &Value<'js>) -> Result<bool, DeepEqualError> {
    let mut validation_seen = HashSet::new();
    validate_no_non_finite_numbers(left, &mut validation_seen)?;
    validate_no_non_finite_numbers(right, &mut validation_seen)?;

    let mut seen = HashSet::new();

    deep_equal_inner(left, right, &mut seen)
}

fn validate_no_non_finite_numbers<'js>(
    value: &Value<'js>,
    seen: &mut HashSet<Value<'js>>,
) -> Result<(), DeepEqualError> {
    validate_finite_number(value)?;

    let Some(object) = value.as_object() else {
        return Ok(());
    };

    if !seen.insert(value.clone()) {
        return Ok(());
    }

    for key in own_enumerable_keys(object)? {
        let child = object.get::<_, Value>(key.as_str())?;
        validate_no_non_finite_numbers(&child, seen)?;
    }

    Ok(())
}

fn deep_equal_inner<'js>(
    left: &Value<'js>,
    right: &Value<'js>,
    seen: &mut HashSet<(Value<'js>, Value<'js>)>,
) -> Result<bool, DeepEqualError> {
    validate_finite_number(left)?;
    validate_finite_number(right)?;

    if let Some(result) = known_actiona_value_equal(left, right)? {
        return Ok(result);
    }

    match (left.as_array(), right.as_array()) {
        (Some(left), Some(right)) => return arrays_equal(left, right, seen),
        (Some(_), None) | (None, Some(_)) => return Ok(false),
        (None, None) => {}
    }

    match (left.as_object(), right.as_object()) {
        (Some(left), Some(right)) if is_plain_object(left)? && is_plain_object(right)? => {
            objects_equal(left, right, seen)
        }
        _ => strict_equal(left, right),
    }
}

fn known_actiona_value_equal<'js>(
    left: &Value<'js>,
    right: &Value<'js>,
) -> Result<Option<bool>, DeepEqualError> {
    macro_rules! try_classes {
        ($($class:ty),* $(,)?) => {
            $(
                if let Some(result) = try_deep_equal_class::<$class>(left, right)? {
                    return Ok(Some(result));
                }
            )*
        };
    }

    try_classes!(
        JsPoint, JsSize, JsRect, JsColor, JsWildcard, JsMatch, JsImage,
    );

    Ok(None)
}

fn try_deep_equal_class<'js, T>(
    left: &Value<'js>,
    right: &Value<'js>,
) -> Result<Option<bool>, DeepEqualError>
where
    T: JsClass<'js> + DeepEqualClass,
{
    let left = value_as_class::<T>(left);
    let right = value_as_class::<T>(right);

    match (left, right) {
        (Some(left), Some(right)) => {
            let left = left.try_borrow()?;
            let right = right.try_borrow()?;

            left.deep_equal_class(&right).map(Some)
        }
        (Some(_), None) | (None, Some(_)) => Ok(Some(false)),
        (None, None) => Ok(None),
    }
}

fn value_as_class<'a, 'js, T>(value: &'a Value<'js>) -> Option<&'a Class<'js, T>>
where
    T: JsClass<'js>,
{
    value.as_object()?.as_class::<T>()
}

fn arrays_equal<'js>(
    left: &Array<'js>,
    right: &Array<'js>,
    seen: &mut HashSet<(Value<'js>, Value<'js>)>,
) -> Result<bool, DeepEqualError> {
    if left.len() != right.len() {
        return Ok(false);
    }

    objects_equal(left.as_object(), right.as_object(), seen)
}

fn objects_equal<'js>(
    left: &Object<'js>,
    right: &Object<'js>,
    seen: &mut HashSet<(Value<'js>, Value<'js>)>,
) -> Result<bool, DeepEqualError> {
    let pair = (left.as_value().clone(), right.as_value().clone());
    if !seen.insert(pair) {
        return Ok(true);
    }

    let left_keys = own_enumerable_keys(left)?;
    let right_keys = own_enumerable_keys(right)?;

    if left_keys.len() != right_keys.len() {
        return Ok(false);
    }

    for key in &left_keys {
        if !right_keys.contains(key) {
            return Ok(false);
        }

        let left_value = left.get::<_, Value>(key.as_str())?;
        let right_value = right.get::<_, Value>(key.as_str())?;

        if !deep_equal_inner(&left_value, &right_value, seen)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn own_enumerable_keys(object: &Object<'_>) -> Result<Vec<String>, DeepEqualError> {
    object
        .keys::<String>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn is_plain_object(object: &Object<'_>) -> Result<bool, DeepEqualError> {
    let Some(prototype) = object.get_prototype() else {
        return Ok(true);
    };

    let object_prototype = object.ctx().eval::<Value, _>("Object.prototype")?;

    strict_equal(prototype.as_value(), &object_prototype)
}

fn strict_equal<'js>(left: &Value<'js>, right: &Value<'js>) -> Result<bool, DeepEqualError> {
    let function = left
        .ctx()
        .eval::<Function, _>("(left, right) => left === right")?;

    function
        .call((left.clone(), right.clone()))
        .map_err(Into::into)
}

fn validate_finite_number(value: &Value<'_>) -> Result<(), DeepEqualError> {
    let Some(number) = value.as_number() else {
        return Ok(());
    };

    validate_finite_f64(number)
}

const fn validate_finite_f64(value: f64) -> Result<(), DeepEqualError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(DeepEqualError::NonFiniteNumber {
            value: non_finite_number_name(value),
        })
    }
}

const fn non_finite_number_name(value: f64) -> &'static str {
    if value.is_nan() {
        "NaN"
    } else if value.is_sign_positive() {
        "Infinity"
    } else {
        "-Infinity"
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Array, Context, Runtime, Value};

    use super::{DeepEqualError, deep_equal};
    use crate::api::{
        color::js::JsColor, image::js::JsImage, js::classes::register_value_class,
        name::js::JsWildcard, point::js::JsPoint, rect::js::JsRect, size::js::JsSize,
    };

    fn compare_pair(script: &str) -> Result<bool, DeepEqualError> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;

        context.with(|ctx| {
            register_value_class::<JsPoint>(&ctx)?;
            register_value_class::<JsSize>(&ctx)?;
            register_value_class::<JsRect>(&ctx)?;
            register_value_class::<JsColor>(&ctx)?;
            register_value_class::<JsWildcard>(&ctx)?;
            register_value_class::<JsImage>(&ctx)?;

            let pair = ctx.eval::<Array, _>(script)?;
            let left = pair.get::<Value>(0)?;
            let right = pair.get::<Value>(1)?;

            deep_equal(&left, &right)
        })
    }

    #[test]
    fn compares_primitives_by_strict_equality() {
        assert!(compare_pair("[1, 1.0]").unwrap());
        assert!(compare_pair("['text', 'text']").unwrap());
        assert!(compare_pair("[-0, 0]").unwrap());
        assert!(!compare_pair("[1, '1']").unwrap());
    }

    #[test]
    fn compares_arrays_and_plain_objects_by_structure() {
        assert!(
            compare_pair("[{ a: [1, { b: true }], c: 'x' }, { c: 'x', a: [1, { b: true }] }]")
                .unwrap()
        );
        assert!(!compare_pair("[{ a: [1, 2] }, { a: [1, 3] }]").unwrap());
    }

    #[test]
    fn compares_known_actiona_value_classes_by_value() {
        for script in [
            "[new Point(1, 2), new Point(1, 2)]",
            "[new Size(3, 4), new Size(3, 4)]",
            "[new Rect(1, 2, 3, 4), new Rect(1, 2, 3, 4)]",
            "[new Color(1, 2, 3, 4), new Color(1, 2, 3, 4)]",
            "[new Wildcard('app*'), new Wildcard('app*')]",
            "[new Image(2, 2), new Image(2, 2)]",
        ] {
            assert!(compare_pair(script).unwrap(), "{script}");
        }

        assert!(!compare_pair("[new Point(1, 2), new Point(2, 1)]").unwrap());
    }

    #[test]
    fn known_actiona_value_classes_do_not_cross_match() {
        assert!(!compare_pair("[new Point(1, 2), new Size(1, 2)]").unwrap());
    }

    #[test]
    fn does_not_call_arbitrary_class_equals_method() {
        assert!(
            !compare_pair(
                "(() => {
                    class AlwaysEqual {
                        equals() {
                            throw new Error('equals should not be called');
                        }
                    }

                    return [new AlwaysEqual(), new AlwaysEqual()];
                })()"
            )
            .unwrap()
        );
    }

    #[test]
    fn distinguishes_sparse_arrays_from_undefined_elements() {
        assert!(!compare_pair("[Array(1), [undefined]]").unwrap());
    }

    #[test]
    fn compares_null_prototype_objects_by_structure() {
        assert!(
            compare_pair(
                "(() => {
                    const left = Object.create(null);
                    const right = Object.create(null);
                    left.value = [1, 2];
                    right.value = [1, 2];
                    return [left, right];
                })()"
            )
            .unwrap()
        );
    }

    #[test]
    fn keeps_non_plain_objects_on_identity_equality() {
        assert!(!compare_pair("[new Date(0), new Date(0)]").unwrap());
    }

    #[test]
    fn handles_cycles() {
        assert!(
            compare_pair(
                "(() => {
                    const left = [];
                    const right = [];
                    left.push(left);
                    right.push(right);
                    return [left, right];
                })()"
            )
            .unwrap()
        );
    }

    #[test]
    fn rejects_non_finite_numbers() {
        for script in [
            "[NaN, NaN]",
            "[Infinity, Infinity]",
            "[{ value: -Infinity }, {}]",
            "[new (class Box { constructor() { this.value = Infinity; } })(), {}]",
        ] {
            assert!(matches!(
                compare_pair(script),
                Err(DeepEqualError::NonFiniteNumber { .. })
            ));
        }
    }
}

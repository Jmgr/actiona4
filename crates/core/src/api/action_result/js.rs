use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};

use crate::api::{
    action_result::{ActionBranch, ActionResult},
    js::classes::HostClass,
};

/// Branch target used by `ActionResult.branch`.
///
/// ```ts
/// ActionResult.branch(ActionBranch.yes());
/// ActionResult.branch(ActionBranch.custom("retry"));
/// ```
/// @category Actions
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsActionBranch {
    inner: ActionBranch,
}

impl<'js> Trace<'js> for JsActionBranch {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsActionBranch {}

impl JsActionBranch {
    /// @skip
    #[must_use]
    pub fn into_inner(self) -> ActionBranch {
        self.inner
    }
}

#[js_methods]
impl JsActionBranch {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "ActionBranch cannot be instantiated directly",
        ))
    }

    /// Selects the `yes` branch.
    #[qjs(static)]
    #[must_use]
    pub const fn yes() -> Self {
        Self {
            inner: ActionBranch::Yes,
        }
    }

    /// Selects the `no` branch.
    #[qjs(static)]
    #[must_use]
    pub const fn no() -> Self {
        Self {
            inner: ActionBranch::No,
        }
    }

    /// Selects the `cancel` branch.
    #[qjs(static)]
    #[must_use]
    pub const fn cancel() -> Self {
        Self {
            inner: ActionBranch::Cancel,
        }
    }

    /// Selects the `true` branch.
    #[qjs(rename = "true")]
    #[qjs(static)]
    #[must_use]
    pub const fn true_() -> Self {
        Self {
            inner: ActionBranch::True,
        }
    }

    /// Selects the `false` branch.
    #[qjs(rename = "false")]
    #[qjs(static)]
    #[must_use]
    pub const fn false_() -> Self {
        Self {
            inner: ActionBranch::False,
        }
    }

    /// Selects a custom named branch.
    #[qjs(static)]
    #[must_use]
    pub const fn custom(name: String) -> Self {
        Self {
            inner: ActionBranch::Custom(name),
        }
    }

    /// Returns a string representation of this action branch.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!("ActionBranch({:?})", self.inner)
    }
}

/// Execution control result for Code actions.
///
/// Evaluate to one of these values from a Code action script to control which
/// action runs next. Evaluating to nothing continues with the next sibling
/// action.
///
/// ```ts
/// const result = shouldStop
///   ? ActionResult.stop()
///   : needsRetry
///     ? ActionResult.gotoLabel("retry")
///     : ActionResult.branch(ActionBranch.true());
///
/// result;
/// ```
/// @category Actions
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsActionResult {
    inner: ActionResult,
}

impl<'js> Trace<'js> for JsActionResult {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsActionResult {}

impl JsActionResult {
    /// @skip
    #[must_use]
    pub fn into_inner(self) -> ActionResult {
        self.inner
    }
}

#[js_methods]
impl JsActionResult {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "ActionResult cannot be instantiated directly",
        ))
    }

    /// Continues execution with the next sibling action.
    #[qjs(static)]
    #[must_use]
    pub const fn next_sibling() -> Self {
        Self {
            inner: ActionResult::NextSibling,
        }
    }

    /// Continues execution with the first child action.
    #[qjs(static)]
    #[must_use]
    pub const fn next_child() -> Self {
        Self {
            inner: ActionResult::NextChild,
        }
    }

    /// Continues execution with the matching branch.
    #[qjs(static)]
    #[must_use]
    pub fn branch(branch: JsActionBranch) -> Self {
        Self {
            inner: ActionResult::Branch(branch.into_inner()),
        }
    }

    /// Jumps to the action with the given label.
    #[qjs(static)]
    #[must_use]
    pub const fn goto_label(label: String) -> Self {
        Self {
            inner: ActionResult::GotoLabel(label),
        }
    }

    /// Stops action execution.
    #[qjs(static)]
    #[must_use]
    pub const fn stop() -> Self {
        Self {
            inner: ActionResult::Stop,
        }
    }

    /// Returns a string representation of this action result.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!("ActionResult({:?})", self.inner)
    }
}

use std::ops::{Deref, DerefMut};

use icons::common::IconType;
use macros::{ActionDefinitions, common_parameters};
use serde::{Deserialize, Serialize};
use types::platform::Platforms;

use crate::{
    TranslationKey,
    parameters::{Param, Parameter, ParameterKind, duration::DurationValue},
    scriptable::Scriptable,
    tree::BranchKind,
};

pub mod clipboard;
pub mod flow;
pub mod misc;
pub mod mouse;
pub mod system;
pub mod window;

pub use clipboard::*;
pub use flow::*;
pub use macros::action;
pub use misc::*;
pub use mouse::*;
pub use system::*;
pub use window::*;

#[common_parameters]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CommonParameters {
    #[parameter(translation = "action-timeout")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<DurationValue>,

    #[parameter(translation = "action-pause-before")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_before: Option<DurationValue>,

    #[parameter(translation = "action-pause-after")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_after: Option<DurationValue>,
}

impl CommonParameters {
    pub fn is_empty(&self) -> bool {
        self.timeout.is_none() && self.pause_before.is_none() && self.pause_after.is_none()
    }
}

#[static_dispatch::setup]
pub trait WithDefinition {
    fn definition(&self) -> &'static ActionDefinition;
}

#[static_dispatch::setup]
pub trait WithCommonParameters {
    fn timeout(&self) -> &Option<DurationValue>;
    fn pause_before(&self) -> &Option<DurationValue>;
    fn pause_after(&self) -> &Option<DurationValue>;

    fn set_timeout(&mut self, timeout: Option<DurationValue>);
    fn set_pause_before(&mut self, pause_before: Option<DurationValue>);
    fn set_pause_after(&mut self, pause_after: Option<DurationValue>);
}

#[static_dispatch::setup]
pub trait ActionBranches {
    fn action_branches(&self) -> Vec<BranchKind> {
        Vec::new()
    }
}

/// Lets an action hide/disable some of its own parameters depending on the
/// current value of another parameter (e.g. a "mode" enum that only makes
/// `start`/`end` relevant in one mode and `time` in another).
#[static_dispatch::setup]
pub trait ParameterAvailability {
    /// Ids (matching [`Parameter::id`]) of parameters that are currently
    /// disabled given the action's other parameter values. Empty means every
    /// parameter is available.
    fn disabled_parameters(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Ids of the parameters that `disabled_parameters` reads. A reactive UI
    /// can use this to scope its recompute (e.g. a memo) to just these
    /// parameters instead of the action's full parameter set. Empty means
    /// `disabled_parameters` never returns anything but the default.
    fn watched_parameters(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

pub trait Branching: ActionBranches + WithDefinition + WithCommonParameters {
    fn branches(&self) -> Vec<BranchKind> {
        let mut branches = self.action_branches();

        if self.definition().supports_timeout && self.timeout().is_some() {
            branches.push(BranchKind::Timeout);
        }

        branches
    }
}

impl<T> Branching for T where T: ActionBranches + WithDefinition + WithCommonParameters {}

/// Pairs an action instance with the [`CommonParameters`] every action carries.
/// Each [`ActionInstance`] variant holds `WithCommon<T>`, so common parameters
/// live in one place rather than being injected into every action struct. The
/// blanket impls below forward the common-parameter traits to the wrapper and
/// the action-specific traits to the inner action, and `Deref` forwards field
/// access, so `WithCommon<Click>` behaves like a `Click` that also has common
/// parameters.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WithCommon<T> {
    #[serde(flatten, skip_serializing_if = "CommonParameters::is_empty")]
    pub common: CommonParameters,

    #[serde(flatten)]
    pub action: T,
}

impl<T> WithCommon<T> {
    pub fn new(action: T) -> Self {
        Self {
            common: CommonParameters::default(),
            action,
        }
    }
}

impl<T> From<T> for WithCommon<T> {
    fn from(action: T) -> Self {
        Self::new(action)
    }
}

impl<T> Deref for WithCommon<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.action
    }
}

impl<T> DerefMut for WithCommon<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.action
    }
}

impl<T: WithDefinition> WithDefinition for WithCommon<T> {
    fn definition(&self) -> &'static ActionDefinition {
        self.action.definition()
    }
}

impl<T> WithCommonParameters for WithCommon<T> {
    fn timeout(&self) -> &Option<DurationValue> {
        &self.common.timeout
    }

    fn pause_before(&self) -> &Option<DurationValue> {
        &self.common.pause_before
    }

    fn pause_after(&self) -> &Option<DurationValue> {
        &self.common.pause_after
    }

    fn set_timeout(&mut self, timeout: Option<DurationValue>) {
        self.common.timeout = timeout.into();
    }

    fn set_pause_before(&mut self, pause_before: Option<DurationValue>) {
        self.common.pause_before = pause_before.into();
    }

    fn set_pause_after(&mut self, pause_after: Option<DurationValue>) {
        self.common.pause_after = pause_after.into();
    }
}

impl<T: ActionBranches> ActionBranches for WithCommon<T> {
    fn action_branches(&self) -> Vec<BranchKind> {
        self.action.action_branches()
    }
}

impl<T: ParameterAvailability> ParameterAvailability for WithCommon<T> {
    fn disabled_parameters(&self) -> Vec<&'static str> {
        self.action.disabled_parameters()
    }

    fn watched_parameters(&self) -> Vec<&'static str> {
        self.action.watched_parameters()
    }
}

#[derive(ActionDefinitions, Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[static_dispatch::setup]
pub enum ActionInstance {
    // Clipboard
    ClearClipboard(WithCommon<ClearClipboard>),
    SetClipboardText(WithCommon<SetClipboardText>),
    GetClipboardText(WithCommon<GetClipboardText>),
    WaitForClipboardChanged(WithCommon<WaitForClipboardChanged>),

    // Flow
    Test(WithCommon<Test>),
    Goto(WithCommon<Goto>),
    Stop(WithCommon<Stop>),
    Exit(WithCommon<Exit>),
    Marker(WithCommon<Marker>),
    Loop(WithCommon<Loop>),
    Wait(WithCommon<Wait>),
    If(WithCommon<If>),
    Switch(WithCommon<Switch>),
    And(WithCommon<And>),
    Or(WithCommon<Or>),

    // Mouse
    ButtonCondition(WithCommon<ButtonCondition>),
    Click(WithCommon<Click>),
    MoveCursor(WithCommon<MoveCursor>),
    SetCursorPosition(WithCommon<SetCursorPosition>),
    DoubleClick(WithCommon<DoubleClick>),
    Press(WithCommon<Press>),
    Release(WithCommon<Release>),
    GetCursorPosition(WithCommon<GetCursorPosition>),
    Scroll(WithCommon<Scroll>),
    WaitForMovement(WithCommon<WaitForMovement>),
    WaitForScroll(WithCommon<WaitForScroll>),
    WaitForButton(WithCommon<WaitForButton>),

    // System
    Code(WithCommon<Code>),

    // Window
    MessageBox(WithCommon<MessageBox>),
}

static_dispatch::implementation!(WithDefinition for ActionInstance);
static_dispatch::implementation!(WithCommonParameters for ActionInstance);
static_dispatch::implementation!(ActionBranches for ActionInstance);
static_dispatch::implementation!(ParameterAvailability for ActionInstance);

#[derive(Debug)]
pub enum ActionEffect {
    ReadState,
    ChangeState,
    TransformData,
    ControlFlow,
    ExternalSystem,
    Destructive,
}

#[derive(Debug)]
pub enum ActionCategory {
    Mouse,
    Keyboard,
    Window,
    FileSystem,
    Data,
    Flow,
    System,
    Clipboard,
}

#[derive(Debug)]
pub struct ActionDefinition {
    pub id: &'static str,
    pub name: TranslationKey,
    pub description: TranslationKey,
    pub icon: IconType, // TODO: Idea: action icons are always black and white; tint depends on the category
    pub parameters: &'static [Parameter],
    pub create_instance: fn() -> ActionInstance,
    pub effect: ActionEffect,
    pub category: ActionCategory,
    pub supports_timeout: bool,
    pub is_waitable: bool,
    pub platforms: Platforms,
}

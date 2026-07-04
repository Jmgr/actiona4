use icons::common::IconType;
use macros::ActionDefinitions;
use serde::{Deserialize, Serialize};

use crate::{
    TranslationKey,
    actions::{click::Click, code::Code, message_box::MessageBox, test::Test},
    parameters::Parameter,
    tree::BranchKind,
};

pub mod click;
pub mod code;
pub mod message_box;
pub mod test;

pub use macros::{Action, action};

#[static_dispatch::setup]
pub trait WithDefinition {
    fn definition(&self) -> &'static ActionDefinition;
}

#[static_dispatch::setup]
pub trait Branching {
    fn branches(&self) -> Vec<BranchKind> {
        Vec::new()
    }
}

#[derive(ActionDefinitions, Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[static_dispatch::setup]
pub enum ActionInstance {
    Click(Click),
    MessageBox(MessageBox),
    Code(Code),
    Test(Test),
}

static_dispatch::implementation!(WithDefinition for ActionInstance);
static_dispatch::implementation!(Branching for ActionInstance);

#[derive(Debug)]
pub struct ActionDefinition {
    pub id: &'static str,
    pub name: TranslationKey,
    pub description: TranslationKey,
    pub icon: IconType, // TODO: Idea: action icons are always black and white; tint depends on the category
    pub parameters: &'static [Parameter],
    pub create_instance: fn() -> ActionInstance,
}

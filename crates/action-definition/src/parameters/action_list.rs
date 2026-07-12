use std::ops::{Deref, DerefMut};

use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

use crate::{actions::ActionInstance, parameters::Param};

/// A list of actions used as inputs to a flow action.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ActionList(Vec<ActionInstance>);

impl ActionList {
    pub fn new(actions: impl Into<Vec<ActionInstance>>) -> Self {
        Self(actions.into())
    }
}

impl From<Vec<ActionInstance>> for ActionList {
    fn from(actions: Vec<ActionInstance>) -> Self {
        Self::new(actions)
    }
}

impl<N> From<Vec<ActionInstance>> for Param<ActionList, N> {
    fn from(actions: Vec<ActionInstance>) -> Self {
        Self::new(actions.into())
    }
}

impl Deref for ActionList {
    type Target = [ActionInstance];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActionList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = ActionList)]
pub struct ActionListParameter;

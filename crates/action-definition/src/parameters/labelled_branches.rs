use std::ops::{Deref, DerefMut};

use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

use crate::parameters::{Param, value::Value};

/// A named branch together with the value that selects it.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LabelledBranch {
    pub name: String,
    pub value: Value,
}

impl LabelledBranch {
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// User-defined branches that each carry a label value.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LabelledBranches(Vec<LabelledBranch>);

impl LabelledBranches {
    pub fn new(branches: impl Into<Vec<LabelledBranch>>) -> Self {
        Self(branches.into())
    }
}

impl From<Vec<LabelledBranch>> for LabelledBranches {
    fn from(branches: Vec<LabelledBranch>) -> Self {
        Self::new(branches)
    }
}

impl<N> From<Vec<LabelledBranch>> for Param<LabelledBranches, N> {
    fn from(branches: Vec<LabelledBranch>) -> Self {
        Self::new(branches.into())
    }
}

pub fn is_empty<N>(branches: &Param<LabelledBranches, N>) -> bool {
    branches.is_empty()
}

impl Deref for LabelledBranches {
    type Target = [LabelledBranch];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LabelledBranches {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = LabelledBranches)]
pub struct LabelledBranchesParameter;

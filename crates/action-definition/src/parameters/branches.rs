use std::ops::{Deref, DerefMut};

use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

use crate::parameters::Param;

/// User-defined branch names.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Branches(Vec<String>);

impl Branches {
    pub fn new(branches: impl Into<Vec<String>>) -> Self {
        Self(branches.into())
    }
}

impl From<Vec<String>> for Branches {
    fn from(branches: Vec<String>) -> Self {
        Self::new(branches)
    }
}

impl<N> From<Vec<String>> for Param<Branches, N> {
    fn from(branches: Vec<String>) -> Self {
        Self::new(branches.into())
    }
}

#[must_use]
pub fn is_empty<N>(branches: &Param<Branches, N>) -> bool {
    branches.is_empty()
}

impl Deref for Branches {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Branches {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Branches)]
pub struct BranchesParameter;

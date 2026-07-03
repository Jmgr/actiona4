#![allow(unused)] // TMP

use std::borrow::Cow;

use fluent::{FluentBundle, FluentResource};
use serde::{Deserialize, Serialize};

pub mod actions;
pub mod parameters;
pub mod rpc;
pub mod scriptable;
pub mod tree;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AddTenRequest {
    Value { value: i32 },
}

impl AddTenRequest {
    pub fn value(&self) -> i32 {
        match self {
            Self::Value { value } => *value,
        }
    }
}

/// A reference to a Fluent message, optionally one of its attributes.
///
/// Fluent groups related strings under a single message with `.attributes`,
/// so a parameter's name and description share one message id:
///
/// ```ftl
/// action-message-box-title =
///     .name = Title
///     .description = The title of the message box
/// ```
///
/// `TranslationKey::with_attribute("action-message-box-title", "name")` resolves
/// the `.name` attribute of that message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationKey {
    pub id: &'static str,
    pub attribute: Option<&'static str>,
}

impl TranslationKey {
    /// A message with no attribute (resolves the message's own value).
    pub const fn new(id: &'static str) -> Self {
        Self {
            id,
            attribute: None,
        }
    }

    /// One of a message's attributes (e.g. `.name`, `.description`).
    pub const fn with_attribute(id: &'static str, attribute: &'static str) -> Self {
        Self {
            id,
            attribute: Some(attribute),
        }
    }

    /// Resolve this key against a bundle, formatting the pattern.
    ///
    /// Returns `None` if the message — or the requested attribute — is missing.
    pub fn resolve<'bundle>(
        &self,
        bundle: &'bundle FluentBundle<FluentResource>,
    ) -> Option<Cow<'bundle, str>> {
        let message = bundle.get_message(self.id)?;
        let pattern = match self.attribute {
            Some(attribute) => message.get_attribute(attribute)?.value(),
            None => message.value()?,
        };

        let mut errors = Vec::new();
        Some(bundle.format_pattern(pattern, None, &mut errors)) // TODO: return error
    }
}

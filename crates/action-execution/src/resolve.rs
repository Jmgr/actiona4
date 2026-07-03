use action_definition::scriptable::Scriptable;

use crate::ExecutionContext;

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    // TODO
}

/// Resolves a [`Scriptable`] value against an execution context.
pub trait Resolve<T> {
    fn resolve(&self, context: &ExecutionContext) -> Result<T, ResolveError>;
}

impl<T: Clone> Resolve<T> for Scriptable<T> {
    fn resolve(&self, _context: &ExecutionContext) -> Result<T, ResolveError> {
        Ok(match self {
            Scriptable::Static { value } => value.clone(),
            Scriptable::Script { .. } => todo!(), // TODO: evaluate JS and convert into T
        })
    }
}

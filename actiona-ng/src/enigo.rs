use std::fmt::Display;

use enigo::{InputError, NewConError};
use rquickjs::{Ctx, Exception};

use crate::IntoJsResult;

pub enum EnigoError {
    InputError(InputError),
    NewConError(NewConError),
}

impl Display for EnigoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Self::InputError(input_error) => input_error.to_string(),
                Self::NewConError(new_con_error) => new_con_error.to_string(),
            }
        )
    }
}

pub type EnigoResult<T> = Result<T, EnigoError>;

impl<T> IntoJsResult<T> for EnigoResult<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &format!("Enigo: {err}")))
    }
}

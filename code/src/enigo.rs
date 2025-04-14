use std::fmt::Display;

use enigo::{InputError, NewConError};
use rquickjs::{Ctx, Exception};

use crate::IntoJS;

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
                EnigoError::InputError(input_error) => input_error.to_string(),
                EnigoError::NewConError(new_con_error) => new_con_error.to_string(),
            }
        )
    }
}

pub type EnigoResult<T> = Result<T, EnigoError>;

impl<T> IntoJS<T> for EnigoResult<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &format!("Enigo: {}", err)))
    }
}

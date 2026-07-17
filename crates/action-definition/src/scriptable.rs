use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use strum::EnumIs;

#[derive(Clone, Debug, Deserialize, EnumIs, Serialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum Scriptable<T> {
    Static { value: T },
    Script { source: String },
}

impl<T: Default> Default for Scriptable<T> {
    fn default() -> Self {
        Self::Static {
            value: T::default(),
        }
    }
}

impl<T: Display> Display for Scriptable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static { value } => write!(f, "{value}"),
            Self::Script { source } => write!(f, "{source}"),
        }
    }
}

impl<T> Scriptable<T> {
    pub fn new_static(value: impl Into<T>) -> Self {
        Self::Static {
            value: value.into(),
        }
    }

    pub fn new_script(source: impl Into<String>) -> Self {
        Self::Script {
            source: source.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Scriptable;

    #[test]
    fn serializes_static_value_with_mode_and_value() {
        let field = Scriptable::<i32>::new_static(42);

        let serialized = serde_json::to_value(field).unwrap();

        assert_eq!(
            serialized,
            json!({
                "mode": "static",
                "value": 42,
            })
        );
    }

    #[test]
    fn serializes_script_with_mode_and_source() {
        let field = Scriptable::<i32>::new_script("mouse.x + 10");

        let serialized = serde_json::to_value(field).unwrap();

        assert_eq!(
            serialized,
            json!({
                "mode": "script",
                "source": "mouse.x + 10",
            })
        );
    }
}

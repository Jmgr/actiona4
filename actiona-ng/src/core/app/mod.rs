use std::collections::BTreeMap;

pub mod js;

pub struct App {}

impl App {
    #[must_use]
    pub fn env_vars() -> BTreeMap<String, String> {
        std::env::vars_os()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().to_string(),
                    value.to_string_lossy().to_string(),
                )
            })
            .collect()
    }
}

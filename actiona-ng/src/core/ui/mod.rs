use std::fmt::Debug;

use eyre::{Result, bail, eyre};
use itertools::Itertools;
use slint::ComponentHandle;
use slint_interpreter::Compiler;

pub mod js;

#[derive(Default)]
pub struct Ui {
    compiler: Compiler,
}

impl Ui {
    pub async fn load(&self, source: &str, component_name: &str) -> Result<()> {
        let result = self
            .compiler
            .build_from_source(source.to_string(), "source".into())
            .await;

        if result.has_errors() {
            bail!("Compilation failed: {}", result.diagnostics().join("\n"));
        }

        let definition = result
            .component(component_name)
            .ok_or(eyre!("Component {component_name} not found"))?;

        let instance = definition.create()?;

        //instance.set_property("my_name", Value::from(SharedString::from("Foo")))?;

        instance.run()?;

        Ok(())
    }
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ui")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Ui;

    const CODE: &str = r#"
import { AboutSlint, Button, VerticalBox } from "std-widgets.slint";

export component Demo {
    VerticalBox {
        alignment: start;
        Text {
            text: "Hello World!";
            font-size: 24px;
            horizontal-alignment: center;
        }
        AboutSlint {
            preferred-height: 150px;
        }
        HorizontalLayout { alignment: center; Button { text: "OK!"; } }
    }
}

"#;

    #[tokio::test]
    async fn test_ui() {
        let ui = Ui::default();
        ui.load(CODE, "Demo").await.unwrap();
    }
}

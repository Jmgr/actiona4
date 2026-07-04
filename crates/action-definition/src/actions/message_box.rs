use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{Branching, action},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageBoxButtons {
    #[default]
    Ok,
    OkCancel,
    Yes,
    YesNo,
    YesNoCancel,
}

#[action(icon = MessageSquareMore)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MessageBox {
    #[parameter]
    pub title: Scriptable<String>,
    #[parameter]
    pub text: Scriptable<String>,
    #[parameter]
    pub buttons: MessageBoxButtons,
}

impl Branching for MessageBox {
    fn branches(&self) -> Vec<BranchKind> {
        match *self.buttons {
            MessageBoxButtons::Ok => vec![],
            MessageBoxButtons::OkCancel => vec![BranchKind::Cancel],
            MessageBoxButtons::Yes => vec![BranchKind::Yes],
            MessageBoxButtons::YesNo => vec![BranchKind::Yes, BranchKind::No],
            MessageBoxButtons::YesNoCancel => {
                vec![BranchKind::Yes, BranchKind::No, BranchKind::Cancel]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use macros::ActionEnum;
    use serde::{Deserialize, Serialize};

    use crate::parameters::ParameterStorage;

    #[derive(ActionEnum, Clone, Copy, Debug, Deserialize, Serialize)]
    #[serde(rename_all = "kebab-case")]
    enum RenamedOptions {
        Plain,
        #[serde(rename = "custom-id")]
        Custom,
        #[serde(rename(serialize = "save-id", deserialize = "load-id"))]
        Split,
    }

    #[test]
    fn action_enum_uses_serde_variant_names_for_metadata_ids() {
        let variants = <RenamedOptions as ParameterStorage>::DEFAULT_SETTINGS.variants;
        let ids = variants
            .iter()
            .map(|variant| variant.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, ["plain", "custom-id", "save-id"]);
    }
}

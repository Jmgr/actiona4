use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    scriptable::Scriptable,
    tree::BranchKind,
};

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageBoxButtons {
    #[default]
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageBoxIcon {
    #[default]
    Info,
    Warning,
    Error,
}

#[action(icon = MessageSquareMore, effect = ExternalSystem, category = Window, timeout = true)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MessageBox {
    #[parameter]
    pub title: Scriptable<Option<String>>,

    #[parameter]
    pub text: Scriptable<String>,

    #[parameter]
    pub buttons: MessageBoxButtons,

    #[parameter]
    pub icon: Scriptable<Option<MessageBoxIcon>>,

    #[parameter]
    pub ok_label: Scriptable<Option<String>>,

    #[parameter]
    pub yes_label: Scriptable<Option<String>>,

    #[parameter]
    pub no_label: Scriptable<Option<String>>,

    #[parameter]
    pub cancel_label: Scriptable<Option<String>>,
}

impl ActionBranches for MessageBox {
    fn action_branches(&self) -> Vec<BranchKind> {
        match *self.buttons {
            MessageBoxButtons::Ok => vec![BranchKind::Ok],
            MessageBoxButtons::OkCancel => vec![BranchKind::Ok, BranchKind::Cancel],
            MessageBoxButtons::YesNo => vec![BranchKind::Yes, BranchKind::No],
            MessageBoxButtons::YesNoCancel => {
                vec![BranchKind::Yes, BranchKind::No, BranchKind::Cancel]
            }
        }
    }
}

impl ParameterAvailability for MessageBox {
    fn disabled_parameters(&self) -> Vec<&'static str> {
        let ok = self.ok_label.name();
        let yes = self.yes_label.name();
        let no = self.no_label.name();
        let cancel = self.cancel_label.name();

        match *self.buttons {
            MessageBoxButtons::Ok => vec![yes, no, cancel],
            MessageBoxButtons::OkCancel => vec![yes, no],
            MessageBoxButtons::YesNo => vec![ok, cancel],
            MessageBoxButtons::YesNoCancel => vec![ok],
        }
    }

    fn watched_parameters(&self) -> Vec<&'static str> {
        vec![self.buttons.name()]
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

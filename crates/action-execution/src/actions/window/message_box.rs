use action_definition::{
    actions::window::message_box::{MessageBox, MessageBoxButtons, MessageBoxIcon},
    parameters::ParameterKind,
    post_run::PostRun,
    tree::BranchKind,
};
use actiona_core::api::dialogs::{
    Dialogs, MessageBoxOptions, MessageBoxResult,
    js::{JsMessageBoxButtons, JsMessageBoxIcon},
};

use crate::{
    ExecutionContext, ResolveParam, Runnable,
    error::RunError,
    resolve_param::{ScriptableParamValue, ValidateParamValue, ValidationError},
};

fn to_core_buttons(
    buttons: MessageBoxButtons,
    ok_label: Option<String>,
    yes_label: Option<String>,
    no_label: Option<String>,
    cancel_label: Option<String>,
) -> Result<JsMessageBoxButtons, eyre::Report> {
    Ok(match buttons {
        MessageBoxButtons::Ok => match ok_label {
            Some(ok_label) => JsMessageBoxButtons::ok_custom(ok_label),
            None => JsMessageBoxButtons::ok(),
        },

        MessageBoxButtons::OkCancel => match (ok_label, cancel_label) {
            (None, None) => JsMessageBoxButtons::ok_cancel(),
            (Some(ok_label), Some(cancel_label)) => {
                JsMessageBoxButtons::ok_cancel_custom(ok_label, cancel_label)
            }
            _ => eyre::bail!("OK/Cancel custom labels must be set together"),
        },

        MessageBoxButtons::YesNo => JsMessageBoxButtons::yes_no(),

        MessageBoxButtons::YesNoCancel => match (yes_label, no_label, cancel_label) {
            (None, None, None) => JsMessageBoxButtons::yes_no_cancel(),
            (Some(yes_label), Some(no_label), Some(cancel_label)) => {
                JsMessageBoxButtons::yes_no_cancel_custom(yes_label, no_label, cancel_label)
            }
            _ => eyre::bail!("Yes/No/Cancel custom labels must be set together"),
        },
    })
}

fn to_core_icon(icon: MessageBoxIcon) -> JsMessageBoxIcon {
    match icon {
        MessageBoxIcon::Info => JsMessageBoxIcon::Info,
        MessageBoxIcon::Warning => JsMessageBoxIcon::Warning,
        MessageBoxIcon::Error => JsMessageBoxIcon::Error,
    }
}

fn from_core_icon(icon: JsMessageBoxIcon) -> MessageBoxIcon {
    match icon {
        JsMessageBoxIcon::Info => MessageBoxIcon::Info,
        JsMessageBoxIcon::Warning => MessageBoxIcon::Warning,
        JsMessageBoxIcon::Error => MessageBoxIcon::Error,
    }
}

impl ScriptableParamValue for MessageBoxIcon {
    type ScriptValue = JsMessageBoxIcon;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        from_core_icon(value)
    }
}

impl ValidateParamValue for MessageBoxIcon {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl Runnable for MessageBox {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let title = self.title.resolve(context).await?;
        let text = self.text.resolve(context).await?;
        let buttons = self.buttons;
        let icon = self.icon.resolve(context).await?;
        let ok_label = self.ok_label.resolve(context).await?;
        let yes_label = self.yes_label.resolve(context).await?;
        let no_label = self.no_label.resolve(context).await?;
        let cancel_label = self.cancel_label.resolve(context).await?;

        let mut options = MessageBoxOptions::default();

        if let Some(title) = title {
            options.title = Some(title);
        }

        options.buttons = Some(to_core_buttons(
            *buttons,
            ok_label,
            yes_label,
            no_label,
            cancel_label,
        )?);

        if let Some(icon) = icon {
            options.icon = Some(to_core_icon(icon));
        }

        let result = Dialogs::message_box(text, Some(options)).await?;

        Ok(match result {
            MessageBoxResult::Yes => PostRun::Branch(BranchKind::Yes),
            MessageBoxResult::No => PostRun::Branch(BranchKind::No),
            MessageBoxResult::Ok => PostRun::Branch(BranchKind::Ok),
            MessageBoxResult::Cancel => PostRun::Branch(BranchKind::Cancel),
        })
    }
}

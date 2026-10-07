use action_definition::{
    actions::window::message_box::{MessageBox, MessageBoxButtons, MessageBoxIcon},
    parameters::ParameterKind,
    post_run::PostRun,
    tree::BranchKind,
};
use actiona_core::api::dialogs::js::JsMessageBoxIcon;
use dialogs::{ButtonLabels, MessageBoxOptions, MessageBoxResult};

use crate::{
    ExecutionContext, ResolveParam, Runnable,
    error::{RunError, RunErrorKind},
    resolve_param::{ScriptableParamValue, ValidateParamValue, ValidationError},
};

const fn to_dialogs_buttons(buttons: MessageBoxButtons) -> dialogs::MessageBoxButtons {
    match buttons {
        MessageBoxButtons::Ok => dialogs::MessageBoxButtons::Ok,
        MessageBoxButtons::OkCancel => dialogs::MessageBoxButtons::OkCancel,
        MessageBoxButtons::YesNo => dialogs::MessageBoxButtons::YesNo,
        MessageBoxButtons::YesNoCancel => dialogs::MessageBoxButtons::YesNoCancel,
    }
}

const fn to_core_icon(icon: MessageBoxIcon) -> JsMessageBoxIcon {
    match icon {
        MessageBoxIcon::Info => JsMessageBoxIcon::Info,
        MessageBoxIcon::Warning => JsMessageBoxIcon::Warning,
        MessageBoxIcon::Error => JsMessageBoxIcon::Error,
    }
}

const fn from_core_icon(icon: JsMessageBoxIcon) -> MessageBoxIcon {
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
        let icon = self.icon.resolve(context).await?;
        let labels = ButtonLabels {
            ok: self.ok_label.resolve(context).await?,
            cancel: self.cancel_label.resolve(context).await?,
            yes: self.yes_label.resolve(context).await?,
            no: self.no_label.resolve(context).await?,
        };

        let options = MessageBoxOptions {
            title: title.unwrap_or_default(),
            text,
            icon: to_core_icon(icon.unwrap_or_default()).into(),
            buttons: to_dialogs_buttons(*self.buttons),
            labels,
        };

        let dialogs = context.runtime.dialogs();
        let result = tokio::select! {
            () = context.cancellation_token.cancelled() => {
                return Err(RunError::new(RunErrorKind::Canceled));
            }
            result = dialogs.message_box(options) => result.map_err(eyre::Report::from)?,
        };

        Ok(match result {
            MessageBoxResult::Yes => PostRun::Branch(BranchKind::Yes),
            MessageBoxResult::No => PostRun::Branch(BranchKind::No),
            MessageBoxResult::Ok => PostRun::Branch(BranchKind::Ok),
            MessageBoxResult::Cancel => PostRun::Branch(BranchKind::Cancel),
        })
    }
}

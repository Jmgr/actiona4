use std::{fmt::Debug, sync::Arc};

use derive_more::Constructor;
use eyre::Result;
use tauri_plugin_dialog::{
    DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult,
};
use tokio::sync::oneshot;

use crate::runtime::Runtime;

pub mod js;

#[derive(Constructor, Debug)]
pub struct Ui {
    //compiler: Arc<Mutex<Compiler>>,
    runtime: Arc<Runtime>,
}

impl Ui {
    pub async fn display_messagebox(&self) -> Result<MessageDialogResult> {
        let app = self.runtime.tauri_app();
        let (finished_sender, finished_receiver) = oneshot::channel();
        app.dialog()
            .message("foo")
            .title("bar")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCancel)
            .show_with_result(move |button| {
                finished_sender.send(button).unwrap();
            });

        Ok(finished_receiver.await?)
        /*
        //WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        //    .title("Tauri + Tokio (custom)")
        //    .build()?;
         */
    }
}

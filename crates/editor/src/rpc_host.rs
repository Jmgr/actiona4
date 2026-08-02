//! Host-side implementation of the shared [`Api`] RPC surface, plus the
//! host → UI event sink.
//!
//! The host implements the trait with plain `async fn`s; `api_serve` (generated
//! in `common::rpc`) routes an incoming `(command, json)` to the right method.
//! Handlers push notifications back to the UI through [`Events`]. See
//! `connect_host_bridge` in `main.rs` for how this plugs into the webview.

use std::sync::Arc;

use action_definition::{
    actions::{Click, MessageBox, Test},
    tree::ActionTree,
};
use editor::rpc::{Api, EventSink, HostEvent};
use webview_app::webview::{WebView, WebViewHandle};

/// Host → UI push channel: serializes a [`HostEvent`] and dispatches it to the
/// page as a `host-event` `CustomEvent`. The handle is an `async_channel`
/// sender, so `send` is safe to call from any thread (the GTK loop drains it).
#[derive(Clone)]
pub struct Events {
    handle: WebViewHandle,
}

impl Events {
    pub const fn new(handle: WebViewHandle) -> Self {
        Self { handle }
    }
}

impl EventSink for Events {
    fn send(&self, event: HostEvent) {
        let json = serde_json::to_string(&event).expect("RPC: failed to serialize event");
        WebView::eval(
            self.handle.clone(),
            &format!("window.dispatchEvent(new CustomEvent('host-event', {{ detail: {json} }}));"),
        );
    }
}

/// The host's RPC implementation. Holds the event sink (and, in real usage, the
/// action tree, the runtime handle, …).
pub struct HostApi {
    events: Arc<dyn EventSink>,
}

impl HostApi {
    pub fn new(events: Arc<dyn EventSink>) -> Self {
        Self { events }
    }
}

impl Api for HostApi {
    #[allow(clippy::unwrap_used)] // TMP
    async fn load_rows(&self) {
        let mut tree = ActionTree::default();
        let root = tree.root();

        // Add a few test actions

        let message_box = tree
            .append_new_action(&MessageBox::DEFINITION, root)
            .unwrap();
        tree.set_node_label(message_box, "confirm_dialog").unwrap();
        tree.set_node_comment(message_box, "Ask the user before continuing")
            .unwrap();

        let test = tree.append_new_action(&Test::DEFINITION, root).unwrap();
        tree.set_node_label(test, "branch_decision").unwrap();
        tree.set_node_comment(test, "Split the workflow into true and false paths")
            .unwrap();

        let branches = tree.get_node(test).unwrap().children().to_vec();

        let true_branch = branches[0];
        let true_click = tree
            .append_new_action(&Click::DEFINITION, true_branch)
            .unwrap();
        tree.set_node_label(true_click, "accepted_click").unwrap();
        tree.set_node_comment(true_click, "Click the accepted-state target")
            .unwrap();

        let false_branch = branches[1];
        let false_click = tree
            .append_new_action(&Click::DEFINITION, false_branch)
            .unwrap();
        tree.set_node_label(false_click, "rejected_click").unwrap();
        tree.set_node_comment(false_click, "Click the rejected-state target")
            .unwrap();

        let final_click = tree.append_new_action(&Click::DEFINITION, root).unwrap();
        tree.set_node_label(final_click, "final_click").unwrap();

        self.events.send(HostEvent::Tree(tree));
    }

    async fn exit(&self) {
        // Handled by connect_host_bridge because Request is UI-thread-bound.
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use action_definition::{
        actions::WithDefinition,
        tree::{BranchKind, NodePayload, Static},
    };

    use super::*;

    /// An [`EventSink`] that records what the host pushed, so tests can assert on it.
    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<HostEvent>>);

    impl EventSink for RecordingSink {
        fn send(&self, event: HostEvent) {
            self.0.lock().unwrap().push(event);
        }
    }

    #[tokio::test]
    async fn load_rows_pushes_sample_tree_with_branches_and_actions() {
        let sink = Arc::new(RecordingSink::default());
        let api = HostApi::new(sink.clone());

        api.load_rows().await;

        let events = sink.0.lock().unwrap();
        let [HostEvent::Tree(tree)] = events.as_slice() else {
            panic!("expected a single tree event");
        };

        let rows = tree.rows();
        assert_eq!(rows.len(), 8);

        let kinds = rows
            .iter()
            .map(|&id| match tree.get_node(id).unwrap().payload() {
                NodePayload::Static(Static::Root) => "root",
                NodePayload::Static(Static::Branch(BranchKind::True)) => "branch:true",
                NodePayload::Static(Static::Branch(BranchKind::False)) => "branch:false",
                NodePayload::Static(Static::Branch(_)) => "branch:other",
                NodePayload::Action(action) => action.definition().id,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            [
                "root",
                "message_box",
                "test",
                "branch:true",
                "click",
                "branch:false",
                "click",
                "click",
            ],
        );

        let labels = rows
            .iter()
            .map(|&id| tree.get_node(id).unwrap().label())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            [
                None,
                Some("confirm_dialog"),
                Some("branch_decision"),
                None,
                Some("accepted_click"),
                None,
                Some("rejected_click"),
                Some("final_click"),
            ],
        );

        let comments = rows
            .iter()
            .map(|&id| tree.get_node(id).unwrap().comment())
            .collect::<Vec<_>>();

        assert_eq!(
            comments,
            [
                None,
                Some("Ask the user before continuing"),
                Some("Split the workflow into true and false paths"),
                None,
                Some("Click the accepted-state target"),
                None,
                Some("Click the rejected-state target"),
                None,
            ],
        );

        for (&row, label) in rows.iter().zip(labels) {
            if let Some(label) = label {
                assert_eq!(tree.node_by_label(label), Some(row));
            }
        }
    }
}

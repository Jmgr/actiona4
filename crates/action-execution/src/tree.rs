use action_definition::{
    post_run::PostRun,
    tree::{ActionTree, BranchKind, Node, NodeId, NodePayload, Static},
};

use crate::{ExecutionContext, RunError, RunErrorKind, Runnable};

impl Runnable for NodePayload {
    async fn run(&self, context: &ExecutionContext) -> Result<PostRun, RunError> {
        match self {
            NodePayload::Static(_) => Ok(PostRun::NextChild),
            NodePayload::Action(action_instance) => action_instance.run(context).await,
        }
    }
}

impl Runnable for Node {
    async fn run(&self, context: &ExecutionContext) -> Result<PostRun, RunError> {
        self.payload().run(context).await
    }
}

/// Drives execution of an [`ActionTree`], walking it in execution order until a
/// node stops the run or the context is cancelled.
#[allow(async_fn_in_trait)]
pub trait RunTree {
    async fn run(&self, context: &ExecutionContext) -> Result<(), RunError>;
}

impl RunTree for ActionTree {
    async fn run(&self, context: &ExecutionContext) -> Result<(), RunError> {
        let mut node_id = self.root();

        loop {
            let node = self
                .get_node(node_id)
                .map_err(|err| RunError::new(err).at_node(node_id))?;
            let post_run = node
                .run(context)
                .await
                .map_err(|err| err.at_node(node_id))?;

            if context.cancellation_token.is_cancelled() {
                break;
            }

            node_id = match post_run {
                PostRun::NextSibling => match self.next_sibling_or_ancestor(node_id) {
                    Some(id) => id,
                    None => break,
                },
                PostRun::NextChild => match self.next_in_preorder(node_id) {
                    Some(id) => id,
                    None => break,
                },
                PostRun::Branch(branch_kind) => find_branch(self, node_id, node, &branch_kind)
                    .map_err(|err| RunError::new(err).at_node(node_id))?,
                PostRun::GotoLabel(label) => self.node_by_label(&label).ok_or_else(|| {
                    RunError::new(RunErrorKind::LabelNotFound(label)).at_node(node_id)
                })?,
                PostRun::Stop => break,
            };
        }

        Ok(())
    }
}

/// Finds the child of `node` that is the branch matching `branch_kind`.
fn find_branch(
    tree: &ActionTree,
    node_id: NodeId,
    node: &Node,
    branch_kind: &BranchKind,
) -> Result<NodeId, RunErrorKind> {
    for child_id in node.children() {
        let child = tree.get_node(*child_id)?;
        let NodePayload::Static(Static::Branch(branch)) = child.payload() else {
            return Err(RunErrorKind::ExpectedChildBranches(node_id));
        };

        if branch_kind == branch {
            return Ok(*child_id);
        }
    }

    Err(RunErrorKind::BranchNotFound(branch_kind.clone(), node_id))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use action_definition::{
        actions::{ActionInstance, code::Code, test::Test},
        parameters::source_code::SourceCode,
        post_run::PostRun,
        scriptable::Scriptable,
        tree::{ActionTree, BranchKind},
    };
    use actiona_core::runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup};
    use tokio_util::sync::CancellationToken;

    use super::RunTree;
    use crate::{ExecutionContext, RunError, RunErrorKind};

    fn test_action(label: &'static str, post_run: PostRun) -> ActionInstance {
        ActionInstance::Test(Test {
            percent: Scriptable::new_script(format!("globalThis.visits.push('{label}'); 0")).into(),
            post_run,
            ..Default::default()
        })
    }

    async fn run_tree_and_collect_visits(tree: ActionTree) -> Result<Vec<String>, RunError> {
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let platform =
            RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new should succeed");

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                script_engine
                    .eval_async::<()>("globalThis.visits = []")
                    .await?;

                let context = ExecutionContext {
                    cancellation_token: CancellationToken::new(),
                    runtime,
                    script_engine: script_engine.clone(),
                };

                let result = match tree.run(&context).await {
                    Ok(()) => {
                        let recorded_visits = script_engine
                            .eval_async::<Vec<String>>("globalThis.visits")
                            .await?;
                        Ok(recorded_visits)
                    }
                    Err(error) => Err(error),
                };

                *output.lock().expect("result mutex should not be poisoned") = Some(result);

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run test tree");

        result
            .lock()
            .expect("result mutex should not be poisoned")
            .take()
            .expect("test tree should finish")
    }

    #[tokio::test]
    async fn next_sibling_runs_the_next_sibling() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(test_action("first", PostRun::NextSibling), tree.root())
            .unwrap();
        tree.append_action_instance(test_action("second", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "second"]);
    }

    #[tokio::test]
    async fn next_child_descends_into_the_first_child() {
        let mut tree = ActionTree::default();
        let first = tree
            .append_action_instance(test_action("first", PostRun::NextChild), tree.root())
            .unwrap();
        let first_branch = tree.get_node(first).unwrap().children()[0];
        tree.append_action_instance(test_action("child", PostRun::Stop), first_branch)
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "child"]);
    }

    #[tokio::test]
    async fn branch_runs_the_matching_branch() {
        let mut tree = ActionTree::default();
        let first = tree
            .append_action_instance(
                test_action("first", PostRun::Branch(BranchKind::False)),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(first).unwrap().children().to_vec();
        tree.append_action_instance(test_action("true", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(test_action("false", PostRun::Stop), branches[1])
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "false"]);
    }

    #[tokio::test]
    async fn goto_label_jumps_to_the_labeled_node() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            test_action("first", PostRun::GotoLabel("target".to_owned())),
            tree.root(),
        )
        .unwrap();
        tree.append_action_instance(test_action("skipped", PostRun::Stop), tree.root())
            .unwrap();
        let target = tree
            .append_action_instance(test_action("target", PostRun::Stop), tree.root())
            .unwrap();
        tree.set_node_label(target, "target").unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "target"]);
    }

    #[tokio::test]
    async fn stop_ends_execution() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(test_action("first", PostRun::Stop), tree.root())
            .unwrap();
        tree.append_action_instance(test_action("second", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first"]);
    }

    #[tokio::test]
    async fn code_action_can_return_action_result() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            ActionInstance::Code(Code::new(SourceCode::from(
                "globalThis.visits.push('code');\nActionResult.stop();",
            ))),
            tree.root(),
        )
        .unwrap();
        tree.append_action_instance(test_action("second", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["code"]);
    }

    #[tokio::test]
    async fn code_action_can_return_branch_result() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Code(
                    Code::new(SourceCode::from(
                        "globalThis.visits.push('code');\nActionResult.branch(ActionBranch.custom('retry'));",
                    ))
                    .with_branches(vec!["skip".to_owned(), "retry".to_owned()]),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(test_action("skip", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(test_action("retry", PostRun::Stop), branches[1])
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["code", "retry"]);
    }

    #[tokio::test]
    async fn script_error_reports_parameter_location_and_node() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Test(Test {
                    percent: Scriptable::new_script(
                        r#"const value: number = 1;
const explode = (): never => {
    throw new Error('percent exploded');
};
explode();
value
"#,
                    )
                    .into(),
                    post_run: PostRun::Stop,
                    ..Default::default()
                }),
                tree.root(),
            )
            .unwrap();

        let error = run_tree_and_collect_visits(tree).await.unwrap_err();

        assert_eq!(
            error.node_id,
            Some(action),
            "error should carry the failing node id"
        );
        let RunErrorKind::ResolveParam(resolve_error) = error.kind else {
            panic!("expected resolve parameter error, got {error:?}");
        };

        assert_eq!(resolve_error.parameter(), "percent");
        assert!(
            resolve_error.error().contains("percent exploded"),
            "unexpected error message: {}",
            resolve_error.error()
        );
        assert_eq!(resolve_error.line(), Some(3));
        assert_eq!(resolve_error.column(), Some(15));
    }
}

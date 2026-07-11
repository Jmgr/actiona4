use std::{sync::Arc, time::Duration};

use action_definition::{
    actions::{WithCommonParameters, WithDefinition},
    post_run::PostRun,
    tree::{ActionTree, BranchKind, Node, NodeId, NodePayload, Static},
};
use actiona_core::{runtime::Runtime, scripting::Engine as ScriptEngine};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{ExecutionContext, RunError, RunErrorKind, Runnable};

impl Runnable for NodePayload {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        match self {
            NodePayload::Static(_) => Ok(PostRun::NextChild), // Static nodes are branches or root, so we should run their children next
            NodePayload::Action(action_instance) => action_instance.run(context).await,
        }
    }

    async fn on_body_enter(&self, context: &mut ExecutionContext) -> Result<(), RunError> {
        if let NodePayload::Action(action_instance) = self {
            action_instance.on_body_enter(context).await?;
        }
        Ok(())
    }

    async fn on_body_completed(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        match self {
            NodePayload::Static(_) => Ok(PostRun::default()),
            NodePayload::Action(action_instance) => {
                action_instance.on_body_completed(context).await
            }
        }
    }
}

impl Runnable for Node {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        self.payload().run(context).await
    }

    async fn on_body_enter(&self, context: &mut ExecutionContext) -> Result<(), RunError> {
        self.payload().on_body_enter(context).await
    }

    async fn on_body_completed(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        self.payload().on_body_completed(context).await
    }
}

/// Drives execution of an [`ActionTree`], walking it in execution order until a
/// node stops the run or the context is cancelled.
#[allow(async_fn_in_trait)]
pub trait RunTree {
    async fn run(
        &self,
        cancellation_token: CancellationToken,
        runtime: Arc<Runtime>,
        script_engine: ScriptEngine,
    ) -> Result<(), RunError>;
}

impl RunTree for ActionTree {
    async fn run(
        &self,
        cancellation_token: CancellationToken,
        runtime: Arc<Runtime>,
        script_engine: ScriptEngine,
    ) -> Result<(), RunError> {
        Runner {
            tree: self,
            context: ExecutionContext::new(cancellation_token, runtime, script_engine),
        }
        .run()
        .await
    }
}

struct Runner<'a> {
    tree: &'a ActionTree,
    context: ExecutionContext,
}

impl Runner<'_> {
    async fn run(&mut self) -> Result<(), RunError> {
        let mut next = RunStep::normal(self.tree.root());

        loop {
            let (node_id, node, post_run) = match next {
                RunStep::Normal(node_id) => {
                    self.context.prepare_action(node_id, self.tree)?;
                    let node = self
                        .tree
                        .get_node(node_id)
                        .map_err(|err| RunError::new(err).at_node(node_id))?;
                    let post_run = self
                        .run_node(node, NodeRun::Normal)
                        .await
                        .map_err(|err| err.at_node(node_id))?;
                    (node_id, node, post_run)
                }
                RunStep::BodyCompleted(node_id) => {
                    self.context.prepare_action(node_id, self.tree)?;
                    let node = self
                        .tree
                        .get_node(node_id)
                        .map_err(|err| RunError::new(err).at_node(node_id))?;
                    let post_run = self
                        .run_node(node, NodeRun::BodyCompleted)
                        .await
                        .map_err(|err| err.at_node(node_id))?;
                    (node_id, node, post_run)
                }
                RunStep::BodyEnter {
                    owner_id,
                    next: following,
                } => {
                    self.context.prepare_action(owner_id, self.tree)?;
                    let owner = self
                        .tree
                        .get_node(owner_id)
                        .map_err(|err| RunError::new(err).at_node(owner_id))?;
                    owner
                        .on_body_enter(&mut self.context)
                        .await
                        .map_err(|err| err.at_node(owner_id))?;
                    next = *following;
                    continue;
                }
            };

            if self.context.cancellation_token.is_cancelled() {
                break;
            }

            next = match post_run {
                PostRun::NextSibling => match self.next_sibling_or_branch_completion(node_id) {
                    Some(next) => next,
                    None => break,
                },
                PostRun::NextChild => match self.next_child_or_branch_completion(node_id) {
                    Some(next) => next,
                    None => break,
                },
                PostRun::Branch(branch_kind) => {
                    let branch = self
                        .find_branch(node_id, node, &branch_kind)
                        .map_err(|err| RunError::new(err).at_node(node_id))?;
                    if branch_kind == BranchKind::Body {
                        RunStep::body_enter(node_id, RunStep::normal(branch))
                    } else {
                        RunStep::normal(branch)
                    }
                }
                PostRun::GotoLabel(label) => {
                    let target = self.tree.node_by_label(&label).ok_or_else(|| {
                        RunError::new(RunErrorKind::LabelNotFound(label)).at_node(node_id)
                    })?;
                    self.enter_body(target)
                        .map_err(|err| RunError::new(err).at_node(node_id))?
                }
                PostRun::Stop => break,
                PostRun::Exit => {
                    self.context.cancellation_token.cancel(); // TODO: check
                    break;
                }
                PostRun::Break => {
                    let loop_id = self.nearest_enclosing_loop(node_id).ok_or_else(|| {
                        RunError::new(RunErrorKind::LoopControlOutsideLoop { action: "Break" })
                            .at_node(node_id)
                    })?;
                    match self.next_sibling_or_branch_completion(loop_id) {
                        Some(next) => next,
                        None => break,
                    }
                }
                PostRun::Continue => {
                    let loop_id = self.nearest_enclosing_loop(node_id).ok_or_else(|| {
                        RunError::new(RunErrorKind::LoopControlOutsideLoop { action: "Continue" })
                            .at_node(node_id)
                    })?;
                    RunStep::body_completed(loop_id)
                }
            };

            if self.context.cancellation_token.is_cancelled() {
                break;
            }
        }

        Ok(())
    }

    async fn run_node(&mut self, node: &Node, run: NodeRun) -> Result<PostRun, RunError> {
        let (timeout, pause_before, pause_after) = match node.payload() {
            NodePayload::Action(action) => (
                action
                    .definition()
                    .supports_timeout
                    .then(|| action.timeout().map(|timeout| timeout.into_inner()))
                    .flatten(),
                action.pause_before().map(|pause| pause.into_inner()),
                action.pause_after().map(|pause| pause.into_inner()),
            ),
            _ => (None, None, None),
        };

        let parent_token = self.context.cancellation_token.clone();
        let action_token = parent_token.child_token();
        self.context.cancellation_token = action_token.clone();

        let result = async {
            if let Some(pause_before) = pause_before {
                wait_for_pause(pause_before, &action_token).await?;
            }

            let action = async {
                match run {
                    NodeRun::Normal => node.run(&mut self.context).await,
                    NodeRun::BodyCompleted => node.on_body_completed(&mut self.context).await,
                }
            };
            let post_run = match timeout {
                Some(timeout) => {
                    select! {
                        result = action => result,
                        _ = sleep(timeout) => {
                            action_token.cancel();
                            Ok(PostRun::Branch(BranchKind::Timeout))
                        }
                    }
                }
                None => action.await,
            }?;

            if !action_token.is_cancelled()
                && let Some(pause_after) = pause_after
            {
                wait_for_pause(pause_after, &action_token).await?;
            }

            Ok(post_run)
        }
        .await;

        self.context.cancellation_token = parent_token;
        result
    }

    /// Finds the child of `node` that is the branch matching `branch_kind`.
    fn find_branch(
        &self,
        node_id: NodeId,
        node: &Node,
        branch_kind: &BranchKind,
    ) -> Result<NodeId, RunErrorKind> {
        for child_id in node.children() {
            let child = self.tree.get_node(*child_id)?;
            let NodePayload::Static(Static::Branch(branch)) = child.payload() else {
                return Err(RunErrorKind::ExpectedChildBranches(node_id));
            };

            if branch_kind == branch {
                return Ok(*child_id);
            }
        }

        Err(RunErrorKind::BranchNotFound(branch_kind.clone(), node_id))
    }

    fn next_child_or_branch_completion(&self, node_id: NodeId) -> Option<RunStep> {
        let node = self.tree.get_node(node_id).ok()?;
        node.children()
            .first()
            .copied()
            .map(RunStep::normal)
            .or_else(|| self.branch_completion(node_id))
            .or_else(|| self.next_sibling_or_branch_completion(node_id))
    }

    fn next_sibling_or_branch_completion(&self, node_id: NodeId) -> Option<RunStep> {
        let mut current_id = node_id;

        while let Some(parent_id) = self.tree.ancestors(current_id).next() {
            let parent = self.tree.get_node(parent_id).ok()?;
            let sibling_ids = parent.children();
            let pos = sibling_ids.iter().position(|&id| id == current_id)?;
            if let Some(&next_id) = sibling_ids.get(pos + 1) {
                return Some(RunStep::normal(next_id));
            }

            if let Some(next) = self.branch_completion(parent_id) {
                return Some(next);
            }

            current_id = parent_id;
        }

        None
    }

    fn branch_completion(&self, branch_id: NodeId) -> Option<RunStep> {
        let branch = self.tree.get_node(branch_id).ok()?;
        let NodePayload::Static(Static::Branch(branch_kind)) = branch.payload() else {
            return None;
        };
        let owner = self.tree.ancestors(branch_id).next()?;

        if branch_kind == &BranchKind::Body {
            return Some(RunStep::body_completed(owner));
        }

        self.next_sibling_or_branch_completion(owner)
    }

    fn nearest_enclosing_loop(&self, node_id: NodeId) -> Option<NodeId> {
        self.tree.ancestors(node_id).find(|&ancestor_id| {
            self.tree.get_node(ancestor_id).is_ok_and(|node| {
                node.payload()
                    .try_as_action_ref()
                    .is_some_and(|action| action.definition().is_looping)
            })
        })
    }

    fn enter_body(&self, target: NodeId) -> Result<RunStep, RunErrorKind> {
        let mut owners = std::iter::once(target)
            .chain(self.tree.ancestors(target))
            .filter_map(|node_id| {
                let node = self.tree.get_node(node_id).ok()?;
                let NodePayload::Static(Static::Branch(BranchKind::Body)) = node.payload() else {
                    return None;
                };
                self.tree.ancestors(node_id).next()
            })
            .collect::<Vec<_>>();
        owners.reverse();

        let mut next = RunStep::normal(target);
        for owner_id in owners.into_iter().rev() {
            next = RunStep::body_enter(owner_id, next);
        }
        Ok(next)
    }
}

async fn wait_for_pause(
    duration: Duration,
    cancellation_token: &CancellationToken,
) -> Result<(), RunError> {
    select! {
        biased;
        _ = cancellation_token.cancelled() => Err(RunError::new(RunErrorKind::Canceled)),
        _ = sleep(duration) => Ok(()),
    }
}

enum NodeRun {
    Normal,
    BodyCompleted,
}

enum RunStep {
    Normal(NodeId),
    BodyEnter {
        owner_id: NodeId,
        next: Box<RunStep>,
    },
    BodyCompleted(NodeId),
}

impl RunStep {
    fn normal(node_id: NodeId) -> Self {
        Self::Normal(node_id)
    }

    fn body_enter(owner_id: NodeId, next: RunStep) -> Self {
        Self::BodyEnter {
            owner_id,
            next: Box::new(next),
        }
    }

    fn body_completed(owner_id: NodeId) -> Self {
        Self::BodyCompleted(owner_id)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc, time::Duration};

    use action_definition::{
        actions::{
            ActionInstance, CommonParameters, Test, WithCommon,
            flow::{
                And, Break, Continue, Exit, For, ForEach, Goto, Loop, Marker, Or, Stop, Switch,
                SwitchCase, Wait, While, wait::WaitUnit,
            },
            system::code::Code,
        },
        parameters::{
            array::Array, duration::DurationValue, source_code::SourceCode, value::Value,
            variable::Variable,
        },
        post_run::PostRun,
        scriptable::Scriptable,
        tree::{ActionTree, BranchKind, NodeId, NodePayload},
    };
    use actiona_core::runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup};
    use parking_lot::Mutex;
    use tokio::sync::broadcast::error::TryRecvError;
    use tokio_util::sync::CancellationToken;

    use super::{RunTree, wait_for_pause};
    use crate::{RunError, RunErrorKind};

    fn post_run_script(post_run: PostRun) -> String {
        match post_run {
            PostRun::NextSibling => "ActionResult.nextSibling();".to_owned(),
            PostRun::NextChild => "ActionResult.nextChild();".to_owned(),
            PostRun::GotoLabel(label) => format!("ActionResult.gotoLabel('{label}');"),
            PostRun::Stop => "ActionResult.stop();".to_owned(),
            PostRun::Branch(BranchKind::False) => {
                "ActionResult.branch(ActionBranch.false());".to_owned()
            }
            other => panic!("unsupported test post-run result: {other:?}"),
        }
    }

    fn code_test_action(label: &'static str, post_run: PostRun) -> ActionInstance {
        ActionInstance::Code(
            Code::new(SourceCode::from(format!(
                "globalThis.visits.push('{label}');\n{}",
                post_run_script(post_run)
            )))
            .into(),
        )
    }

    fn test_action(label: &'static str, post_run: PostRun) -> ActionInstance {
        let visit_id = crate::test_support::register_test_action_visit_label(label);
        ActionInstance::Test(
            Test {
                percent: Scriptable::new_static(visit_id).into(),
                post_run,
                ..Default::default()
            }
            .into(),
        )
    }

    fn for_action(count: u32) -> ActionInstance {
        for_action_with_index(count, "i")
    }

    fn for_action_with_index(count: u32, index_variable: &str) -> ActionInstance {
        ActionInstance::For(
            For {
                count: Scriptable::new_static(count).into(),
                index_variable: Variable::new(index_variable).into(),
            }
            .into(),
        )
    }

    fn for_each_action(array: &str, item_variable: &str) -> ActionInstance {
        ActionInstance::ForEach(
            ForEach {
                array: Array::new(array).into(),
                item_variable: Variable::new(item_variable).into(),
            }
            .into(),
        )
    }

    fn loop_action() -> ActionInstance {
        ActionInstance::Loop(Loop::default().into())
    }

    fn while_action(condition: Scriptable<bool>) -> ActionInstance {
        ActionInstance::While(
            While {
                condition: condition.into(),
            }
            .into(),
        )
    }

    fn wait_action(duration: f64, timeout: Duration) -> ActionInstance {
        ActionInstance::Wait(WithCommon {
            common: CommonParameters {
                timeout: Some(DurationValue::new(timeout)).into(),
                ..Default::default()
            },
            action: Wait {
                duration: Scriptable::new_static(duration).into(),
                ..Default::default()
            },
        })
    }

    fn wait_input(duration: f64) -> ActionInstance {
        ActionInstance::Wait(
            Wait {
                duration: Scriptable::new_static(duration).into(),
                unit: Scriptable::new_static(WaitUnit::Milliseconds).into(),
            }
            .into(),
        )
    }

    async fn run_tree_and_collect_visits(tree: ActionTree) -> Result<Vec<String>, RunError> {
        run_tree_and_collect_visits_with_setup(tree, "").await
    }

    async fn run_tree_and_collect_visits_with_setup(
        tree: ActionTree,
        setup_script: &str,
    ) -> Result<Vec<String>, RunError> {
        let test_action_visit_ids = test_action_visit_ids(&tree);
        let mut test_action_visits = crate::test_support::subscribe_test_action_visits();
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let setup_script = setup_script.to_owned();
        let platform =
            RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new should succeed");

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                script_engine
                    .eval_async::<()>("globalThis.visits = []")
                    .await?;
                if !setup_script.is_empty() {
                    script_engine.eval_async::<()>(&setup_script).await?;
                }

                let result = match tree
                    .run(CancellationToken::new(), runtime, script_engine.clone())
                    .await
                {
                    Ok(()) => {
                        let recorded_visits = script_engine
                            .eval_async::<Vec<String>>("globalThis.visits")
                            .await?;
                        let test_recorded_visits = drain_test_action_visits(
                            &mut test_action_visits,
                            &test_action_visit_ids,
                        );
                        Ok(if test_recorded_visits.is_empty() {
                            recorded_visits
                        } else {
                            test_recorded_visits
                        })
                    }
                    Err(error) => Err(error),
                };

                *output.lock() = Some(result);

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

        result.lock().take().expect("test tree should finish")
    }

    fn test_action_visit_ids(tree: &ActionTree) -> HashSet<i64> {
        fn collect(tree: &ActionTree, node_id: NodeId, ids: &mut HashSet<i64>) {
            let node = tree.get_node(node_id).expect("test node should exist");

            if let NodePayload::Action(ActionInstance::Test(test)) = node.payload()
                && let Scriptable::Static { value } = test.percent.value()
            {
                ids.insert(*value);
            }

            for &child_id in node.children() {
                collect(tree, child_id, ids);
            }
        }

        let mut ids = HashSet::new();
        collect(tree, tree.root(), &mut ids);
        ids
    }

    fn drain_test_action_visits(
        receiver: &mut tokio::sync::broadcast::Receiver<i64>,
        expected_ids: &HashSet<i64>,
    ) -> Vec<String> {
        let mut visits = Vec::new();

        loop {
            match receiver.try_recv() {
                Ok(id) if expected_ids.contains(&id) => {
                    if let Some(label) = crate::test_support::test_action_visit_label(id) {
                        visits.push(label.to_owned());
                    }
                }
                Ok(_) | Err(TryRecvError::Lagged(_)) => {}
                Err(TryRecvError::Empty | TryRecvError::Closed) => break,
            }
        }

        visits
    }

    #[tokio::test]
    async fn next_sibling_runs_the_next_sibling() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(test_action("first", PostRun::default()), tree.root())
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
    async fn completed_branch_skips_sibling_branches_and_continues_after_owner() {
        let mut tree = ActionTree::default();
        let first = tree
            .append_action_instance(
                test_action("first", PostRun::Branch(BranchKind::True)),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(first).unwrap().children().to_vec();
        tree.append_action_instance(test_action("true", PostRun::default()), branches[0])
            .unwrap();
        tree.append_action_instance(test_action("false", PostRun::Stop), branches[1])
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "true", "after"]);
    }

    #[tokio::test]
    async fn empty_completed_branch_skips_sibling_branches_and_continues_after_owner() {
        let mut tree = ActionTree::default();
        let first = tree
            .append_action_instance(
                test_action("first", PostRun::Branch(BranchKind::True)),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(first).unwrap().children().to_vec();
        tree.append_action_instance(test_action("false", PostRun::Stop), branches[1])
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["first", "after"]);
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
    async fn for_runs_its_body_until_count_is_reached() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(test_action("body", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["body", "body", "after"]);
    }

    #[tokio::test]
    async fn for_with_zero_count_skips_body() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(0), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(test_action("body", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["after"]);
    }

    #[tokio::test]
    async fn for_each_binds_each_array_item_once() {
        let mut tree = ActionTree::default();
        let for_each_id = tree
            .append_action_instance(for_each_action("vars.items", "item"), tree.root())
            .unwrap();
        let body = tree.get_node(for_each_id).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push(vars.item.name);\nvars.items = ['replacement'];\nActionResult.nextSibling();",
                ))
                .into(),
            ),
            body,
        )
        .unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits_with_setup(
            tree,
            "globalThis.vars = { items: [{ name: 'first' }, { name: 'second' }] };",
        )
        .await
        .unwrap();

        assert_eq!(visits, ["first", "second", "after"]);
    }

    #[tokio::test]
    async fn for_each_rejects_non_array_values() {
        let mut tree = ActionTree::default();
        let for_each_id = tree
            .append_action_instance(for_each_action("42", "item"), tree.root())
            .unwrap();

        let error = run_tree_and_collect_visits(tree).await.unwrap_err();

        assert_eq!(error.node_id, Some(for_each_id));
        let RunErrorKind::ResolveParam(resolve_error) = error.kind else {
            panic!("expected array parameter resolution error, got {error:?}");
        };
        assert_eq!(resolve_error.parameter(), "array");
        assert!(resolve_error.error().contains("must resolve to an array"));
    }

    #[tokio::test]
    async fn loop_repeats_its_body_until_control_leaves_it() {
        let mut tree = ActionTree::default();
        let loop_id = tree
            .append_action_instance(loop_action(), tree.root())
            .unwrap();
        let body = tree.get_node(loop_id).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push('body');\nglobalThis.visits.length === 2 ? ActionResult.gotoLabel('after') : ActionResult.nextSibling();",
                ))
                .into(),
            ),
            body,
        )
        .unwrap();
        let after = tree
            .append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();
        tree.set_node_label(after, "after").unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["body", "body", "after"]);
    }

    #[tokio::test]
    async fn while_re_evaluates_its_condition_after_each_iteration() {
        let mut tree = ActionTree::default();
        let while_id = tree
            .append_action_instance(
                while_action(Scriptable::new_script("vars.remaining > 0")),
                tree.root(),
            )
            .unwrap();
        let body = tree.get_node(while_id).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push(String(vars.remaining));\nvars.remaining -= 1;\nActionResult.nextSibling();",
                ))
                .into(),
            ),
            body,
        )
        .unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.vars = { remaining: 2 };")
                .await
                .unwrap();

        assert_eq!(visits, ["2", "1", "after"]);
    }

    #[tokio::test]
    async fn for_exposes_the_current_index_variable() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(3), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push(String(vars.i));\nActionResult.nextSibling();",
                ))
                .into(),
            ),
            body,
        )
        .unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["0", "1", "2", "after"]);
    }

    #[tokio::test]
    async fn for_resolves_count_once_before_its_first_iteration() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(
                ActionInstance::For(
                    For {
                        count: Scriptable::new_script("vars.count").into(),
                        index_variable: Variable::new("i").into(),
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push(String(vars.i));\nvars.count = 0;\nActionResult.nextSibling();",
                ))
                .into(),
            ),
            body,
        )
        .unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.vars = { count: 2 };")
                .await
                .unwrap();

        assert_eq!(visits, ["0", "1", "after"]);
    }

    #[tokio::test]
    async fn nested_for_actions_expose_independent_index_variables() {
        let mut tree = ActionTree::default();
        let outer_for = tree
            .append_action_instance(for_action_with_index(2, "row"), tree.root())
            .unwrap();
        let outer_body = tree.get_node(outer_for).unwrap().children()[0];
        let inner_for = tree
            .append_action_instance(for_action_with_index(2, "column"), outer_body)
            .unwrap();
        let inner_body = tree.get_node(inner_for).unwrap().children()[0];
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push(`${vars.row}:${vars.column}`);\nActionResult.nextSibling();",
                ))
                .into(),
            ),
            inner_body,
        )
        .unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["0:0", "0:1", "1:0", "1:1", "after"]);
    }

    #[tokio::test]
    async fn break_leaves_the_nearest_loop() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(test_action("before", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(ActionInstance::Break(Break::default().into()), body)
            .unwrap();
        tree.append_action_instance(test_action("skipped", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["before", "after"]);
    }

    #[tokio::test]
    async fn continue_starts_the_next_iteration() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(test_action("before", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(ActionInstance::Continue(Continue::default().into()), body)
            .unwrap();
        tree.append_action_instance(test_action("skipped", PostRun::default()), body)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["before", "before", "after"]);
    }

    #[tokio::test]
    async fn break_targets_the_innermost_loop() {
        let mut tree = ActionTree::default();
        let outer_for = tree
            .append_action_instance(for_action(1), tree.root())
            .unwrap();
        let outer_body = tree.get_node(outer_for).unwrap().children()[0];
        let inner_for = tree
            .append_action_instance(for_action(2), outer_body)
            .unwrap();
        let inner_body = tree.get_node(inner_for).unwrap().children()[0];
        tree.append_action_instance(test_action("inner", PostRun::default()), inner_body)
            .unwrap();
        tree.append_action_instance(ActionInstance::Break(Break::default().into()), inner_body)
            .unwrap();
        tree.append_action_instance(test_action("skipped", PostRun::default()), inner_body)
            .unwrap();
        tree.append_action_instance(test_action("outer-tail", PostRun::default()), outer_body)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["inner", "outer-tail", "after"]);
    }

    #[tokio::test]
    async fn break_outside_a_loop_errors() {
        let mut tree = ActionTree::default();
        let break_node = tree
            .append_action_instance(ActionInstance::Break(Break::default().into()), tree.root())
            .unwrap();

        let error = run_tree_and_collect_visits(tree).await.unwrap_err();

        assert_eq!(error.node_id, Some(break_node));
        assert!(matches!(
            error.kind,
            RunErrorKind::LoopControlOutsideLoop { action: "Break" }
        ));
    }

    #[tokio::test]
    async fn goto_out_of_for_exits_for_runtime_state() {
        let mut tree = ActionTree::default();
        let for_id = tree
            .append_action_instance(for_action(5), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(
            test_action("body", PostRun::GotoLabel("after".to_owned())),
            body,
        )
        .unwrap();
        let after = tree
            .append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();
        tree.set_node_label(after, "after").unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["body", "after"]);
    }

    #[tokio::test]
    async fn goto_into_for_body_enters_fresh_for_runtime_state() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            test_action("jump", PostRun::GotoLabel("inside".to_owned())),
            tree.root(),
        )
        .unwrap();
        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        let inside = tree
            .append_action_instance(test_action("body", PostRun::default()), body)
            .unwrap();
        tree.set_node_label(inside, "inside").unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["jump", "body", "body", "after"]);
    }

    #[tokio::test]
    async fn goto_into_for_body_initializes_its_index_variable() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            code_test_action("jump", PostRun::GotoLabel("inside".to_owned())),
            tree.root(),
        )
        .unwrap();
        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        let inside = tree
            .append_action_instance(
                ActionInstance::Code(
                    Code::new(SourceCode::from(
                        "globalThis.visits.push(String(vars.i));\nActionResult.nextSibling();",
                    ))
                    .into(),
                ),
                body,
            )
            .unwrap();
        tree.set_node_label(inside, "inside").unwrap();
        tree.append_action_instance(code_test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["jump", "0", "1", "after"]);
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
    async fn action_timeout_runs_timeout_branch() {
        let mut tree = ActionTree::default();
        let wait = tree
            .append_action_instance(wait_action(60.0, Duration::from_millis(1)), tree.root())
            .unwrap();
        let timeout_branch = tree
            .get_node(wait)
            .unwrap()
            .children()
            .iter()
            .copied()
            .find(|&child| {
                matches!(
                    tree.get_node(child).unwrap().payload(),
                    action_definition::tree::NodePayload::Static(
                        action_definition::tree::Static::Branch(BranchKind::Timeout)
                    )
                )
            })
            .expect("wait action should have a timeout branch");
        tree.append_action_instance(test_action("timeout", PostRun::Stop), timeout_branch)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["timeout"]);
    }

    #[tokio::test]
    async fn and_waits_for_every_input_before_continuing() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            ActionInstance::And(
                And {
                    inputs: vec![wait_input(1.0), wait_input(5.0)],
                }
                .into(),
            ),
            tree.root(),
        )
        .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["after"]);
    }

    #[tokio::test]
    async fn or_runs_only_the_winning_input_handler() {
        let mut tree = ActionTree::default();
        let or = tree
            .append_action_instance(
                ActionInstance::Or(
                    Or {
                        inputs: vec![wait_input(5.0), wait_input(1.0)],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let handler = tree.get_node(or).unwrap().children()[1];
        tree.append_action_instance(test_action("winner", PostRun::default()), handler)
            .unwrap();
        tree.append_action_instance(test_action("after", PostRun::Stop), tree.root())
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["winner", "after"]);
    }

    #[tokio::test]
    async fn and_rejects_non_waitable_inputs() {
        let mut tree = ActionTree::default();
        let and = tree
            .append_action_instance(
                ActionInstance::And(
                    And {
                        inputs: vec![ActionInstance::Stop(Stop::default().into())],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();

        let error = run_tree_and_collect_visits(tree).await.unwrap_err();

        assert_eq!(error.node_id, Some(and));
        assert!(matches!(
            error.kind,
            RunErrorKind::NonWaitableInput { action: "stop" }
        ));
    }

    #[tokio::test]
    async fn pauses_stop_when_the_action_token_is_canceled() {
        let cancellation_token = CancellationToken::new();
        let cancel = cancellation_token.clone();
        let cancellation = tokio::spawn(async move {
            tokio::task::yield_now().await;
            cancel.cancel();
        });

        let result = wait_for_pause(Duration::from_secs(60), &cancellation_token).await;
        cancellation.await.unwrap();

        assert!(matches!(
            result,
            Err(RunError {
                kind: RunErrorKind::Canceled,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn code_action_can_return_action_result() {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new(SourceCode::from(
                    "globalThis.visits.push('code');\nActionResult.stop();",
                ))
                .into(),
            ),
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
                    .with_branches(vec!["skip".to_owned(), "retry".to_owned()])
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("skip", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("retry", PostRun::Stop), branches[1])
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["code", "retry"]);
    }

    #[tokio::test]
    async fn switch_matches_case_values_without_string_coercion() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("vars.selected").into(),
                        cases: vec![
                            SwitchCase::new("number", "1"),
                            SwitchCase::new("string", "'1'"),
                        ],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("number", PostRun::Stop), branches[1])
            .unwrap();
        tree.append_action_instance(code_test_action("string", PostRun::Stop), branches[2])
            .unwrap();

        let visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.vars = { selected: 1 };")
                .await
                .unwrap();

        assert_eq!(visits, ["number"]);
    }

    #[tokio::test]
    async fn switch_can_match_object_identity() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("vars.selected").into(),
                        cases: vec![
                            SwitchCase::new("same", "vars.selected"),
                            SwitchCase::new("other", "({})"),
                        ],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("same", PostRun::Stop), branches[1])
            .unwrap();
        tree.append_action_instance(code_test_action("other", PostRun::Stop), branches[2])
            .unwrap();

        let visits = run_tree_and_collect_visits_with_setup(
            tree,
            "globalThis.vars = { selected: { id: 1 } };",
        )
        .await
        .unwrap();

        assert_eq!(visits, ["same"]);
    }

    #[tokio::test]
    async fn switch_matches_array_by_structure() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("vars.selected").into(),
                        cases: vec![SwitchCase::new("array", "[1, 2]")],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("array", PostRun::Stop), branches[1])
            .unwrap();

        let visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.vars = { selected: [1, 2] };")
                .await
                .unwrap();

        assert_eq!(visits, ["array"]);
    }

    #[tokio::test]
    async fn switch_can_match_array_identity() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("vars.selected").into(),
                        cases: vec![
                            SwitchCase::new("same", "vars.selected"),
                            SwitchCase::new("other", "[1, 2]"),
                        ],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("same", PostRun::Stop), branches[1])
            .unwrap();
        tree.append_action_instance(code_test_action("other", PostRun::Stop), branches[2])
            .unwrap();

        let visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.vars = { selected: [1, 2] };")
                .await
                .unwrap();

        assert_eq!(visits, ["same"]);
    }

    #[tokio::test]
    async fn switch_matches_object_by_structure() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("vars.selected").into(),
                        cases: vec![SwitchCase::new(
                            "object",
                            "({ nested: [1, { done: true }], name: 'case' })",
                        )],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("object", PostRun::Stop), branches[1])
            .unwrap();

        let visits = run_tree_and_collect_visits_with_setup(
            tree,
            "globalThis.vars = { selected: { name: 'case', nested: [1, { done: true }] } };",
        )
        .await
        .unwrap();

        assert_eq!(visits, ["object"]);
    }

    #[tokio::test]
    async fn switch_matches_actiona_value_class_by_value() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("new Point(1, 2)").into(),
                        cases: vec![SwitchCase::new("point", "new Point(1, 2)")],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();
        let branches = tree.get_node(action).unwrap().children().to_vec();
        tree.append_action_instance(code_test_action("default", PostRun::Stop), branches[0])
            .unwrap();
        tree.append_action_instance(code_test_action("point", PostRun::Stop), branches[1])
            .unwrap();

        let visits = run_tree_and_collect_visits(tree).await.unwrap();

        assert_eq!(visits, ["point"]);
    }

    #[tokio::test]
    async fn switch_errors_on_non_finite_values() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Switch(
                    Switch {
                        value: Value::from("Number.POSITIVE_INFINITY").into(),
                        cases: vec![SwitchCase::new("infinity", "Number.POSITIVE_INFINITY")],
                    }
                    .into(),
                ),
                tree.root(),
            )
            .unwrap();

        let error = run_tree_and_collect_visits(tree).await.unwrap_err();

        assert_eq!(error.node_id, Some(action));
        let RunErrorKind::SwitchBranchCompareFailed { branch, source } = error.kind else {
            panic!("expected switch compare error, got {error:?}");
        };

        assert_eq!(branch, "infinity");
        assert!(
            source.to_string().contains("Infinity"),
            "unexpected error message: {source}"
        );
    }

    #[tokio::test]
    async fn script_error_reports_parameter_location_and_node() {
        let mut tree = ActionTree::default();
        let action = tree
            .append_action_instance(
                ActionInstance::Code(
                    Code::new(SourceCode::from(
                        r#"const value: number = 1;
const explode = (): never => {
    throw new Error('source exploded');
};
explode();
value
"#,
                    ))
                    .into(),
                ),
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

        assert_eq!(resolve_error.parameter(), "source");
        assert!(
            resolve_error.error().contains("source exploded"),
            "unexpected error message: {}",
            resolve_error.error()
        );
        assert_eq!(resolve_error.line(), Some(3));
        assert_eq!(resolve_error.column(), Some(15));
    }

    #[tokio::test]
    async fn tree_with_all_flow_actions_runs_expected_paths() {
        let mut tree = ActionTree::default();

        tree.append_action_instance(
            ActionInstance::Marker(Marker::default().into()),
            tree.root(),
        )
        .unwrap();
        tree.append_action_instance(test_action("start", PostRun::default()), tree.root())
            .unwrap();
        tree.append_action_instance(
            ActionInstance::Wait(
                Wait {
                    duration: Scriptable::new_static(0.0).into(),
                    unit: Scriptable::new_static(WaitUnit::Milliseconds).into(),
                }
                .into(),
            ),
            tree.root(),
        )
        .unwrap();
        let timeout_wait = tree
            .append_action_instance(
                ActionInstance::Wait(WithCommon {
                    common: CommonParameters {
                        timeout: Some(DurationValue::new(Duration::from_millis(1))).into(),
                        ..Default::default()
                    },
                    action: Wait {
                        duration: Scriptable::new_static(60.0).into(),
                        unit: Scriptable::new_static(WaitUnit::Seconds).into(),
                    },
                }),
                tree.root(),
            )
            .unwrap();
        let timeout_branch = tree
            .get_node(timeout_wait)
            .unwrap()
            .children()
            .iter()
            .copied()
            .find(|&child| {
                matches!(
                    tree.get_node(child).unwrap().payload(),
                    action_definition::tree::NodePayload::Static(
                        action_definition::tree::Static::Branch(BranchKind::Timeout)
                    )
                )
            })
            .expect("wait action should have a timeout branch");
        tree.append_action_instance(test_action("timeout", PostRun::default()), timeout_branch)
            .unwrap();

        let for_id = tree
            .append_action_instance(for_action(2), tree.root())
            .unwrap();
        let body = tree.get_node(for_id).unwrap().children()[0];
        tree.append_action_instance(test_action("loop-body", PostRun::default()), body)
            .unwrap();

        tree.append_action_instance(
            ActionInstance::Goto(
                Goto {
                    target: Scriptable::new_script("globalThis.terminator").into(),
                }
                .into(),
            ),
            tree.root(),
        )
        .unwrap();

        let stop_marker = tree
            .append_action_instance(
                ActionInstance::Marker(Marker::default().into()),
                tree.root(),
            )
            .unwrap();
        tree.set_node_label(stop_marker, "stop-path").unwrap();
        tree.append_action_instance(test_action("before-stop", PostRun::default()), tree.root())
            .unwrap();
        tree.append_action_instance(ActionInstance::Stop(Stop::default().into()), tree.root())
            .unwrap();
        tree.append_action_instance(test_action("after-stop", PostRun::Stop), tree.root())
            .unwrap();

        let exit_marker = tree
            .append_action_instance(
                ActionInstance::Marker(Marker::default().into()),
                tree.root(),
            )
            .unwrap();
        tree.set_node_label(exit_marker, "exit-path").unwrap();
        tree.append_action_instance(test_action("before-exit", PostRun::default()), tree.root())
            .unwrap();
        tree.append_action_instance(ActionInstance::Exit(Exit::default().into()), tree.root())
            .unwrap();
        tree.append_action_instance(test_action("after-exit", PostRun::Stop), tree.root())
            .unwrap();

        let stop_visits = run_tree_and_collect_visits_with_setup(
            tree.clone(),
            "globalThis.terminator = 'stop-path';",
        )
        .await
        .unwrap();
        assert_eq!(
            stop_visits,
            ["start", "timeout", "loop-body", "loop-body", "before-stop"]
        );

        let exit_visits =
            run_tree_and_collect_visits_with_setup(tree, "globalThis.terminator = 'exit-path';")
                .await
                .unwrap();
        assert_eq!(
            exit_visits,
            ["start", "timeout", "loop-body", "loop-body", "before-exit"]
        );
    }
}

use action_definition::tree::{ActionTree, BranchKind, Node, NodeId, NodePayload, Static};

use crate::{ExecutionContext, PostRun, RunError, Runnable};

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
            let node = self.get_node(node_id)?;
            let post_run = node.run(context).await?;

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
                PostRun::Branch(branch_kind) => find_branch(self, node_id, node, &branch_kind)?,
                PostRun::GotoLabel(label) => self
                    .node_by_label(&label)
                    .ok_or(RunError::LabelNotFound(label))?,
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
) -> Result<NodeId, RunError> {
    for child_id in node.children() {
        let child = tree.get_node(*child_id)?;
        let NodePayload::Static(Static::Branch(branch)) = child.payload() else {
            return Err(RunError::ExpectedChildBranches(node_id));
        };

        if branch_kind == branch {
            return Ok(*child_id);
        }
    }

    Err(RunError::BranchNotFound(branch_kind.clone(), node_id))
}

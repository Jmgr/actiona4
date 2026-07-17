use std::any::Any;

use action_definition::tree::{ActionTree, NodeId};

use crate::RunError;

pub struct ActionFrame {
    owner: NodeId,
    state: Box<dyn Any + Send + Sync>,
}

impl ActionFrame {
    #[must_use]
    pub const fn owner(&self) -> NodeId {
        self.owner
    }

    #[must_use]
    pub fn state<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.state.downcast_ref()
    }

    pub fn state_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.state.downcast_mut()
    }
}

#[derive(Default)]
pub struct ExecutionState {
    frames: Vec<ActionFrame>,
}

impl ExecutionState {
    #[must_use]
    pub fn frames(&self) -> &[ActionFrame] {
        &self.frames
    }

    pub fn runtime_state<T>(&self, owner: NodeId) -> Option<&T>
    where
        T: 'static,
    {
        self.frames
            .iter()
            .find(|frame| frame.owner == owner)
            .and_then(ActionFrame::state)
    }

    pub fn runtime_state_mut<T>(&mut self, owner: NodeId) -> Option<&mut T>
    where
        T: 'static,
    {
        self.frames
            .iter_mut()
            .find(|frame| frame.owner == owner)
            .and_then(ActionFrame::state_mut)
    }

    pub fn runtime_state_mut_or_insert_with<T>(
        &mut self,
        owner: NodeId,
        create: impl FnOnce() -> T,
    ) -> &mut T
    where
        T: Send + Sync + 'static,
    {
        if let Some(index) = self.frames.iter().position(|frame| frame.owner == owner) {
            return self.frames[index].state_mut().unwrap_or_else(|| {
                panic!("unexpected runtime state type for action at {owner:?}")
            });
        }

        self.frames.push(ActionFrame {
            owner,
            state: Box::new(create()),
        });

        self.frames
            .last_mut()
            .expect("runtime state frame was just pushed")
            .state_mut()
            .expect("created runtime state should match requested type")
    }

    pub fn reconcile_to(&mut self, tree: &ActionTree, target: NodeId) -> Result<(), RunError> {
        let mut runtime_state_path = runtime_state_path(tree, target);
        if self.frames.iter().any(|frame| frame.owner == target) {
            runtime_state_path.push(target);
        }

        self.frames
            .retain(|frame| runtime_state_path.contains(&frame.owner));

        Ok(())
    }

    pub fn exit_all(&mut self) {
        self.frames.clear();
    }
}

fn runtime_state_path(tree: &ActionTree, target: NodeId) -> Vec<NodeId> {
    let mut path = tree.ancestors(target).collect::<Vec<_>>();
    path.reverse();
    path
}

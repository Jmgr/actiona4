use action_definition::{actions::misc::test::Test, post_run::PostRun};

use crate::{ExecutionContext, Runnable, error::RunError};

impl Runnable for Test {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        // TODO

        #[cfg(test)]
        if let action_definition::scriptable::Scriptable::Static { value } = self.percent.value() {
            use crate::test_support::record_test_action_visit;

            record_test_action_visit(*value);
        }

        Ok(self.post_run.clone())
    }
}

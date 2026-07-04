use action_definition::{actions::test::Test, post_run::PostRun};

use crate::{ExecutionContext, ResolveParam, Runnable, error::RunError};

impl Runnable for Test {
    async fn run(&self, context: &ExecutionContext) -> Result<PostRun, RunError> {
        // TODO

        let percent = self.percent.resolve(context).await?;
        println!("Percent: {}", percent);

        Ok(self.post_run.clone())
    }
}

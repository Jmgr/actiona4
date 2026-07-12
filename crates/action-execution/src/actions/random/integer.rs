use action_definition::{actions::RandomInteger, post_run::PostRun};

use crate::{ExecutionContext, ResolveParam, RunError, RunErrorKind, Runnable};

impl Runnable for RandomInteger {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let minimum = self.minimum.resolve(context).await?;
        let maximum = self.maximum.resolve(context).await?;
        if minimum > maximum {
            return Err(RunError::new(RunErrorKind::InvalidRandomIntegerRange {
                minimum,
                maximum,
            }));
        }

        let result = self.result.resolve(context).await?;
        let value = context.runtime.rng().random_range(minimum..=maximum);
        context.set_variable(result.inner(), value).await?;

        Ok(PostRun::default())
    }
}

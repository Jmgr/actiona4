use action_definition::{actions::RandomItem, post_run::PostRun};

use crate::{ExecutionContext, ResolveParam, ResolveParamError, RunError, RunErrorKind, Runnable};

impl Runnable for RandomItem {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let array = self.array.resolve(context).await?;
        let result = self.result.resolve(context).await?;
        let count = context
            .store_array(array.inner())
            .await
            .map_err(|source| ResolveParamError::new(self.array.name(), &source))?;
        if count == 0 {
            return Err(RunError::new(RunErrorKind::EmptyRandomItem));
        }

        let index = context.runtime.rng().random_range(0..count);
        context
            .set_variable_from_stored_array(result.inner(), index)
            .await?;

        Ok(PostRun::default())
    }
}

use action_definition::{actions::RandomString, post_run::PostRun};
use unicode_segmentation::UnicodeSegmentation;

use crate::{ExecutionContext, ResolveParam, RunError, RunErrorKind, Runnable};

impl Runnable for RandomString {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let length = self.length.resolve(context).await?;
        let characters = self.characters.resolve(context).await?;
        let result = self.result.resolve(context).await?;
        let characters = characters.graphemes(true).collect::<Vec<_>>();
        if length > 0 && characters.is_empty() {
            return Err(RunError::new(RunErrorKind::EmptyRandomCharacters));
        }

        let rng = context.runtime.rng();
        let mut value = String::new();
        for _ in 0..length {
            let index = rng.random_range(0..characters.len());
            value.push_str(characters[index]);
        }
        context.set_variable(result.inner(), value).await?;

        Ok(PostRun::default())
    }
}

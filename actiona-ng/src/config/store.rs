use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use color_eyre::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Store<T> {
    value: Arc<RwLock<T>>,
    directory: PathBuf,
    filename: &'static str,
}

impl<T> Store<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Default,
{
    #[must_use]
    pub fn new(directory: PathBuf, filename: &'static str) -> Self {
        Self {
            value: Default::default(),
            directory,
            filename,
        }
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.directory.join(self.filename)
    }

    pub async fn load(&self) -> Result<()> {
        let contents = match tokio::fs::read_to_string(&self.path()).await {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                // Ignore if the file is not found, we'll use defaults.
                return Ok(());
            }
            Err(err) => return Err(err)?,
        };

        let parsed = toml::from_str(&contents)?;
        *self.value.write() = parsed;

        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.directory).await?;

        let settings = toml::to_string(&*self.value.read())?;

        let temporary_filepath = self.directory.join(format!("{}.tmp", self.filename));

        tokio::fs::write(&temporary_filepath, &settings).await?;

        tokio::fs::rename(temporary_filepath, self.path()).await?;

        Ok(())
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.value.read();

        f(&guard)
    }

    pub async fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R> {
        let result = {
            let mut guard = self.value.write();
            f(&mut guard)
        };

        self.save().await?;

        Ok(result)
    }
}

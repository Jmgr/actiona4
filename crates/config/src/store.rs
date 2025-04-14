use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use color_eyre::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct StoreInner<T> {
    value: RwLock<T>,
    directory: PathBuf,
    filename: &'static str,
}

#[derive(Debug)]
pub struct Store<T> {
    inner: Arc<StoreInner<T>>,
}

impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Store<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Default + Send + Sync,
{
    #[must_use]
    pub fn new(directory: PathBuf, filename: &'static str) -> Self {
        Self {
            inner: Arc::new(StoreInner {
                value: Default::default(),
                directory,
                filename,
            }),
        }
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.inner.directory.join(self.inner.filename)
    }

    pub async fn load(&self) -> Result<()> {
        let contents = match tokio::fs::read_to_string(&self.path()).await {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        let parsed = toml::from_str(&contents)?;
        *self.inner.value.write() = parsed;

        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.inner.directory).await?;

        let serialized_value = toml::to_string(&*self.inner.value.read())?;
        let temporary_filepath = self
            .inner
            .directory
            .join(format!("{}.tmp", self.inner.filename));

        tokio::fs::write(&temporary_filepath, &serialized_value).await?;
        tokio::fs::rename(temporary_filepath, self.path()).await?;

        Ok(())
    }

    pub fn with<R>(&self, operation: impl FnOnce(&T) -> R) -> R {
        let value_guard = self.inner.value.read();

        operation(&value_guard)
    }

    pub async fn with_mut<R>(&self, operation: impl FnOnce(&mut T) -> R + Send) -> Result<R>
    where
        R: Send,
    {
        let result = {
            let mut value_guard = self.inner.value.write();
            operation(&mut value_guard)
        };

        self.save().await?;

        Ok(result)
    }
}

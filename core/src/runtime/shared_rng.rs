use std::sync::Arc;

use parking_lot::Mutex;
use rand::{
    Rng, RngExt, SeedableRng,
    distr::{
        Distribution, StandardUniform,
        uniform::{SampleRange, SampleUniform},
    },
};
use rand_chacha::ChaCha8Rng;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct SharedRng(Arc<Mutex<ChaCha8Rng>>);

impl Default for SharedRng {
    #[instrument(skip_all)]
    fn default() -> Self {
        let mut rng = rand::rng();
        Self(Arc::new(Mutex::new(ChaCha8Rng::from_rng(&mut rng))))
    }
}

impl SharedRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(Arc::new(Mutex::new(ChaCha8Rng::seed_from_u64(seed))))
    }

    pub fn set_seed(&self, seed: u64) {
        let mut guard = self.0.lock();
        *guard = ChaCha8Rng::seed_from_u64(seed);
    }

    pub fn reset_seed(&self) {
        let mut guard = self.0.lock();
        let mut rng = rand::rng();
        *guard = ChaCha8Rng::from_rng(&mut rng);
    }

    #[must_use]
    pub fn random<T>(&self) -> T
    where
        StandardUniform: Distribution<T>,
    {
        let mut guard = self.0.lock();
        guard.random()
    }

    pub fn random_range<T, R>(&self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        let mut guard = self.0.lock();
        guard.random_range(range)
    }

    #[must_use]
    pub fn next_u32(&self) -> u32 {
        let mut guard = self.0.lock();
        guard.next_u32()
    }

    #[must_use]
    pub fn next_u64(&self) -> u64 {
        let mut guard = self.0.lock();
        guard.next_u64()
    }
}

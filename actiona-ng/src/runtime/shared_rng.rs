use std::sync::{Arc, Mutex};

use rand::{
    Rng, RngCore,
    distr::{
        Distribution, StandardUniform,
        uniform::{SampleRange, SampleUniform},
    },
};
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

#[derive(Clone, Debug)]
pub struct SharedRng(Arc<Mutex<ChaCha8Rng>>);

impl Default for SharedRng {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(ChaCha8Rng::from_os_rng())))
    }
}

impl SharedRng {
    pub fn set_seed(&self, seed: u64) {
        let mut guard = self.0.lock().unwrap();
        *guard = ChaCha8Rng::seed_from_u64(seed);
    }

    pub fn reset_seed(&self) {
        let mut guard = self.0.lock().unwrap();
        *guard = ChaCha8Rng::from_os_rng();
    }

    #[must_use]
    pub fn random<T>(&self) -> T
    where
        StandardUniform: Distribution<T>,
    {
        let mut guard = self.0.lock().unwrap();
        guard.random()
    }

    pub fn random_range<T, R>(&self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        let mut guard = self.0.lock().unwrap();
        guard.random_range(range)
    }

    #[must_use]
    pub fn next_u32(&self) -> u32 {
        let mut guard = self.0.lock().unwrap();
        guard.next_u32()
    }

    #[must_use]
    pub fn next_u64(&self) -> u64 {
        let mut guard = self.0.lock().unwrap();
        guard.next_u64()
    }
}

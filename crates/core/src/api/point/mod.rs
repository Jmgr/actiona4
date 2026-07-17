use std::f64::consts::TAU;

pub use types::{Point, point};

use crate::runtime::shared_rng::SharedRng;

pub mod js;

const RANDOM_POINT_MAX_ATTEMPTS: usize = 64;

#[must_use]
pub fn random_point_in_circle(center: Point, radius: f64, rng: SharedRng) -> Point {
    if !radius.is_finite() || radius <= 0.0 {
        return center;
    }

    let (center_x, center_y) = center.as_f64();

    for _ in 0..RANDOM_POINT_MAX_ATTEMPTS {
        let theta = rng.random_range(0.0..TAU);
        let r = radius * rng.random::<f64>().sqrt();
        let x = r.mul_add(theta.cos(), center_x);
        let y = r.mul_add(theta.sin(), center_y);
        let candidate = point(x, y);

        if center.distance_to(candidate) <= radius {
            return candidate;
        }
    }

    center
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use types::point;

    use super::*;

    // ---------- random_in_circle --------------------------------------------
    // Basic property test: results lie within radius after conversion to integer coordinates.

    #[test]
    fn random_in_circle_within_radius() {
        let center = point(100, -50);
        let radius = 100.0;
        let rng = SharedRng::default();

        for _ in 0..1000 {
            let p = random_point_in_circle(center, radius, rng.clone());
            let d = center.distance_to(p);
            assert!(d <= radius, "d={d} > radius");
        }
    }
}

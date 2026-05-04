use std::f64::consts::TAU;

pub use types::point::{Point, point};

use crate::runtime::shared_rng::SharedRng;

pub mod js;

pub fn random_point_in_circle(center: Point, radius: f64, rng: SharedRng) -> Point {
    let (center_x, center_y) = center.as_f64();
    let theta = rng.random_range(0.0..TAU);
    let r = radius * rng.random::<f64>().sqrt();
    let x = r.mul_add(theta.cos(), center_x);
    let y = r.mul_add(theta.sin(), center_y);

    point(x, y)
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use types::point::point;

    use super::*;

    // ---------- random_in_circle --------------------------------------------
    // Basic property test: results lie within ~radius (allowing <= 1.0 slack for rounding to i32).

    #[test]
    fn random_in_circle_within_radius() {
        let center = point(100, -50);
        let radius = 100.0;
        let rng = SharedRng::default();

        for _ in 0..1000 {
            let p = random_point_in_circle(center, radius, rng.clone());
            let d = center.distance_to(p);
            assert!(d <= radius + 1.0, "d={} > radius", d); // rounding slack
        }
    }
}

use crate::img::Image;
use crate::geom::{Color, Vector};

pub trait Sampler {
    /// Validate if `imgs` can be sampled with this sampler.
    fn validate(&self, img: &[Image]) -> bool;
    /// Sample a color from `imgs`.
    ///
    /// NOTE: `v` must be normalized.
    fn sample(&self, imgs: &[Image], v: Vector) -> Color;
}

#[derive(Default)]
pub struct CubeSampler();
impl Sampler for CubeSampler {
    fn validate(&self, imgs: &[Image]) -> bool {
        imgs.len() == 6
    }
    fn sample(&self, imgs: &[Image], v: Vector) -> Color {
        let Vector(x, y, z) = v;
        let dir = [x, y, z];
        let absdir = [x.abs(), y.abs(), z.abs()];
        let i = (0..3).into_iter()
            .max_by(|a, b| {
                absdir[*a].partial_cmp(&absdir[*b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            }).unwrap();
        let (u, v, img) = match (i, dir[i] > 0.0) {
            // Positive x.
            (0, true) => (-z, y, &imgs[0]),
            // Negative x.
            (0, false) => (z, y, &imgs[1]),
            // Positive y.
            (1, true) => (x, -z, &imgs[2]),
            // Negative y.
            (1, false) => (x, z, &imgs[3]),
            // Positive z.
            (2, true) => (x, y, &imgs[4]),
            // Negative z.
            (2, false) => (-x, y, &imgs[5]),
            (_, _) => unreachable!(),
        };
        let max = absdir[i];
        let u = (0.5 * (u / max + 1.0)).max(0.0).min(1.0) * (img.width() - 1) as f32;
        let v = (0.5 * (v / max + 1.0)).max(0.0).min(1.0) * (img.height() - 1) as f32;
        img.load_px(u as usize, v as usize)
    }
}

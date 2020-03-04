use std::ops::Mul;
use crate::geom::{Transform, Triangle};
use crate::scene::Scene;

pub trait Framebuffer {
    /// Color unit.
    type Color;

    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn store(&mut self, x: u32, y: u32, color: Self::Color);
}

#[derive(PartialEq, Eq)]
pub enum HitKind {
    Front, Back
}
pub struct Intersection<RayAttr> {
    /// Data used to describe a hit, say, the position of intersection.
    pub attr: RayAttr,
    /// Front face or back face.
    pub kind: HitKind,
    /// Distance from ray origin to triangle.
    pub t: f32,
}

pub trait RayTracer {
    type Material;
    /// User specified data for computation.
    type Payload;
    /// Ray data.
    type Ray: Clone;
    /// Data that describes how a ray intersected with a primitive.
    type RayAttr;
    /// Color unit. A ray tracer can only write to framebuffers with the same
    /// color type.
    type Color: Clone;

    /// Generate rays and invoke `trace` to trace the rays.
    fn ray_gen(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Self::Color;
    /// Determine whether a ray intersected with an object.
    fn intersect(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        mat: &Self::Material,
    ) -> Option<Intersection<Self::RayAttr>>;
    /// The ray hit any object. Returns whether the hit is accepted.
    fn any_hit(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
        mat: &Self::Material,
    ) -> bool;
    /// The ray didn't hit while all scene objects have been checked.
    fn miss(
        &self,
        ray: &Self::Ray,
        payload: &mut Self::Payload
    ) -> Self::Color;
    /// The ray hit the nearest object.
    fn closest_hit(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
        mat: &Self::Material,
    ) -> Self::Color;

    /// Trace ray in the scene.
    fn trace(
        &self,
        ray: Self::Ray,
        payload: &mut Self::Payload,
    ) -> Self::Color {
        let mut closest: Option<(
            Triangle,
            &Self::Material,
            Intersection<Self::RayAttr>,
        )> = None;
        for obj in self.scene().objs.iter() {
            let verts = obj.verts.iter()
                .map(|&x| obj.world2obj * x)
                .collect::<Vec<_>>();
            for (x, y, z) in obj.idxs.iter() {
                let tri = Triangle::new(
                    verts[*x],
                    verts[*y],
                    verts[*z],
                );
                if let Some(x) = self.intersect(&ray, &tri, &obj.mat) {
                    if self.any_hit(&ray, &tri, &x, payload, &obj.mat) {
                        let tmax = closest.as_ref()
                            .map(|(_, _, intersect)| intersect.t)
                            .unwrap_or(std::f32::INFINITY);
                        if x.t < tmax {
                            closest = Some((tri, &obj.mat, x));
                        }
                    }
                }
            }
        }
        if let Some((tri, mat, intersect)) = closest {
            self.closest_hit(&ray, &tri, &intersect, payload, mat)
        } else {
            self.miss(&ray, payload)
        }
    }

    fn draw<FB>(&self, framebuf: &mut FB)
        where FB: Framebuffer<Color=Self::Color>
    {
        let w = framebuf.width();
        let h = framebuf.height();
        for x in 0..w {
            for y in 0..h {
                let color = self.ray_gen(x, y, w, h);
                framebuf.store(x, y, color);
            }
        }
    }

    /// The scene the tracer is bound to.
    fn scene(&self) -> &Scene<Self::Material>;
}

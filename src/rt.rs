use crate::geom::Triangle;
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
    type Ray;
    /// Data that describes how a ray intersected with a primitive.
    type RayAttr;
    /// Color unit. A ray tracer can only write to framebuffers with the same
    /// color type.
    type Color;

    /// Generate rays and invoke `trace` to trace the rays.
    fn ray_gen(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        scene: &Scene<Self::Material>,
    ) -> Self::Color;
    /// Determine whether a ray intersected with an object.
    fn intersect(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
    ) -> Option<Intersection<Self::RayAttr>>;
    /// The ray hit any object. Returns whether the hit is accepted.
    fn any_hit(
        &self,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
    ) -> bool;
    /// The ray didn't hit while all scene objects have been checked.
    fn miss(&self, payload: &mut Self::Payload) -> Self::Color;
    /// The ray hit the nearest object.
    fn closest_hit(
        &self,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
    ) -> Self::Color;

    /// Trace ray in the scene.
    fn trace(
        &self,
        ray: &Self::Ray,
        payload: &mut Self::Payload,
        scene: &Scene<Self::Material>,
    ) -> Self::Color {
        let mut closest: Option<Intersection<Self::RayAttr>> = None;
        for obj in scene.objs.iter() {
            for (x, y, z) in obj.idxs.iter() {
                let tri = Triangle::new(
                    obj.verts[*x],
                    obj.verts[*y],
                    obj.verts[*z],
                );
                //println!("{:?}", tri);
                if let Some(x) = self.intersect(ray, &tri) {
                    if self.any_hit(&x, payload) {
                        let tmax = closest.as_ref().map(|intersect| intersect.t)
                            .unwrap_or(std::f32::INFINITY);
                        if x.t < tmax {
                            closest = Some(x);
                        }
                    }
                }
            }
        }
        if let Some(intersect) = closest {
            self.closest_hit(&intersect, payload)
        } else {
            self.miss(payload)
        }
    }

    fn draw<FB>(&self, scene: &Scene<Self::Material>, framebuf: &mut FB)
        where FB: Framebuffer<Color=Self::Color>
    {
        let w = framebuf.width();
        let h = framebuf.height();
        for x in 0..w {
            for y in 0..h {
                let color = self.ray_gen(x, y, w, h, scene);
                framebuf.store(x, y, color);
            }
        }
    }
}

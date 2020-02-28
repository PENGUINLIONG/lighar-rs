use std::ops::{Add, Sub, Mul, Div};

#[derive(Clone, Copy)]
struct Point(pub f32, pub f32, pub f32);
impl Point {
    #[inline]
    fn affine_add(self, rhs: Vector) -> Point {
        Point(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
    #[inline]
    fn affine_sub(self, rhs: Vector) -> Point {
        Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
    /// Get the relative position offset vector from `rhs`.
    #[inline]
    fn rel_from(&self, rhs: Point) -> Vector {
        Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}


#[derive(Clone, Copy)]
struct Vector(pub f32, pub f32, pub f32);
impl Vector {
    #[inline]
    fn normalize(self) -> Vector {
        let l = self.dot(self).sqrt();
        Vector(self.0 / l, self.1 / l, self.2 / l)
    }
    #[inline]
    fn dot(self, rhs: Vector) -> f32 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
    #[inline]
    fn cross(self, rhs: Vector) -> Vector {
        let n1 = self.1 * rhs.2 - self.2 * rhs.1;
        let n2 = self.2 * rhs.0 - self.0 * rhs.2;
        let n3 = self.0 * rhs.1 - self.1 * rhs.0;
        Vector(n1, n2, n3)
    }
}
impl Add<Vector> for Vector {
    type Output = Vector;
    #[inline]
    fn add(self, rhs: Vector) -> Self::Output {
        Vector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}
impl Sub<Vector> for Vector {
    type Output = Vector;
    #[inline]
    fn sub(self, rhs: Vector) -> Self::Output {
        Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}
impl Mul<f32> for Vector {
    type Output = Vector;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Vector(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}
impl Mul<Vector> for f32 {
    type Output = Vector;
    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output {
        Vector(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}
impl Div<f32> for Vector {
    type Output = Vector;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Vector(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}


#[derive(Clone)]
struct Triangle {
    /// Origin of the triangle.
    o: Point,
    /// First vector in clockwise order.
    x: Vector,
    /// Second vector in clockwise order.
    y: Vector,
    /// Unit normal vector.
    n: Vector,
}
impl Triangle {
    pub fn new(a: Point, b: Point, c: Point) -> Triangle {
        let x = b.rel_from(a);
        let y = c.rel_from(a);
        // Note that right-hand system axes are in counter-clockwise order.
        let n = y.cross(x).normalize();
        Triangle { o: a, x, y, n }
    }
}


#[derive(Clone)]
struct Ray {
    /// Origin.
    pub o: Point,
    /// Direction.
    pub v: Vector,
}

/// Cast a ray to the triangle and return the point of intersection if such
/// point exists.
#[inline]
fn ray_cast_tri(ray: &Ray, tri: &Triangle) -> Option<Intersection<Point>> {
    // Relative position from the origin of triangle to the origin of the ray.
    let dtriray = ray.o.rel_from(tri.o);
    // Displacement from the triangle plane to the ray origin (in normal unit).
    // <0 if triangle is ahead of the ray origin; >0 if triangle is behind the
    // ray origin (in normal direction).
    let r1 = dtriray.dot(tri.n);
    // Length of projection of the ray direction vector in normal direction. <0
    // if attempting to hit at the front face; >0 if attempting to hit at the
    // back face.
    let r2 = ray.v.dot(tri.n);
    // Don't extract `r1 / r2` for this. It's unknown whether `r2` is 0.
    if r1 * r2 >= 0.0 {
        // Conflicting requirements. The ray want to hit the front face from
        // behind, or want to hit the back face from ahead, which is not
        // possible, or the ray is tracing parallel to the triangle plane.
        return None
    }

    // Intersection position.
    let pos = ray.o.affine_sub(ray.v * r1 / r2);
    // Distance from the ray origin to the triangle.
    let t = r1.abs();
    let kind = if r2 < 0.0 { HitKind::Front } else { HitKind::Back };
    let res = Intersection { attr: pos, kind, t };
    Some(res)
}


trait Framebuffer {
    /// Color unit.
    type Color;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn store(&mut self, x: usize, y: usize, color: Self::Color);
}


struct Object {
    tris: Vec<Triangle>,
}
struct Scene {
    objs: Vec<Object>,
}


enum HitKind {
    Front, Back
}
struct Intersection<RayAttr> {
    /// Data used to describe a hit, say, the position of intersection.
    attr: RayAttr,
    /// Front face or back face.
    kind: HitKind,
    /// Distance from ray origin to triangle.
    t: f32,
}

trait RayTracer {
    /// User specified data for computation.
    type Payload;
    /// Data that describes how a ray intersected with a primitive.
    type RayAttr;
    /// Color unit. A ray tracer can only write to framebuffers with the same
    /// color type.
    type Color;

    /// Generate rays and invoke `trace` to trace the rays.
    fn ray_gen(&self, x: f32, y: f32) -> Self::Color;
    /// Determine whether a ray intersected with an object.
    fn intersect(&self, ray: &Ray, tri: &Triangle) -> Option<Intersection<Self::RayAttr>>;
    /// The ray hit any object. Returns whether the hit is accepted.
    fn any_hit(&self, intersect: &Intersection<Self::RayAttr>, payload: &Self::Payload) -> bool;
    /// The ray didn't hit while all scene objects have been checked.
    fn miss(&self, payload: &Self::Payload) -> Self::Color;
    /// The ray hit the nearest object.
    fn closest_hit(&self, intersect: &Intersection<Self::RayAttr>, payload: &Self::Payload) -> Self::Color;

    /// Trace ray in the scene.
    fn trace<S>(&self, ray: &Ray, payload: &Self::Payload, scene: &Scene) -> Self::Color {
        let mut closest: Option<Intersection<Self::RayAttr>> = None;
        for obj in scene.objs.iter() {
            for tri in obj.tris.iter() {
                if let Some(x) = self.intersect(ray, tri) {
                    let tmax = closest.as_ref().map(|intersect| intersect.t)
                        .unwrap_or(std::f32::INFINITY);
                    if x.t < tmax {
                        closest = Some(x);
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

    fn draw<FB>(&self, framebuf: &mut FB)
        where FB: Framebuffer<Color=Self::Color>
    {
        let w = framebuf.width() as f32;
        let h = framebuf.height() as f32;

        for x in 0..framebuf.width() {
            for y in 0..framebuf.height() {
                let color = self.ray_gen(x as f32 / w, y as f32 / h);
                framebuf.store(x, y, color);
            }
        }
    }
}



fn main() {
    println!("Hello, world!");
}

use std::ops::{Add, Sub, Mul, Div, Neg};
use crate::rt::{Intersection, HitKind};

#[derive(Debug, Clone, Copy)]
pub struct Point(pub f32, pub f32, pub f32);
impl Point {
    #[inline]
    pub fn affine_add(self, rhs: Vector) -> Point {
        Point(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
    #[inline]
    pub fn affine_sub(self, rhs: Vector) -> Point {
        Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
    /// Get the relative position offset vector from `rhs`.
    #[inline]
    pub fn rel_from(&self, rhs: Point) -> Vector {
        Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
    /// Only used to calculate barycentric coordinates.
    #[inline]
    fn to_vec(&self) -> Vector {
        Vector(self.0, self.1, self.2)
    }
}
impl From<Point> for (f32, f32, f32) {
    fn from(x: Point) -> (f32, f32, f32) {
        (x.0, x.1, x.2)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Vector(pub f32, pub f32, pub f32);
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
impl Neg for Vector {
    type Output = Vector;
    #[inline]
    fn neg(self) -> Vector {
        Vector(-self.0, -self.1, -self.2)
    }
}
impl From<Vector> for (f32, f32, f32) {
    fn from(x: Vector) -> (f32, f32, f32) {
        (x.0, x.1, x.2)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);
impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2, self.3 + rhs.3)
    }
}
impl Sub<Color> for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Self::Output {
        Color(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2, self.3 - rhs.3)
    }
}
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color(self.0 * rhs, self.1 * rhs, self.2 * rhs, self.3 * rhs)
    }
}
impl Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color(self * rhs.0, self * rhs.1, self * rhs.2, self * rhs.3)
    }
}
impl From<Color> for [u8; 3] {
    fn from(x: Color) -> [u8; 3] {
        [
            (x.0 * 255.0) as u8,
            (x.1 * 255.0) as u8,
            (x.2 * 255.0) as u8,
        ]
    }
}
impl From<Color> for [u8; 4] {
    fn from(x: Color) -> [u8; 4] {
        [
            (x.0 * 255.0) as u8,
            (x.1 * 255.0) as u8,
            (x.2 * 255.0) as u8,
            (x.3 * 255.0) as u8,
        ]
    }
}
impl From<[u8; 3]> for Color {
    fn from(x: [u8; 3]) -> Color {
        Color(
            (x[0] as f32) / 255.0,
            (x[1] as f32) / 255.0,
            (x[2] as f32) / 255.0,
            1.0,
        )
    }
}
impl From<[u8; 4]> for Color {
    fn from(x: [u8; 4]) -> Color {
        Color(
            (x[0] as f32) / 255.0,
            (x[1] as f32) / 255.0,
            (x[2] as f32) / 255.0,
            (x[3] as f32) / 255.0,
        )
    }
}


#[derive(Debug, Clone)]
pub struct Triangle {
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

// A general purpose ray attribute.
#[derive(Debug, Clone, Copy)]
pub struct Barycentric {
    pub u: f32,
    pub v: f32,
}
impl Barycentric {
    pub fn new(p: &Point, tri: &Triangle) -> Option<Barycentric> {
        let x = tri.x;
        let y = tri.y;
        let p = p.to_vec() - tri.o.to_vec();
        // See: https://gamedev.stackexchange.com/questions/23743/whats-the-most-efficient-way-to-find-barycentric-coordinates
        let d00 = x.dot(x);
        let d01 = x.dot(y);
        let d11 = y.dot(y);
        let d20 = p.dot(x);
        let d21 = p.dot(y);
        let denom = d00 * d11 - d01 * d01;
        let u = (d11 * d20 - d01 * d21) / denom;
        let v = (d00 * d21 - d01 * d20) / denom;
        if u < 0.0 || v < 0.0 || u + v > 1.0 {
            // Invalid weight. Weight must be greater than 0 so that the point
            // is situated inside the triangle.
            None
        } else {
            Some(Barycentric { u, v })
        }
    }
}


/// A general purpose ray.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin.
    pub o: Point,
    /// Direction.
    pub v: Vector,
}


#[derive(Debug, Clone, Copy)]
pub struct Transform {
    // Row 1.
    pub r1: Vector,
    // Row 2.
    pub r2: Vector,
    // Row 3.
    pub r3: Vector,
    // Affine add.
    pub af: Vector,
}
impl Transform {
    pub fn eye() -> Transform {
        Transform {
            r1: Vector(1.0, 0.0, 0.0),
            r2: Vector(0.0, 1.0, 0.0),
            r3: Vector(0.0, 0.0, 1.0),
            af: Vector(0.0, 0.0, 0.0),
        }
    }
    pub fn translate(self, af: Vector) -> Self {
        let af = self.af + af;
        Transform { r1: self.r1, r2: self.r2, r3: self.r3, af }
    }
    pub fn scale(self, scale: Vector) -> Self {
        let r1 = self.r1 * scale.0;
        let r2 = self.r2 * scale.1;
        let r3 = self.r3 * scale.2;
        Transform { r1, r2, r3, af: self.af }
    }
    pub fn rotate(self, angle: f32, axis: Vector) -> Self {
        let (x, y, z) = axis.into();
        let (sin, cos) = angle.sin_cos();
        let rcos = 1.0 - cos;
        let r1 = Vector(
            cos + rcos * x * x,
            rcos * x * y - sin * z,
            rcos * x * z + sin * y,
        );
        let r2 = Vector(
            rcos * y * x + sin * z,
            cos + rcos * y * y,
            rcos * y * z - sin * x,
        );
        let r3 = Vector(
            rcos * z * x - sin * y,
            rcos * z * y + sin * x,
            cos + rcos * z * z,
        );
        let trans = Transform {
            r1, r2, r3, af: Vector(0.0, 0.0, 0.0),
        };
        trans * self
    }
    pub fn inverse(&self) -> Transform {
        let det = {
            self.r1.0 * self.r2.1 * self.r3.2 -
            self.r1.0 * self.r2.2 * self.r3.1 -
            self.r2.0 * self.r1.1 * self.r3.2 +
            self.r2.0 * self.r1.2 * self.r3.1 +
            self.r3.0 * self.r1.1 * self.r2.2 -
            self.r3.0 * self.r1.2 * self.r2.1

            //+ 4.0 * self.r1.0 * self.r2.1 * self.r3.2 -
            //self.r1.2 * self.r2.1 * self.r3.0
        };
        let r1 = Vector(
            (self.r2.1 * self.r3.2 - self.r3.1 * self.r2.2) / det,
            (self.r1.2 * self.r3.1 - self.r1.1 * self.r3.2) / det,
            (self.r1.1 * self.r2.2 - self.r1.2 * self.r2.1) / det,
        );
        let r2 = Vector(
            (self.r2.2 * self.r3.0 - self.r2.0 * self.r3.2) / det,
            (self.r1.0 * self.r3.2 - self.r1.2 * self.r3.0) / det,
            (self.r2.0 * self.r1.2 - self.r1.0 * self.r2.2) / det,
        );
        let r3 = Vector(
            (self.r2.0 * self.r3.1 - self.r2.1 * self.r3.0) / det,
            (self.r1.1 * self.r3.0 - self.r1.0 * self.r3.1) / det,
            (self.r1.0 * self.r2.1 - self.r2.0 * self.r1.1) / det,
        );
        let af = -self.af;
        Transform { r1, r2, r3, af }
    }

    pub fn to_cols(&self) -> (Vector, Vector, Vector) {
        let c1 = Vector(
            self.r1.0,
            self.r2.0,
            self.r3.0,
        );
        let c2 = Vector(
            self.r1.1,
            self.r2.1,
            self.r3.1,
        );
        let c3 = Vector(
            self.r1.2,
            self.r2.2,
            self.r3.2,
        );
        (c1, c2, c3)
    }
}
impl Mul<Point> for Transform {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        Point(
            self.r1.dot(rhs.to_vec()) + self.af.0,
            self.r2.dot(rhs.to_vec()) + self.af.1,
            self.r3.dot(rhs.to_vec()) + self.af.2,
        )
    }
}
impl Mul<Vector> for Transform {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Self::Output {
        Vector(
            self.r1.dot(rhs),
            self.r2.dot(rhs),
            self.r3.dot(rhs),
        )
    }
}
impl Mul<Ray> for Transform {
    type Output = Ray;
    fn mul(self, ray: Ray) -> Self::Output {
        let o = self * ray.o;
        let v = (self * ray.v).normalize();
        Ray { o, v }
    }
}
impl Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, rhs: Transform) -> Self::Output {
        let (c1, c2, c3) = rhs.to_cols();
        let e11 = self.r1.dot(c1);
        let e12 = self.r1.dot(c2);
        let e13 = self.r1.dot(c3);
        let e21 = self.r2.dot(c1);
        let e22 = self.r2.dot(c2);
        let e23 = self.r2.dot(c3);
        let e31 = self.r3.dot(c1);
        let e32 = self.r3.dot(c2);
        let e33 = self.r3.dot(c3);
        let r1 = Vector(e11, e12, e13);
        let r2 = Vector(e21, e22, e23);
        let r3 = Vector(e31, e32, e33);
        let af = Vector(
            self.r1.dot(rhs.af) + self.af.0,
            self.r2.dot(rhs.af) + self.af.1,
            self.r3.dot(rhs.af) + self.af.2,
        );
        Transform { r1, r2, r3, af }
    }
}


/// Cast a ray to the triangle and return the point of intersection if such
/// point exists.
#[inline]
pub fn ray_cast_tri(ray: &Ray, tri: &Triangle) -> Option<Intersection<Barycentric>> {
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
    // Barycentric coords.
    if let Some(bary) = Barycentric::new(&pos, tri) {
        // Distance from the ray origin to the triangle.
        let t = r1.abs();
        let kind = if r2 < 0.0 { HitKind::Front } else { HitKind::Back };
        let res = Intersection { attr: bary, kind, t };
        Some(res)
    } else {
        None
    }
}

/// Calculate the reflected direction of an incidental vector `i` about a normal
/// of `n`.
///
/// NOTE: `i` and `n` MUST be normalized.
#[inline]
pub fn reflect(i: Vector, n: Vector) -> Vector {
    -2.0 * (i - i.dot(n) * n)
}

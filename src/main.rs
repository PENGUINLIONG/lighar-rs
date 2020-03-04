mod geom;
mod rt;
mod scene;
mod model;

use geom::*;
use rt::*;
use scene::*;
use model::*;

#[derive(Default)]
struct PbrMaterial {
    albedo: Color,
    rough: f32,
    metal: f32,
    emit: Color,
}


struct DemoFramebuffer {
    w: u32,
    h: u32,
    buf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
}
impl DemoFramebuffer {
    pub fn new(w: u32, h: u32) -> DemoFramebuffer {
        let buf = image::ImageBuffer::new(w, h);
        DemoFramebuffer { w, h, buf }
    }
    pub fn save<P>(&self, path: P) -> image::ImageResult<()>
        where P: AsRef<std::path::Path>
    {
        self.buf.save(path)
    }
}
impl Framebuffer for DemoFramebuffer {
    type Color = Color;
    fn width(&self) -> u32 { self.w }
    fn height(&self) -> u32 { self.h }
    fn store(&mut self, x: u32, y: u32, color: Self::Color) {
        let color: [u8; 3] = color.into();
        self.buf.put_pixel(x, y, color.into());
    }
}

struct DebugPayload {
    refl_ray: Ray,
}


struct DemoRayTracer {
    s: Scene<PbrMaterial>,
    ambient: Color,
}
impl DemoRayTracer {
    pub fn new(s: Scene<PbrMaterial>, ambient: Color) -> DemoRayTracer {
        DemoRayTracer { s, ambient }
    }
}
impl RayTracer for DemoRayTracer {
    type Material = PbrMaterial;
    type Payload = i32; // Recursion count.
    type Ray = Ray;
    type RayAttr = Barycentric;
    type Color = Color;

    fn ray_gen(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Self::Color {
        let w = w as f32 / 2.0;
        let h = h as f32 / 2.0;
        let x = (x as f32 + 0.5) / w - 1.0;
        let y = (y as f32 + 0.5) / h - 1.0;
        let mut payload = Default::default();

        let mut rv = Default::default();
        let n = 2;
        let rn = (n as f32).recip();
        let rn2 = rn * rn;
        for i in 0..n {
            for j in 0..n {
                let ray = Ray {
                    o: Point(x + i as f32 * rn / w, y + j as f32 * rn / h, 0.0),
                    v: Vector(0.0, 0.0, 1.0),
                };
        
                rv = rv + self.trace(ray, &mut payload) * rn2;
            }
        }
        rv
    }
    fn intersect(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        mat: &Self::Material,
    ) -> Option<Intersection<Self::RayAttr>> {
        ray_cast_tri(&ray, tri)
    }
    fn any_hit(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
        mat: &Self::Material,
    ) -> bool {
        intersect.kind == HitKind::Front
        //true
    }
    fn miss(
        &self,
        ray: &Self::Ray,
        payload: &mut Self::Payload
    ) -> Self::Color {
        [0, 0, 0].into()
    }
    fn closest_hit(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
        mat: &Self::Material,
    ) -> Self::Color {
        let bary = intersect.attr;
        let u = bary.u;
        let v = bary.v;
        let w = 1.0 - u - v;
        let p = tri.o.affine_add(u * tri.x + v * tri.y);
        let refl = -reflect(ray.v, tri.n);
        let refl_ray = Ray {
            o: p,
            v: refl.normalize(),
        };
        //return [255, 0, 255].into();
        if *payload < 5 {
            *payload += 1;
            mat.emit + mat.albedo * (self.trace(refl_ray, payload) + self.ambient)
        } else {
            mat.emit + mat.albedo * self.ambient
        }
    }
    fn scene(&self) -> &Scene<PbrMaterial> {
        &self.s
    }
}

fn main() {
    let cam_trans = Transform::eye()
        .scale(Vector(0.5, 0.5, 0.5))
        .rotate((45.0 as f32).to_radians(), Vector(0.0, 1.0, 0.0))
        .rotate((45.0 as f32).to_radians(), Vector(1.0, 0.0, 0.0))
        .translate(Vector(0.0, 0.0, 1.0));
    let cube = make_cube(
        PbrMaterial {
            albedo: [255, 228, 0].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .translate(Vector(-1.0, 0.0, 0.0)),
    );
    let cube2 = make_cube(
        PbrMaterial {
            albedo: [254, 228, 0].into(),
            emit: [68, 228, 254].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .translate(Vector(0.75, 0.0, 0.0))
            .rotate((30.0_f32).to_radians(), Vector(1.0, 1.0, 0.0)),
    );
    let scene = Scene {
        objs: vec![cube, cube2],
    };
    let mut framebuf = DemoFramebuffer::new(256, 256);
    let albedo = [104, 100, 70].into();
    let rt = DemoRayTracer::new(scene, albedo);
    rt.draw(&mut framebuf);
    framebuf.save("1.bmp").unwrap();
}

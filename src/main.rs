mod geom;
mod rt;
mod scene;
mod model;

use geom::*;
use rt::*;
use scene::*;
use model::*;

struct PbrMaterial {
    albedo: Color,
    rough: f32,
    metal: f32,
    emit: (f32, Color),
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

#[derive(Default)]
struct DebugPayload {
    small_t: bool,
}


struct DemoRayTracer {
}
impl DemoRayTracer {
    pub fn new() -> DemoRayTracer {
        DemoRayTracer {}
    }
}
impl RayTracer for DemoRayTracer {
    type Material = ();
    type Payload = DebugPayload;
    type Ray = Ray;
    type RayAttr = Barycentric;
    type Color = Color;

    fn ray_gen(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        scene: &Scene<Self::Material>,
    ) -> Self::Color {
        let w = w as f32 / 2.0;
        let h = h as f32 / 2.0;
        let x = (x as f32 + 0.5) / w - 1.0;
        let y = (y as f32 + 0.5) / h - 1.0;
        let ray = Ray {
            o: Point(x, y, 0.0),
            v: Vector(0.0, 0.0, 1.0),
        };
        let mut payload = Default::default();

        self.trace(ray, &mut payload, scene)
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
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
    ) -> bool {
        if (intersect.t < 1e-2) {
            payload.small_t = true;
        }
        intersect.kind == HitKind::Front
    }
    fn miss(&self, payload: &mut Self::Payload) -> Self::Color {
        [255, 228, 0].into()
    }
    fn closest_hit(
        &self,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
    ) -> Self::Color {
        let u = intersect.attr.u;
        let v = intersect.attr.v;
        let w = 1.0 - u - v;
        
        if payload.small_t {
            [0, 0, 255].into()
        } else if u < 2e-2 || v < 2e-2 || w < 2e-2 {
            [65, 65, 65].into()
        } else {
            [255, 255, 255].into()
        }
    }
}

fn main() {
    let cube = make_cube(
        (),
        Transform::eye()
            .scale(Vector(0.5, 0.5, 0.5))
            .rotate((45.0 as f32).to_radians(), Vector(0.0, 1.0, 0.0))
            .rotate((45.0 as f32).to_radians(), Vector(1.0, 0.0, 0.0))
            .translate(Vector(0.0, 0.0, 1.0)),
    );
    let scene = Scene {
        objs: vec![cube],
    };
    let mut framebuf = DemoFramebuffer::new(256, 256);
    let rt = DemoRayTracer::new();
    rt.draw(&scene, &mut framebuf);
    framebuf.save("1.bmp").unwrap();
}

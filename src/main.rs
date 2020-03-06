mod geom;
mod rt;
mod scene;
mod model;
mod img;
mod sampler;

use geom::*;
use rt::*;
use scene::*;
use model::*;
use img::*;
use sampler::*;

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
    skybox: Vec<Image>,
    skybox_samp: CubeSampler,
    counter: std::cell::RefCell<usize>,
}
impl DemoRayTracer {
    pub fn new(s: Scene<PbrMaterial>, ambient: Color, skybox: Vec<Image>) -> DemoRayTracer {
        let skybox_samp = CubeSampler::default();
        debug_assert!(skybox_samp.validate(&skybox),
            "sampled image failed to meet the sampler's requirement");
        let counter = std::cell::RefCell::new(0);
        DemoRayTracer { s, ambient, skybox, skybox_samp, counter }
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
        let id = x * h + y;
        let w = w as f32 / 2.0;
        let h = h as f32 / 2.0;
        let x = (x as f32) / w - 1.0;
        let y = (y as f32) / h - 1.0;

        let n = 3;
        let rn = (n as f32).recip();
        let rn2 = rn * rn;
        let rv: Color = (0..n).into_iter()
            .fold(Color::default(), |seed, i| {
                seed + (0..n).into_iter()
                    .fold(Color::default(), |seed, j| {
                        let ray = Ray {
                            o: Point(
                                x + i as f32 * rn / w,
                                y + j as f32 * rn / h,
                                0.0
                            ),
                            v: Vector(0.0, 0.0, 10.0),
                        };
                        let mut payload = Default::default();

                        let tic = std::time::Instant::now();
                        let cur = self.trace(ray, &mut payload);
                        println!("traced ray for pixel #{} in {}s",
                            id,
                            tic.elapsed().as_millis() as f64 / 1000.0);
                        seed + cur
                    })
            });
        rv * ((n * n) as f32).recip()
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
    }
    fn miss(
        &self,
        ray: &Self::Ray,
        payload: &mut Self::Payload
    ) -> Self::Color {
        //let vec = Vector(ray.o.0, ray.o.1, ray.o.2 + 1.0).normalize();
        //self.skybox_samp.sample(&self.skybox, vec)
        *self.counter.borrow_mut() += 1;
        self.ambient
    }
    fn closest_hit(
        &self,
        ray: &Self::Ray,
        tri: &Triangle,
        intersect: &Intersection<Self::RayAttr>,
        payload: &mut Self::Payload,
        mat: &Self::Material,
    ) -> Self::Color {
        // Number of extra rays to trace from this intersection.
        const NRAY: usize = 32;
        const F0: f32 = 0.04;

        let bary = intersect.attr;
        let p = tri.o.affine_add(bary.u * tri.x + bary.v * tri.y);
        let refl = -reflect(ray.v, tri.n);
        let refl_ray = Ray {
            o: p,
            v: refl.normalize(),
        };

        if *payload < 3 {
            *payload += 1;

            // Lighting.
            let specular = self.trace(refl_ray, payload);
            let diffuse = {
                let n = tri.n;
                let u = tri.y.normalize();
                let v = n.cross(u);
                let mut temp = Color::default();
                for _ in 0..NRAY {
                    use std::f32::consts::PI;
                    let lon = (rand::random::<f32>() - 0.5) * 2.0 * PI;
                    let lat = (rand::random::<f32>() - 0.5) * PI;
                    let dir = (u * lon.cos() + v * lon.sin()) * lat.sin() + n * lat.cos();
                    let diffuse_ray = Ray { o: p, v: dir.normalize() };
                    let mut payload2 = *payload;
                    temp = temp + self.trace(diffuse_ray, &mut payload2);
                }
                temp * (NRAY as f32).recip()
            };

            mat.emit + mat.albedo * (diffuse + specular * F0)
        } else {
            *self.counter.borrow_mut() += 1;
            mat.emit + self.ambient
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
            albedo: [245, 228, 0].into(),
            emit: [245, 228, 0].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .translate(Vector(-1.0, 0.75, 0.0)),
    );
    let cube2 = make_cube(
        PbrMaterial {
            albedo: [68, 228, 235].into(),
            emit: [68, 228, 235].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .translate(Vector(0.75, 0.0, 0.0))
            .rotate((15.0_f32).to_radians(), Vector(1.0, 1.0, 0.0).normalize())
            .translate(Vector(0.0, -1.0, 0.25)),
    );
    let cube3 = make_cube(
        PbrMaterial {
            albedo: [235, 54, 72].into(),
            emit: [235, 54, 72].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .translate(Vector(-1.0, -0.75, 0.0)),
    );
    let floor = make_pln(
        PbrMaterial {
            albedo: [255, 255, 255].into(),
            //emit: [40, 40, 40].into(),
            ..Default::default()
        },
        cam_trans * Transform::eye()
            .scale(Vector(15.0, 15.0, 15.0))
            .translate(Vector(0.0, 1.5, 0.0)),
    );

    let scene = Scene {
        objs: vec![cube, cube2, cube3, floor],
    };
    let mut framebuf = DemoFramebuffer::new(64, 64);
    let ambient = [50, 50, 50].into();
    let skybox = load_skybox();
    let rt = DemoRayTracer::new(scene, ambient, skybox);
    let tic = std::time::Instant::now();
    rt.draw(&mut framebuf);
    println!("traced {} rays in {}s",
        rt.counter.borrow(),
        tic.elapsed().as_millis() as f64 / 1000.0);
    framebuf.save("1.bmp").unwrap();
}

fn load_img<P: AsRef<std::path::Path>>(p: P) -> Image {
    image::io::Reader::open(p).unwrap()
        .decode().unwrap()
        .into()
}
fn load_skybox() -> Vec<Image> {
    [
        "./skybox/pos-x.png",
        "./skybox/neg-x.png",
        "./skybox/pos-y.png",
        "./skybox/neg-y.png",
        "./skybox/pos-z.png",
        "./skybox/neg-z.png",
    ].into_iter()
        .map(|x| load_img(x))
        .collect()
}

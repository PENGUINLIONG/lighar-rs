use crate::geom::{Point, Transform};
use crate::scene::Object;

pub fn make_cube<M>(mat: M, world2obj: Transform) -> Object<M> {
    let obj2world = world2obj.inverse();
    const p: f32 = 0.5;
    const n: f32 = -0.5;
    let verts = vec![
        Point(n,p,n),
        Point(n,p,p),
        Point(p,p,p),
        Point(p,p,n),
        Point(n,n,n),
        Point(n,n,p),
        Point(p,n,p),
        Point(p,n,n),
    ];
    const a: usize = 0;
    const b: usize = 1;
    const c: usize = 2;
    const d: usize = 3;
    const e: usize = 4;
    const f: usize = 5;
    const g: usize = 6;
    const h: usize = 7;
    let idxs = vec![
        (f, e, a), (f, a, b),
        (g, f, b), (g, b, c),
        (h, g, c), (h, c, d),
        (e, h, d), (e, d, a),
        (a, d, c), (a, c, b),
        (e, f, g), (e, g, h),
    ];
    Object { verts, idxs, mat, obj2world, world2obj }
}

pub fn make_pln<M>(mat: M, world2obj: Transform) -> Object<M> {
    let obj2world = world2obj.inverse();
    let verts = vec![
        Point(-0.5, 0.0, -0.5),
        Point(-0.5, 0.0, 0.5),
        Point(0.5, 0.0, 0.5),
        Point(0.5, 0.0, -0.5),
    ];
    let idxs = vec![
        (0, 1, 2), (0, 2, 3),
    ];
    Object { verts, idxs, mat, obj2world, world2obj }
}

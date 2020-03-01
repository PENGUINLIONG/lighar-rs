use crate::geom::{Point, Transform};

pub struct Object<Material> {
    pub verts: Vec<Point>,
    pub idxs: Vec<(usize, usize, usize)>,
    pub mat: Material,
    pub obj2world: Transform,
    pub world2obj: Transform,
}

pub struct Scene<Material> {
    pub objs: Vec<Object<Material>>,
}

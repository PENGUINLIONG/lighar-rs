use crate::geom::Point;
pub struct Object<Material> {
    pub verts: Vec<Point>,
    pub idxs: Vec<(usize, usize, usize)>,
    pub mat: Material,
}

pub struct Scene<Material> {
    pub objs: Vec<Object<Material>>,
}

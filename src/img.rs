use crate::geom::{Color, Vector};

pub struct Image {
    buf: Vec<Color>,
    w: usize,
    h: usize,
}
impl Image {
    pub fn new(w: usize, h: usize) -> Image {
        let buf = std::iter::repeat(Color::default())
            .take(w * h)
            .collect();
        Image { buf, w, h }
    }

    // The dimension data are seldom used directly but quite frequently
    // multiplied up to calculate pixel offsets in pixel load/store; so we store
    // the relative dimensions instead.
    #[inline]
    pub fn width(&self) -> usize { self.w }
    #[inline]
    pub fn height(&self) -> usize { self.h }

    #[inline]
    fn coords2offset(&self, x: usize, y: usize) -> usize {
        x + self.w * y
    }
    #[inline]
    pub fn load_px(&self, x: usize, y: usize) -> Color {
        let i = self.coords2offset(x, y);
        self.buf[i]
    }
    #[inline]
    pub fn store_px(&mut self, x: usize, y: usize, c: Color) {
        let i = self.coords2offset(x, y);
        self.buf[i] = c;
    }
}
impl From<Image> for image::RgbaImage {
    fn from(img: Image) -> image::RgbaImage {
        let mut buf = Vec::with_capacity(4 * img.buf.len());
        let w = img.width() as u32;
        let h = img.height() as u32;
        for c in img.buf.into_iter() {
            let c: [u8; 4] = c.into();
            buf.extend(&c);
        }
        image::RgbaImage::from_raw(w, h, buf)
            .unwrap()
    }
}
impl From<image::DynamicImage> for Image {
    fn from(img: image::DynamicImage) -> Image {
        use image::GenericImageView;
        let w = img.width() as usize;
        let h = img.height() as usize;
        let buf = img.into_rgba()
            .chunks_exact(4)
            .map(|x| [x[0], x[1], x[2], x[3]].into())
            .collect::<Vec<Color>>();
        Image { buf, w, h }
    }
}

use sdl2::rect::Rect;
use sdl2::render::Texture;

pub struct Sprite<'a> {
    texture: &'a Texture<'a>,
    frame: u32,
    frame_len: u32,
    size: (u32, u32),
    nrow: u32,
    ncol: u32,
}

impl<'a> Sprite<'a> {
    pub fn new(texture: &'a Texture, size: (u32, u32)) -> Self {
        let (sprite_h, sprite_w) = size;
        let h = texture.query().height;
        let w = texture.query().width;
        let nrow = h / sprite_h;
        let ncol = w / sprite_w;
        Self {
            texture,
            frame: 0,
            frame_len: nrow * ncol,
            size,
            nrow,
            ncol,
        }
    }
    pub fn next_frame(&mut self) -> (&Texture, Rect) {
        let i = self.frame;
        let (size_x, size_y) = self.size;
        let row = if self.nrow == 1 {
            0
        } else {
            ((i / self.nrow) as u32 * size_x) as i32
        };
        let col = ((i % self.ncol) as u32 * size_y) as i32;
        let rect = Rect::new(col, row, size_x, size_y);
        self.frame += 1;
        self.frame %= self.frame_len;
        (self.texture, rect)
    }
}

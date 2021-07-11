use std::collections::HashMap;

use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

pub struct Sprite<'a> {
    texture: Texture<'a>,
    frame: u32,
    frame_len: u32,
    size: (u32, u32),
    nrow: u32,
    ncol: u32,
}

impl<'a> Sprite<'a> {
    pub fn new(texture: Texture<'a>, size: (u32, u32)) -> Self {
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
        (&self.texture, rect)
    }
}
pub struct SpriteConfig {
    pub path: &'static str,
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}
pub struct SpriteManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    sprites: HashMap<String, Sprite<'a>>,
}

impl<'a> SpriteManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            sprites: HashMap::new(),
        }
    }
    pub fn add(&mut self, config: SpriteConfig) {
        let texture = self.texture_creator.load_texture(config.path).unwrap();
        let sprite = Sprite::new(texture, (config.width, config.height));
        self.sprites.insert(config.name.into(), sprite);
    }
    pub fn get(&mut self, name: &str) -> Option<&mut Sprite<'a>> {
        self.sprites.get_mut(name)
    }
}

use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;
use std::time::Duration;

use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use crate::aesprite::AespriteJSON;

#[derive(Debug)]
pub struct SpriteState {
    json: AespriteJSON,
    current_frame: usize,
    time: Duration,
}
pub struct Sprite<'a> {
    texture: Texture<'a>,
    state: SpriteState,
}

impl From<&str> for SpriteState {
    fn from(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let json = serde_json::from_reader(reader).unwrap();
        dbg!(Self {
            json,
            current_frame: 0,
            time: Duration::ZERO
        })
    }
}

impl<'a> Sprite<'a> {
    pub fn from_aesprite(texture: Texture<'a>, aes_config_path: &'static str) -> Self {
        let state = SpriteState::from(aes_config_path);
        Self { texture, state }
    }
    pub fn next_frame(&mut self, elapsed: Duration) -> (&Texture, Rect) {
        let frame = self.state.current_frame;
        let json = &self.state.json;
        let frame_info = &json.frames[frame];
        self.state.time += elapsed;
        // println!("{:?} {} [{:?}]", elapsed, frame, frame_info);
        // Buggy
        if self.state.time.as_millis() > frame_info.duration {
            self.state.current_frame += 1;
            self.state.current_frame %= 5;
            self.state.time = Duration::ZERO;
            self.next_frame(Duration::ZERO)
        } else {
            (&self.texture, frame_info.frame.into())
        }
    }
}
pub struct SpriteConfig {
    pub path: &'static str,
    pub name: &'static str,
    pub json: &'static str,
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
        let sprite = Sprite::from_aesprite(texture, config.json);
        self.sprites.insert(config.name.into(), sprite);
    }
    pub fn get(&mut self, name: &str) -> Option<&mut Sprite<'a>> {
        self.sprites.get_mut(name)
    }
}

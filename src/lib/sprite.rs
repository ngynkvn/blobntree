use crate::ecs::Component;
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

pub type SpriteIndex = usize;

#[derive(Debug)]
pub struct SpriteState {
    sprite: &'static str,
    current_frame: usize,
    time: Duration,
}

pub struct Sprite<'a> {
    texture: Texture<'a>,
    json: AespriteJSON,
}

impl<'a> std::fmt::Debug for Sprite<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.write_str(&format!("{:?}", self.json))
    }
}

impl<'a> From<&'static str> for SpriteState {
    fn from(sprite: &'static str) -> Self {
        Self {
            sprite,
            current_frame: 0,
            time: Duration::ZERO,
        }
    }
}


impl<'a> Sprite<'a> {
    pub fn from_aesprite(texture: Texture<'a>, aes_config_path: &'static str) -> Self {
        let file = File::open(aes_config_path).unwrap();
        let reader = BufReader::new(file);
        let json = serde_json::from_reader(reader).unwrap();
        Self { texture, json }
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
    instances: Vec<SpriteState>,
}

impl<'a> SpriteManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            sprites: HashMap::new(),
            instances: Vec::new(),
        }
    }
    pub fn add(&mut self, config: SpriteConfig) {
        let texture = self.texture_creator.load_texture(config.path).unwrap();
        let sprite = Sprite::from_aesprite(texture, config.json);
        self.sprites.insert(config.name.into(), sprite);
    }

    pub fn next_frame(&mut self, index: SpriteIndex, elapsed: Duration) -> (&Texture, Rect) {
        let state = &mut self.instances[index];
        let sprite = &self.sprites.get(state.sprite).unwrap();
        let frame = state.current_frame;
        let json = &sprite.json;
        let frame_info = &json.frames[frame];
        state.time += elapsed;
        // println!("{:?} {} [{:?}]", elapsed, frame, frame_info);
        // Buggy
        if state.time.as_millis() > frame_info.duration {
            state.current_frame += 1;
            state.current_frame %= 5;
            state.time = Duration::ZERO;
        }
        (&sprite.texture, frame_info.frame.into())
    }

    pub fn init(&mut self, name: &'static str) -> SpriteIndex {
        self.instances.push(SpriteState::from(name));
        self.instances.len() - 1
    }
    pub fn get(&mut self, name: &str) -> Option<&Sprite<'a>> {
        self.sprites.get(name)
    }
    pub fn get_instance(&mut self, index: SpriteIndex) -> &mut SpriteState {
        &mut self.instances[index]
    }
    pub fn take(&mut self, name: &str) -> Sprite<'a> {
        self.sprites.remove(name).unwrap()
    }
}

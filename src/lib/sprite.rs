use crate::aseprite::SpriteRect;
use crate::aseprite::Tags;
use crate::opengl::texture2d::Texture2D;
use crate::systems::components::SpriteHandle;
use crate::systems::input::InputState;
use crate::systems::renderer::TextureInfo;

use std::collections::HashMap;

use std::time::Duration;

pub type SpriteIndex = usize;

#[derive(Debug)]
pub struct SpriteState {
    pub state: InputState,
    pub tag: Option<Tags>,
    pub frame_i: usize,
    sprite: &'static str,
    time: Duration,
    pub texture: u32,
}

#[derive(Debug)]
pub struct Sprite {
    pub texture: Option<Texture2D>,
    info: TextureInfo,
}

#[derive(Debug)]
pub struct SpriteConfig {
    pub name: &'static str,
    pub path: &'static str,
    pub json: &'static str,
}

impl<'a> From<&'static str> for SpriteState {
    fn from(sprite: &'static str) -> Self {
        Self {
            sprite,
            tag: None,
            frame_i: 0,
            time: Duration::ZERO,
            state: InputState::Idle,
            texture: 0,
        }
    }
}

impl<'a> From<TextureInfo> for Sprite {
    fn from(info: TextureInfo) -> Self {
        Self {
            texture: None,
            info,
        }
    }
}

pub struct SpriteManager {
    sprites: HashMap<String, Sprite>,
    loaded_textures: Vec<TextureInfo>,
    pub instances: Vec<SpriteState>,
}

pub struct SpriteQuery {
    size: (usize, usize),
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            loaded_textures: Vec::new(),
            instances: Vec::new(),
        }
    }
    pub fn add(&mut self, config: SpriteConfig) {
        // let texture = self.texture_creator.load_texture(config.path).unwrap();
        let name = config.name.clone();
        let info = TextureInfo::from(config);
        let sprite = Sprite::from(info.clone());
        self.sprites.insert(name.to_string(), sprite);
        self.loaded_textures.push(info);
    }

    pub fn signal(&mut self, handle: &SpriteHandle, signal: Option<&InputState>) {
        if let Some(input_state) = signal {
            let state = &mut self.instances[handle.index];
            if &state.state != input_state {
                state.state = *input_state;
                let sprite = self.sprites.get_mut(state.sprite).unwrap();
                let json = sprite.info.json.as_ref().unwrap();
                let tags = &json.meta.frame_tags;
                match state.state {
                    InputState::Running => {
                        if let Some(tag) = tags.iter().find(|tag| tag.name == "run") {
                            println!("{:?}", tag);
                            state.tag.replace(tag.clone());
                            state.frame_i = tag.from;
                        }
                    }
                    InputState::Idle => {
                        if let Some(tag) = tags.iter().find(|tag| tag.name == "still") {
                            state.tag.replace(tag.clone());
                            state.frame_i = tag.from;
                        }
                    }
                }
            }
        }
    }

    pub fn next_frame(
        &mut self,
        handle: &SpriteHandle,
        elapsed: Duration,
    ) -> (&Texture2D, SpriteRect) {
        let state = &mut self.instances[handle.index];
        let sprite = self.sprites.get_mut(state.sprite).unwrap();
        let frame = state.frame_i;
        let json = sprite.info.json.as_ref().unwrap();
        let frame_info = &json.frames[frame];
        let (from, to) = state
            .tag
            .as_ref()
            .map(|t| (t.from, t.to))
            .unwrap_or((0, json.frames.len()));
        state.time += elapsed;
        if state.time.as_millis() > frame_info.duration {
            state.frame_i += 1;
            if state.frame_i >= to {
                state.frame_i = from;
            }
            state.frame_i %= json.frames.len();
            state.time = Duration::ZERO;
        }
        if sprite.texture.is_none() {
            sprite.texture = Some(Texture2D::from(sprite.info.clone()));
        }
        (sprite.texture.as_ref().unwrap(), frame_info.frame)
    }

    // Creates a sprite handle.
    pub fn init(&mut self, name: &'static str) -> SpriteHandle {
        let state = SpriteState::from(name);
        let SpriteQuery {
            size: (width, height),
        } = self.query(&state);
        self.instances.push(state);
        SpriteHandle {
            index: self.instances.len() - 1,
            width,
            height,
            scale: 2,
        }
    }

    pub fn query(&mut self, state: &SpriteState) -> SpriteQuery {
        let sprite = &self.sprites.get(state.sprite).unwrap();
        let json = sprite.info.json.as_ref().unwrap();
        let meta = &json.frames[0];
        SpriteQuery {
            size: (meta.source_size.w, meta.source_size.h),
        }
    }
    pub fn get(&mut self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }
    pub fn get_instance(&mut self, index: SpriteIndex) -> &mut SpriteState {
        &mut self.instances[index]
    }
    pub fn take(&mut self, name: &str) -> Sprite {
        self.sprites.remove(name).unwrap()
    }
}

impl Drop for SpriteManager {
    fn drop(&mut self) {
        let mut texture_ids = vec![];
        for sprite in self.sprites.values() {
            // There is an allocated texture here we need to delete.
            if let Some(texture) = &sprite.texture {
                texture_ids.push(texture.id);
            }
        }
        unsafe {
            eprintln!("Deleting {} textures from OpenGL.", texture_ids.len());
            gl::DeleteTextures(texture_ids.len() as i32, texture_ids.as_ptr());
        }
    }
}

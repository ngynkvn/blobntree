use crate::aseprite::Tags;

use crate::systems::components::SpriteHandle;
use crate::systems::input::InputState;
use crate::systems::renderer::TextureInfo;
use std::marker::PhantomData;

use std::collections::HashMap;

use std::time::Duration;

pub type SpriteIndex = usize;

#[derive(Debug)]
pub struct StateMachine<T> {
    pub tags: Vec<Tags>,
    _t: PhantomData<T>,
}

impl StateMachine<InputState> {
    fn run(&mut self, object: InputState) -> Result<&Tags, &str> {
        println!("You found my state machine!");
        match object {
            InputState::Idle => self
                .tags
                .iter()
                .find(|t| t.name == "still")
                .ok_or("Still tag not found"),
            InputState::Running => self
                .tags
                .iter()
                .find(|t| t.name == "run")
                .ok_or("Run tag not found"),
        }
    }
}

#[derive(Debug)]
pub struct SpriteState {
    pub state: InputState,
    pub tag: Option<Tags>,
    pub frame_i: usize,
    sprite: &'static str,
    time: Duration,
    pub texture: u32,
    pub state_machine: Option<StateMachine<InputState>>,
}

use color_eyre::Result;
use glium::texture::Texture2dArray;
use glium::uniforms::MagnifySamplerFilter;
use glium::uniforms::MinifySamplerFilter;
use glium::uniforms::Sampler;
use glium::uniforms::SamplerWrapFunction;

trait Ack<T> {
    fn ack(&mut self, object: T) -> Result<(), &str>;
}

impl Ack<InputState> for SpriteState {
    fn ack(&mut self, object: InputState) -> Result<(), &str> {
        if self.state == object {
            // Force idempotence on input state object
            // Ignore states that do not cause change in internal state.
            return Ok(());
        } else {
            self.state = object;
        }
        if self.state_machine.is_none() {
            return Err("The current object does not have an associated state machine with it..");
        }
        match self.state_machine.as_mut().unwrap().run(object) {
            Ok(tag) => {
                println!("{:?}", tag);
                self.tag.replace(tag.clone());
                self.frame_i = tag.from;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct Sprite {
    pub texture: Texture2dArray,
    pub info: TextureInfo,
}

impl Sprite {
    pub fn sampler(&self) -> Sampler<Texture2dArray> {
        self.texture
            .sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .wrap_function(SamplerWrapFunction::Clamp)
    }
}

#[derive(Debug)]
pub struct SpriteConfig {
    pub name: &'static str,
    pub path: &'static str,
    pub json: &'static str,
}

impl<'a> From<&'a Sprite> for SpriteState {
    fn from(sprite: &Sprite) -> Self {
        let mut state = Self {
            sprite: sprite.info.name,
            tag: None,
            frame_i: 0,
            time: Duration::ZERO,
            state: InputState::Idle,
            texture: 0,
            state_machine: None,
        };
        if let Some(tags) = sprite
            .info
            .json
            .as_ref()
            .map(|json| json.meta.frame_tags.clone())
        {
            let sm = StateMachine::<InputState> {
                tags: tags.to_vec(),
                _t: PhantomData,
            };
            state.state_machine.replace(sm);
        }
        state
    }
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
            state_machine: None,
        }
    }
}

pub struct SpriteManager {
    sprites: HashMap<String, Sprite>,
    pub loaded_textures: Vec<Sprite>,
    pub instances: Vec<SpriteState>,
}

pub struct SpriteQuery {
    size: (usize, usize),
}

type SpriteInfo = ((f32, f32), (f32, f32));

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            loaded_textures: Vec::new(),
            instances: Vec::new(),
        }
    }
    pub fn add(&mut self, sprite: Sprite) {
        self.sprites.insert(sprite.info.name.to_string(), sprite);
        // self.loaded_textures.push(sprite);
    }

    pub fn add_sprites(&mut self, sprites: Vec<Sprite>) {
        for sprite in sprites {
            self.add(sprite);
        }
    }

    pub fn signal(&mut self, handle: &SpriteHandle, signal: Option<&InputState>) {
        if let Some(input_state) = signal {
            let state = &mut self.instances[handle.index];
            state.ack(*input_state).unwrap();
        }
    }

    pub fn next_frame(&mut self, handle: &SpriteHandle, elapsed: Duration) -> (&Sprite, usize) {
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
        (sprite, state.frame_i)
    }

    // Creates a sprite handle.
    pub fn init(&mut self, name: &'static str) -> SpriteHandle {
        let sprite = self.sprites.get(name).unwrap();
        let state = SpriteState::from(sprite);
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
}

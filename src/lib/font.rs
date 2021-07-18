use crate::Color;
use crate::HashMap;
use crate::Texture;
use crate::TextureCreator;

use crate::WindowContext;
use sdl2::ttf::Font;
use sdl2::ttf::FontStyle;
use sdl2::ttf::Sdl2TtfContext;

pub struct FontConfig {
    pub path: &'static str,
    pub size: u16,
    pub style: FontStyle,
}
pub struct FontManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    ttf_context: &'a Sdl2TtfContext,
    fonts: HashMap<String, Font<'a, 'a>>,
}

impl<'a> FontManager<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a Sdl2TtfContext,
    ) -> Self {
        Self {
            texture_creator,
            ttf_context,
            fonts: HashMap::new(),
        }
    }
    pub fn add(&mut self, config: FontConfig) {
        let mut font = self
            .ttf_context
            .load_font(config.path, config.size)
            .unwrap();
        font.set_style(config.style);
        self.fonts.insert(config.path.into(), font);
    }
    pub fn render(&mut self, font: &str, text: &str) -> Option<Texture<'a>> {
        self.fonts.get(font).map(|font| {
            let surface = font
                .render(text)
                .blended(Color::RGBA(255, 255, 255, 255))
                .unwrap();
            self.texture_creator
                .create_texture_from_surface(&surface)
                .unwrap()
        })
    }
}

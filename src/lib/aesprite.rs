use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SpriteRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrameInfo {
    pub filename: String,
    pub frame: SpriteRect,
    pub duration: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tags {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub direction: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SpriteSheetSize {
    pub w: usize,
    pub h: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaInfo {
    pub size: SpriteSheetSize,
    #[serde(rename = "frameTags")]
    pub frame_tags: Vec<Tags>,
}

impl From<SpriteRect> for Rect {
    fn from(SpriteRect { x, y, w, h }: SpriteRect) -> Self {
        Self::new(x, y, w, h)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AespriteJSON {
    pub frames: Vec<FrameInfo>,
    pub meta: MetaInfo,
}

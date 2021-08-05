use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SpriteRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameInfo {
    pub filename: String,
    pub frame: SpriteRect,
    pub duration: u128,
    #[serde(rename = "sourceSize")]
    pub source_size: SpriteSheetSize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tags {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub direction: String,
}
impl Display for Tags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "[{}] ({} -> {})", self.name, self.from, self.to)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpriteSheetSize {
    pub w: usize,
    pub h: usize,
}

impl Display for SpriteSheetSize {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}x{}", self.w, self.h)
    }
}

impl Display for SpriteRect {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}x{}", self.w, self.h)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct AsepriteJSON {
    pub frames: Vec<FrameInfo>,
    pub meta: MetaInfo,
}

use std::fmt;

impl Debug for AsepriteJSON {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut table = table!(["filename", "frame", "duration", "sourceSize"]);
        for f in &self.frames {
            table.add_row(row![f.filename, f.frame, f.duration, f.source_size]);
        }
        let mut meta = table!();
        meta.add_row(row!["meta"]);
        meta.add_row(row!["size", self.meta.size]);
        for t in &self.meta.frame_tags {
            meta.add_row(row![t.name, t]);
        }
        table.add_row(row![meta]);

        // meta.add_row(row!["tags", self.meta.frame_tags]);
        write!(fmt, "\n{}", table)
    }
}

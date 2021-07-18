use crate::systems::renderer::TextureInfo;

#[derive(Debug, Clone)]
pub struct Texture2D {
    pub id: u32,
    pub info: Option<TextureInfo>,
}

impl Texture2D {
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
    fn init_texture(id: &mut u32, fp: &'static str) {
        unsafe {
            // Open image from file path, convert to rgba8 format
            let image = image::open(fp);
            let image = image
                .unwrap_or_else(|_| panic!("Cannot find {}", fp))
                .to_rgba8();
            let (width, height) = image.dimensions();
            let image = image.into_raw();
            let image = image.as_ptr();

            // Reset unpack alignment to be default again.
            // TODO -- Remove SDL2, add a new window / input system and rely only on opengl for rendering.
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
            // generate and unpack texture.
            gl::GenTextures(1, id);
            gl::BindTexture(gl::TEXTURE_2D, *id);
            gl::TexImage2D(
                gl::TEXTURE_2D,    // target
                0,                 // level
                gl::RGBA as _,     // internalformat
                width as _,        // width
                height as _,       // height
                0,                 // border
                gl::RGBA,          // format
                gl::UNSIGNED_BYTE, // type
                image as *const _, // data
            );
            println!("Uploaded a texture to {}", id);
            assert_eq!(gl::GetError(), 0);
            // Set texture properties
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            // Unbind
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl From<TextureInfo> for Texture2D {
    fn from(info: TextureInfo) -> Self {
        let path = &info.path;
        let mut id = 0;
        Texture2D::init_texture(&mut id, info.path);
        assert_ne!(id, 0);
        println!(
            "Texture created from {}, [OpenGL] Allocated ID: {}",
            path, id
        );
        println!("{:?}", &info);
        Texture2D {
            id,
            info: Some(info),
        }
    }
}

impl From<&'static str> for Texture2D {
    // Allocate OpenGL texture from file path.
    fn from(fp: &'static str) -> Self {
        let mut texture = Texture2D { id: 0, info: None };
        Texture2D::init_texture(&mut texture.id, fp);
        assert_ne!(texture.id, 0);
        println!(
            "Texture created from {}, [OpenGL] Allocated ID: {}",
            fp, texture.id
        );
        texture
    }
}

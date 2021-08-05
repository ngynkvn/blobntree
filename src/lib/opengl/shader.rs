use crate::HashMap;
use std::ffi::CString;

lazy_static! {
    /**
     * Shader programs compiled at runtime
     */
    pub static ref SHADERS: HashMap<&'static str, Shader> = {
        println!("INIT LAZY");
        let mut shaders = HashMap::new();
        shaders.insert(
            "world",
            Shader::from(("graphics/world.vert", "graphics/world.frag")),
        );
        shaders.insert(
            "test shader",
            Shader::from(("graphics/test.vert", "graphics/test.frag")),
        );
        shaders
    };
}

macro_rules! check_errors {
    ($id: expr, COMPILE_STATUS) => {{
        let mut len = 0;
        let mut success = 1;
        gl::GetShaderiv($id, gl::COMPILE_STATUS, &mut success);
        dbg!(success);
        if success == 0 {
            gl::GetShaderiv($id, gl::INFO_LOG_LENGTH, &mut len);
            let len = len as usize;
            dbg!(len);
            let mut info: Vec<u8> = Vec::with_capacity(len as usize + 1);
            info.extend([b' '].iter().cycle().take(len as usize));
            let error: CString = CString::from_vec_unchecked(info);
            gl::GetShaderInfoLog($id, 1024, std::ptr::null_mut(), error.as_ptr() as *mut _);
            panic!("===Shader Compile Error===\n{}", error.to_string_lossy());
        }
    }};

    ($id: expr, LINK_STATUS) => {{
        let mut len = 0;
        let mut success = 1;
        gl::GetProgramiv($id, gl::LINK_STATUS, &mut success);
        dbg!(success);
        if success == 0 {
            gl::GetProgramiv($id, gl::INFO_LOG_LENGTH, &mut len);
            let len = len as usize;
            dbg!(len);
            let mut info: Vec<u8> = Vec::with_capacity(len as usize + 1);
            info.extend([b' '].iter().cycle().take(len as usize));
            let error: CString = CString::from_vec_unchecked(info);
            gl::GetProgramInfoLog($id, 1024, std::ptr::null_mut(), error.as_ptr() as *mut _);
            panic!("===Shader Link Error==={:?}", error);
        }
    }};
}

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn compile(vert_code: String, frag_code: String) -> Shader {
        let mut shader = Shader { id: 0 };
        unsafe {
            let s_vertex = gl::CreateShader(gl::VERTEX_SHADER);
            dbg!(s_vertex);
            let vert_code = CString::new(vert_code).unwrap();
            gl::ShaderSource(
                s_vertex,
                1,
                &vert_code.as_ptr() as *const _,
                std::ptr::null(),
            );
            gl::CompileShader(s_vertex);
            check_errors!(s_vertex, COMPILE_STATUS);

            let s_fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            dbg!(s_fragment);
            let frag_code = CString::new(frag_code).unwrap();
            gl::ShaderSource(
                s_fragment,
                1,
                &frag_code.as_ptr() as *const _,
                std::ptr::null(),
            );
            gl::CompileShader(s_fragment);
            check_errors!(s_fragment, COMPILE_STATUS);
            shader.id = gl::CreateProgram();
            gl::AttachShader(shader.id, s_vertex);
            gl::AttachShader(shader.id, s_fragment);
            gl::LinkProgram(shader.id);
            check_errors!(shader.id, LINK_STATUS);

            gl::DeleteShader(s_vertex);
            gl::DeleteShader(s_fragment);
        }
        println!("Compiled program. {}", shader.id);
        shader
    }
    pub fn use_shader(&self) {
        unsafe { gl::UseProgram(self.id) }
    }
    pub fn set<T: GlArg>(&self, name: &'static str, object: &T) {
        object.send(self.id, name)
    }
}

impl From<(&'static str, &'static str)> for Shader {
    fn from((vert_fp, frag_fp): (&'static str, &'static str)) -> Shader {
        let vert_code = std::fs::read_to_string(vert_fp).unwrap();
        let frag_code = std::fs::read_to_string(frag_fp).unwrap();
        dbg!(vert_fp, frag_fp);
        Shader::compile(vert_code, frag_code)
    }
}

pub trait GlArg {
    fn send(&self, id: u32, name: &'static str)
    where
        Self: std::marker::Sized;
}
macro_rules! gl_arg {
    ($typ: ty, $call: expr => (SELF)) => {
        impl GlArg for $typ {
            fn send(&self, id: u32, name: &'static str) {
                unsafe {
                    let cstr = CString::new(name).unwrap();
                    let gl_uniform_location = gl::GetUniformLocation(id, cstr.as_ptr() as *const _);
                    if(gl_uniform_location == -1) {
                        eprintln!("ERR: gl::GetUniformLocation for {:?} was not found. Silently ignoring.", cstr);
                    } else {
                        $call(
                            gl_uniform_location,
                            *self
                        )
                    }
                }
            }
        }
    };
    ($typ: ty, $call: expr => ($($e: ident),*)) => {
        impl GlArg for $typ {
            fn send(&self, id: u32, name: &'static str) {
                unsafe {
                    let cstr = CString::new(name).unwrap();
                    let gl_uniform_location = gl::GetUniformLocation(id, cstr.as_ptr() as *const _);
                    // $(dbg!(self.$e);)*
                    if(gl_uniform_location == -1) {
                        eprintln!("ERR: gl::GetUniformLocation for {:?} was not found. Silently ignoring.", cstr);
                    } else {
                        $call(
                            gl_uniform_location,
                            $(self.$e,)*
                        )
                    }
                }
            }
        }
    };
}

gl_arg!(i32, gl::Uniform1i => (SELF));
gl_arg!(f32, gl::Uniform1f => (SELF));
gl_arg!(nalgebra_glm::TVec2<i32>, gl::Uniform2i => (x, y));
gl_arg!(nalgebra_glm::TVec3<i32>, gl::Uniform3i => (x, y, z));
gl_arg!(nalgebra_glm::TVec4<i32>, gl::Uniform4i => (x, y, z, w));
gl_arg!(nalgebra_glm::TVec2<f32>, gl::Uniform2f => (x, y));
gl_arg!(nalgebra_glm::TVec3<f32>, gl::Uniform3f => (x, y, z));
gl_arg!(nalgebra_glm::TVec4<f32>, gl::Uniform4f => (x, y, z, w));

impl GlArg for nalgebra_glm::Mat4x4 {
    fn send(&self, id: u32, name: &'static str) {
        unsafe {
            let cstr = CString::new(name).unwrap();
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(id, cstr.as_ptr() as *const _),
                1,
                gl::FALSE,
                nalgebra_glm::value_ptr(self).as_ptr(),
            )
        }
    }
}

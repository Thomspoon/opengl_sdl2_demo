use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::ptr;
use std::str;

use gl_gen::gl;
use gl_gen::gl::types::*;

#[derive(Copy, Clone, Debug)]
pub struct Shader {
    program: GLuint,
}

impl Shader {
    pub fn from_source<'a>(vertex: &'a str, fragment: &'a str) -> Shader {

        let vertex_glsl = File::open(vertex).expect("Could not open vertex path");
        let fragment_glsl = File::open(fragment).expect("Could not open fragment path");
        
        let mut buf_reader = BufReader::new(vertex_glsl);
        let mut vertex_shader = String::new();
        buf_reader.read_to_string(&mut vertex_shader).expect("Could not read vertex path");;

        let mut buf_reader = BufReader::new(fragment_glsl);
        let mut fragment_shader = String::new();
        buf_reader.read_to_string(&mut fragment_shader).expect("Could not read fragment path");

        // Create GLSL shaders
        let vs = Self::compile_shader(vertex_shader.as_str(), gl::VERTEX_SHADER);
        let fs = Self::compile_shader(fragment_shader.as_str(), gl::FRAGMENT_SHADER);

        let program = Self::link_program(vs, fs);

        unsafe {
            gl::DeleteShader(fs);
            gl::DeleteShader(vs);
        }

        Shader {
            program,
        }
    }

    pub fn gl_use(self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn program(self) -> GLuint {
        self.program
    }

    fn compile_shader(src: &str, ty: GLenum) -> GLuint {
        let shader;

        unsafe {
            shader = gl::CreateShader(ty);

            // Attempt to compile the shader
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

                panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
            }
        }
        shader
    }

    fn link_program(vs: GLuint, fs: GLuint) -> GLuint { 
        let program;

        unsafe {
            program = gl::CreateProgram();

            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);

            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                
                panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
            }
        } 
        program
    }
}
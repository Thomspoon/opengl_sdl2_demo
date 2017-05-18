extern crate sdl2;

#[allow(non_upper_case_globals)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::time::Duration;

use gl::types::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

// Vertex data for first triangle
static VERTEX_DATA_1: [GLfloat; 9] = [
        0.05, -0.5, 0.0,
        0.55, -0.5, 0.0,
        0.05, 0.5, 0.0,
];

// Vertex data for second triangle
static VERTEX_DATA_2: [GLfloat; 9] = [
        -0.05, 0.5, 0.0,
        -0.05, -0.5, 0.0, 
        -0.55, -0.5, 0.0,
];

// Shader sources
static VS_SRC: &'static str =
   "#version 330 core

    layout (location = 0) in vec3 position;

    void main()
    {
        gl_Position = vec4(position.x, position.y, position.z, 1.0);
    }
    ";

static FS_SRC: &'static str =
   "#version 330 core

    out vec4 color;

    void main()
    {
        color = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
    ";

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

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    #[cfg(target_os = "macos")]
    video_subsystem.gl_attr().set_context_profile(GLProfile::Core);

    let window = video_subsystem.window("Window", 800, 600)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .present_vsync()
        .build()
        .unwrap();

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    canvas.window().gl_set_context_to_current().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut vao = [0; 2];
    let mut vbo = [0; 2];

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);

    unsafe {
        gl::GenVertexArrays(2, &mut vao[0]);
        gl::GenBuffers(2, &mut vbo[0]);

        // Bind first triangle
        gl::BindVertexArray(vao[0]);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
        gl::BufferData(gl::ARRAY_BUFFER, 
                        ((VERTEX_DATA_1.len() * mem::size_of::<GLfloat>()) as GLsizeiptr),
                        mem::transmute(&VERTEX_DATA_1[0]),
                        gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(0);

        // Bind second triangle
        gl::BindVertexArray(vao[1]);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
        gl::BufferData(gl::ARRAY_BUFFER, 
                        ((VERTEX_DATA_2.len() * mem::size_of::<GLfloat>()) as GLsizeiptr),
                        mem::transmute(&VERTEX_DATA_2[0]),
                        gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(0);

    }

    let program = link_program(vs, fs);

    unsafe {
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
    }

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        
        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);

            gl::BindVertexArray(vao[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);

            gl::BindVertexArray(vao[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        canvas.window().gl_swap_window();
    }

    // Cleanup
    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteBuffers(2, &vbo[0]);
        gl::DeleteVertexArrays(2, &vao[0]);
    }
}
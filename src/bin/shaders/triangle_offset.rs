extern crate sdl2;

#[path="../../shader/mod.rs"]
mod shader;
use shader::Shader;

#[path="../../gl_gen/mod.rs"]
mod gl_gen;
use gl_gen::gl;

use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::time::Duration;

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

// Vertex data
static VERTEX_DATA: [GLfloat; 18] = [
        // Position       // Color
        -0.5, -0.5, 0.0,  1.0, 0.0, 0.0,
         0.5, -0.5, 0.0,  0.0, 1.0, 0.0,
         0.0,  0.5, 0.0,  0.0, 0.0, 1.0,
];

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

    let mut vao = 0;
    let mut vbo = 0;

    let shader = Shader::from_source("src/bin/shaders/shader/triangle.glslv", "src/bin/shaders/shader/triangle.glslf");

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        // VAO
        gl::BindVertexArray(vao);

        // VBO
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 
                        ((VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr),
                        mem::transmute(&VERTEX_DATA[0]),
                        gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (6 * mem::size_of::<GLfloat>()) as i32, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (6 * mem::size_of::<GLfloat>()) as i32, (3 * mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(0);
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

            shader.gl_use();
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }

        canvas.window().gl_swap_window();
    }

    // Cleanup
    unsafe {
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
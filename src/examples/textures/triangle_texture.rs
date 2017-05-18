extern crate image;
extern crate sdl2;

#[path="../../shader/mod.rs"]
mod shader;
use shader::Shader;

#[path="../../gl_gen/mod.rs"]
mod gl_gen;
use gl_gen::gl;
use gl_gen::gl::types::*;

use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::time::Duration;

use image::GenericImage;

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
static VERTEX_DATA: [GLfloat; 32] = [
    // Positions       // Colors        // Texture Coords
     0.5,  0.5, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0,   // Top Right
     0.5, -0.5, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0,   // Bottom Right
    -0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   0.0, 0.0,   // Bottom Left
    -0.5,  0.5, 0.0,   1.0, 1.0, 0.0,   0.0, 1.0,   // Top Left 
];

static INDICES_DATA: [GLuint; 6] = [
    0, 1, 3, // First Triangle
    1, 2, 3  // Second Triangle
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
    let mut ebo = 0;
    let mut texture1 = 0;
    let mut texture2 = 0;

    let shader = Shader::from_source("src/examples/textures/shader/texture.glslv", "src/examples/textures/shader/texture.glslf");

    let texture_image1 = image::open(&Path::new("src/examples/textures/resources/container.jpg")).unwrap();
    let texture_image2 = image::open(&Path::new("src/examples/textures/resources/awesomeface.png")).unwrap();

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        // VAO
        gl::BindVertexArray(vao);

        // VBO
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 
                        ((VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr),
                        mem::transmute(&VERTEX_DATA[0]),
                        gl::STATIC_DRAW);

        // EBO
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 
                       ((INDICES_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr),
                       mem::transmute(&INDICES_DATA[0]),
                       gl::STATIC_DRAW);       

        // Position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Color attribute
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, (3 * mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(1);

        // Texture attribute
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (8 * mem::size_of::<GLfloat>()) as i32, (6 * mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(2); 
        
        gl::BindVertexArray(0);

        // Texture
        gl::GenTextures(1, &mut texture1);
        gl::GenTextures(1, &mut texture2);

        // Set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);	// Set texture wrapping to GL_REPEAT (usually basic wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // Set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::BindTexture(gl::TEXTURE_2D, texture1);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, texture_image1.width() as i32, 
                       texture_image1.height() as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 
                       texture_image1.to_rgb().into_raw().as_ptr() as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::BindTexture(gl::TEXTURE_2D, texture2);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, texture_image2.width() as i32, 
                       texture_image2.height() as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 
                       texture_image2.to_rgb().into_raw().as_ptr() as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);
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

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::Uniform1i(gl::GetUniformLocation(shader.program(), "ourTexture1".as_ptr() as *const i8), 0);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);
            gl::Uniform1i(gl::GetUniformLocation(shader.program(), "ourTexture2".as_ptr() as *const i8), 1);

            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
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
extern crate cgmath;
extern crate image;
extern crate sdl2;

#[path="../../shader/mod.rs"]
mod shader;
use shader::Shader;

#[path="../../gl_gen/mod.rs"]
mod gl_gen;
use gl_gen::gl;
use gl_gen::gl::types::*;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::time::Duration;

use cgmath::{Deg, InnerSpace, Matrix, Matrix4, 
             Point3, perspective, Rad, Vector3};

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

static VERTEX_DATA: [GLfloat; 180] = [
    -0.5, -0.5, -0.5,  0.0, 0.0,
     0.5, -0.5, -0.5,  1.0, 0.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5,  0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 0.0,

    -0.5, -0.5,  0.5,  0.0, 0.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 1.0,
     0.5,  0.5,  0.5,  1.0, 1.0,
    -0.5,  0.5,  0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,

    -0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5, -0.5,  1.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5,  0.5,  1.0, 0.0,

     0.5,  0.5,  0.5,  1.0, 0.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5,  0.5,  0.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0,

    -0.5, -0.5, -0.5,  0.0, 1.0,
     0.5, -0.5, -0.5,  1.0, 1.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
     0.5, -0.5,  0.5,  1.0, 0.0,
    -0.5, -0.5,  0.5,  0.0, 0.0,
    -0.5, -0.5, -0.5,  0.0, 1.0,

    -0.5,  0.5, -0.5,  0.0, 1.0,
     0.5,  0.5, -0.5,  1.0, 1.0,
     0.5,  0.5,  0.5,  1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0,
    -0.5,  0.5,  0.5,  0.0, 0.0,
    -0.5,  0.5, -0.5,  0.0, 1.0
];

static CUBE_POS: [(f32, f32, f32); 10] = [
    ( 0.0,  0.0,  0.0), 
    ( 2.0,  5.0, -15.0), 
    (-1.5, -2.2, -2.5),  
    (-3.8, -2.0, -12.3),  
    ( 2.4, -0.4, -3.5),  
    (-1.7,  3.0, -7.5),  
    ( 1.3, -2.0, -2.5),  
    ( 1.5,  2.0, -2.5), 
    ( 1.5,  0.2, -1.5), 
    (-1.3,  1.0, -1.5)  
];

fn main() {
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();

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
    let mut texture1 = 0;
    let mut texture2 = 0;

    let shader = Shader::from_source("src/examples/camera/shader/camera.glslv", "src/examples/camera/shader/camera.glslf");

    let texture_image1 = image::open(&Path::new("resources/container.jpg")).unwrap();
    let texture_image2 = image::open(&Path::new("resources/awesomeface.png")).unwrap();

    unsafe {
        gl::Enable(gl::DEPTH_TEST);  

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

        // Position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<GLfloat>()) as i32, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Texture attribute
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (5 * mem::size_of::<GLfloat>()) as i32, (3 * mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(2); 
        
        gl::BindVertexArray(0);

        // Texture
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);

        // Set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);	// Set texture wrapping to GL_REPEAT (usually basic wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);


        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, texture_image1.width() as i32, 
                       texture_image1.height() as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 
                       texture_image1.to_rgb().into_raw().as_ptr() as *const c_void);

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2);

        // Set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);	// Set texture wrapping to GL_REPEAT (usually basic wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // Set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, texture_image2.width() as i32, 
                       texture_image2.height() as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, 
                       texture_image2.to_rgb().into_raw().as_ptr() as *const c_void);

        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::BindTexture(gl::TEXTURE_2D, 0);

    }

    let camera_pos = Vector3::new(0.0, 0.0, 2.0);
    let camera_target = Vector3::new(0.0, 0.0, 0.0);
    let camera_dir = (camera_pos-camera_target).normalize();
    let camera_right = Vector3::new(0.0, 1.0, 0.0).cross(camera_dir).normalize();
    let camera_up = camera_dir.cross(camera_right);

    'running: loop {        
        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::Uniform1i(gl::GetUniformLocation(shader.program(), CString::new("ourTexture1").unwrap().as_ptr()), 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);
            gl::Uniform1i(gl::GetUniformLocation(shader.program(), CString::new("ourTexture2").unwrap().as_ptr()), 1);

            shader.gl_use();

            let ticks = (timer.ticks() as f32) / 1000.0;

            let radius = 10.0;
            let cam_x = ticks.sin() * radius;
            let cam_z = ticks.cos() * radius;

            let model = Matrix4::from_angle_x(Deg(50.0 * ticks)) * Matrix4::from_angle_y(Deg(50.0 * ticks));
            let view = Matrix4::look_at(Point3::new(cam_x, 0.0, cam_z), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
            let projection = perspective(Rad::from(Deg(90.0)), 1.33, 0.1, 100.0);

            let model_loc = gl::GetUniformLocation(shader.program(), CString::new("model").unwrap().as_ptr());
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());

            let view_loc = gl::GetUniformLocation(shader.program(), CString::new("view").unwrap().as_ptr());
            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());

            let projection_loc = gl::GetUniformLocation(shader.program(), CString::new("projection").unwrap().as_ptr());
            gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());

            gl::BindVertexArray(vao);
            for i in 0..CUBE_POS.len() {
                let model = Matrix4::from_translation(Vector3::new(CUBE_POS[i].0, CUBE_POS[i].1, CUBE_POS[i].2)) 
                            * Matrix4::from_angle_x(Deg(20.0 * i as f32))
                            * 0.3 * Matrix4::from_angle_y(Deg(20.0 * i as f32)) * 0.5 * Matrix4::from_angle_z(Deg(20.0 * i as f32));
                gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        canvas.window().gl_swap_window();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    // Cleanup
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
    }
}
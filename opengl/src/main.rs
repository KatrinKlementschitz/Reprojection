#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[macro_use]
extern crate glium;
extern crate nalgebra_glm as glm;
extern crate simdnoise;
#[path = "camera.rs"] mod camera;

use camera::CameraState;
use glium::{glutin, texture, Surface, index::PrimitiveType};
use simdnoise::NoiseBuilder;
use std::fs;

#[derive(Copy, Clone)]
struct VertexRect {
    position: [f32; 3],
}
implement_vertex!(VertexRect, position);

use std::time::Instant;
struct Rect {
    index_buffer: glium::IndexBuffer<u16>,
    vertex_buffer: glium::VertexBuffer<VertexRect>,
    texture: glium::Texture2d,
    start: Instant
}

static WIDTH:  u32 = 25;
static HEIGHT: u32 = 25;
static DEPTH:  u32 = 25;
static SCREEN_WIDTH:  u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

impl Rect {
    fn new(display: &glium::Display) -> Rect {        
        let vertex_buffer = {
            glium::VertexBuffer::new(display,
                &[
                    VertexRect { position:[ 1.0,  1.0, 0.0] }, // top right
                    VertexRect { position:[ 1.0, -1.0, 0.0] }, // bottom right
                    VertexRect { position:[-1.0, -1.0, 0.0] }, // bottom left
                    VertexRect { position:[-1.0,  1.0, 0.0] }, // top left 
                ]
            ).unwrap()
        };

        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 3, 1, 1, 3, 2]).unwrap();

        Rect {
            vertex_buffer,
            index_buffer,
            start: Instant::now(),
            texture: texture::Texture2d::empty_with_format(display,
                texture::UncompressedFloatFormat::U8U8U8U8,
                texture::MipmapsOption::NoMipmap, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap()
        }
    }

    pub fn draw<T: glium::Surface>(&mut self, program: &glium::Program, target: &mut T, cam: &mut CameraState, tex: &texture::texture3d::Texture3d, b: bool, prev_view: glm::TMat4<f32>)
    {
        cam.update_camera_vectors(self.start.elapsed().as_secs_f32());
        self.start = Instant::now();
        let proj: [[f32; 4]; 4] = glm::perspective(8.0/6.0, glm::radians(&glm::vec1(100.0)).x, 0.1, 100.0).into();
        let view: [[f32; 4]; 4] = cam.get_view_matrix().into();
        let s = tex.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
        let ft = self.texture.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
        let prev_v: [[f32; 4]; 4] = prev_view.into();

        let uniforms = uniform! {
            width: WIDTH,
            height: HEIGHT,
            depth: DEPTH,
            block_tex: s,
            frame_tex: ft,
            show_texture: b,
            proj: proj,
            view: view,
            prev_proj: proj,
            prev_view: prev_v,
        };  
        
        target.draw(&self.vertex_buffer, &self.index_buffer, &program, &uniforms, &Default::default()).unwrap();
    }

    pub fn draw_to_texture(&mut self, display: &glium::Display, program: &glium::Program, cam: &mut CameraState, tex: &texture::texture3d::Texture3d)
    {
        let proj: [[f32; 4]; 4] = glm::perspective(8.0/6.0, glm::radians(&glm::vec1(100.0)).x, 0.1, 100.0).into();
        let view: [[f32; 4]; 4] = cam.get_view_matrix().into();
        let s = tex.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
        let ft = self.texture.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest).magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

        let uniforms = uniform! {
            width: WIDTH,
            height: HEIGHT,
            depth: DEPTH,
            block_tex: s,
            frame_tex: ft,
            show_texture: false,
            proj: proj,
            view: view,
            prev_proj: proj,
            prev_view: view,
        };  
        
        let mut f = glium::framebuffer::SimpleFrameBuffer::new(display, &self.texture).unwrap();
        f.draw(&self.vertex_buffer, &self.index_buffer, &program, &uniforms, &Default::default()).unwrap();
    }
}

fn create_terrain(display: &glium::Display) -> texture::texture3d::Texture3d {
    let noise = NoiseBuilder::ridge_3d(WIDTH as usize, HEIGHT as usize, DEPTH as usize)
        .with_freq(1.0)
        .with_octaves(5)
        .with_gain(2.0)
        .with_seed(1337)
        .with_lacunarity(0.5)
        .generate_scaled(0.0, 255.0);

    let noise_u8: Vec<u8> = noise.iter().map(|&e| {
        let x = e as u8;
        if x > 127 { return 0; }
        else { return x; }
    }).collect();
    
    use std::borrow::Cow;
    let raw_tex3d = texture::RawImage3d{
        data: Cow::Owned(noise_u8),
        width: WIDTH,
        height: HEIGHT,
        depth: DEPTH,
        format: texture::ClientFormat::U8,
    };

    texture::texture3d::Texture3d::with_format(
        display,
        raw_tex3d,
        texture::UncompressedFloatFormat::U8,
        texture::MipmapsOption::NoMipmap
    ).unwrap()
}

#[no_mangle]
fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        })
        .with_title("Title");
        
    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);
        
    let windowed_context = context_builder
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    
        glium::Display::from_gl_window(windowed_context).unwrap()
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(&display);
    let mut cam = CameraState::new();
    let mut rect = Rect::new(&display);

    let fragment = fs::read_to_string("shader.fs").expect("Unable to read fs");
    let vertex = fs::read_to_string("shader.vs").expect("Unable to read vs");

    let program = program!(&display,
        450 => {
            vertex: &vertex,
            fragment: &fragment
        },
    ).unwrap();

    let tex3d = create_terrain(&display);

    // Draw the triangle to the screen.
    let mut draw_texture = false;
    let mut prev_view = cam.get_view_matrix();

    event_loop.run(move |event, _, control_flow| {

        let mut redraw = || {
            let mut quit = false;

            let (_, shapes) = egui_glium.run(&display, |egui_ctx| {
                egui::Window::new("Debug Window").show(egui_ctx, |ui| {
                    ui.label(format!("cam rotation: {:+0.1} : {:+0.1}", cam.rotation.x, cam.rotation.y));
                    ui.label(format!("cam position: {:+0.1} : {:+0.1} : {:+0.1}", cam.position.x, cam.position.y, cam.position.z));
                    ui.label(format!("cam front: {:+0.1} : {:+0.1} : {:+0.1}", cam.front().x, cam.front().y, cam.front().z));
                    if ui.button("Quit").clicked() {
                        quit = true;
                    }
                });
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            };

            {
                let mut target = display.draw();
                target.clear_color(0.0,0.0,0.0,0.0);

                if !draw_texture {
                    prev_view = cam.get_view_matrix();
                }
                
                rect.draw(&program, &mut target, &mut cam, &tex3d, draw_texture, prev_view);
                egui_glium.paint(&display, &mut target, shapes);

                target.finish().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),


            glutin::event::Event::DeviceEvent { event, .. } => {
                use glutin::event::DeviceEvent;
                                
                if let DeviceEvent::MouseMotion{ delta } = event {
                    cam.process_input_cursor(glm::vec2(delta.0 as f32, delta.1 as f32));
                }
            }


            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                
                cam.process_input_keyboard(&event);

                if let glutin::event::WindowEvent::KeyboardInput{ input, .. } = event {
                    if input.virtual_keycode == Some(glutin::event::VirtualKeyCode::Escape) {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    else if input.virtual_keycode == Some(glutin::event::VirtualKeyCode::T) && input.state == glutin::event::ElementState::Pressed {
                        if !draw_texture {
                            rect.draw_to_texture(&display, &program, &mut cam, &tex3d);
                        }
                        draw_texture = !draw_texture;
                    }
                }

                //if matches!(event, WindowEvent::Resized(..)) {
                    //draw(&rect, &program, &display);
                //}

                egui_glium.on_event(&event);

                display.gl_window().window().request_redraw();
            }

            _ => (),
        }
    });
}
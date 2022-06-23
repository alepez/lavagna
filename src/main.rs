#![deny(clippy::all)]
#![forbid(unsafe_code)]

extern crate core;

mod app;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalSize;
use winit::window::CursorIcon;
use winit_input_helper::WinitInputHelper;
use crate::app::AppBuilder;
use crate::app::doc::MutSketch;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let mut canvas_size = PhysicalSize::new(640, 480);

    let window = {
        WindowBuilder::new()
            .with_title("lavagna")
            .with_inner_size(canvas_size)
            .with_min_inner_size(canvas_size)
            .build(&event_loop)
            .unwrap()
    };

    window.set_cursor_icon(CursorIcon::Crosshair);

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(canvas_size.width, canvas_size.height, &window);
        Pixels::new(canvas_size.width, canvas_size.height, surface_texture)?
    };

    let mut app = AppBuilder::new().build();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::X) {
                app.clear_all();
            }

            if input.key_pressed(VirtualKeyCode::C) {
                app.change_color();
            }

            if input.key_pressed(VirtualKeyCode::Z) {
                app.resume();
            }

            if input.key_pressed(VirtualKeyCode::S) {
                app.take_snapshot();
            }

            let mouse = input.mouse();

            if let Some(mouse) = mouse {
                if let Ok((x, y)) = pixels.window_pos_to_pixel(mouse) {
                    app.set_cursor_position(x as isize, y as isize);
                }
            }

            if input.mouse_pressed(0) {
                app.set_pressed(true);
            }

            if input.mouse_released(0) {
                app.set_pressed(false);
            }

            if let Some(new_size) = input.window_resized() {
                if canvas_size != new_size {
                    resize_buffer(&mut pixels, canvas_size, new_size);
                    canvas_size = new_size;
                }
            }

            let sketch = MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height);

            app.update(sketch);

            window.request_redraw();
        }
    });
}

fn resize_buffer(pixels: &mut Pixels, canvas_size: PhysicalSize<u32>, new_size: PhysicalSize<u32>) {
    let old_sketch = MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height).to_owned();

    pixels.get_frame().fill(0x00);
    pixels.resize_surface(new_size.width, new_size.height);
    pixels.resize_buffer(new_size.width, new_size.height);

    let mut new_sketch = MutSketch::new(pixels.get_frame(), new_size.width, new_size.height);

    new_sketch.copy_from(&old_sketch.as_sketch());
}
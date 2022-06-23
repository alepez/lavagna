#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod app;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalSize;
use winit_input_helper::WinitInputHelper;
use crate::app::{App, AppBuilder};

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

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(canvas_size.width, canvas_size.height, &window);
        Pixels::new(canvas_size.width, canvas_size.height, surface_texture)?
    };

    let mut app = AppBuilder::new()
        .with_size(canvas_size.width as isize, canvas_size.height as isize)
        .build();

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

            if input.key_pressed(VirtualKeyCode::B) {
                app.backup();
            }

            let mouse = input.mouse();

            if let Some(mouse) = mouse {
                if let Ok((x, y)) = pixels.window_pos_to_pixel(mouse) {
                    app.set_position(x as isize, y as isize);
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
                    resize_canvas(&mut canvas_size, &mut pixels, &mut app, new_size);
                }
            }

            app.update(pixels.get_frame());

            window.request_redraw();
        }
    });
}

fn resize_canvas(canvas_size: &mut PhysicalSize<u32>, pixels: &mut Pixels, app: &mut App, new_size: PhysicalSize<u32>) {
    resize_buffer(canvas_size, pixels, new_size);

    canvas_size.width = new_size.width;
    canvas_size.height = new_size.height;

    app.resize(new_size.width as isize, new_size.height as isize);
}

fn resize_buffer(canvas_size: &PhysicalSize<u32>, pixels: &mut Pixels, new_size: PhysicalSize<u32>) {
    let mut backup = Vec::from(pixels.get_frame());

    pixels.get_frame().fill(0x00);
    pixels.resize_surface(new_size.width, new_size.height);
    pixels.resize_buffer(new_size.width, new_size.height);

    copy_frame(
        pixels.get_frame(),
        new_size.width as usize,
        new_size.height as usize,
        backup.as_mut_slice(),
        canvas_size.width as usize,
        canvas_size.height as usize,
    );
}

fn copy_frame(dst: &mut [u8], dst_w: usize, dst_h: usize, src: &mut [u8], src_w: usize, src_h: usize) {
    let min_h = usize::min(dst_h, src_h);
    let min_w = usize::min(dst_w, src_w);

    for y in 0..min_h {
        let dst_begin = dst_w * y;
        let dst_end = dst_begin + min_w;
        let src_begin = src_w * y;
        let src_end = src_begin + min_w;
        let dst_range = (4 * dst_begin)..(4 * dst_end);
        let src_range = (4 * src_begin)..(4 * src_end);
        dst[dst_range].copy_from_slice(&src[src_range]);
    }
}
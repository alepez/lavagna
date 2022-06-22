#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod app;

use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use crate::app::AppBuilder;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let mut canvas_size = LogicalSize::new(640, 480);

    let window = {
        let min_size = LogicalSize::new(128, 128);

        WindowBuilder::new()
            .with_title("Lavagna")
            .with_inner_size(canvas_size)
            .with_min_inner_size(min_size)
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
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::RedrawRequested(_) = event {
            // life.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
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
                debug!("Resize pixels to {}x{}", new_size.width, new_size.height);
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

                canvas_size.width = new_size.width;
                canvas_size.height = new_size.height;
                app.resize(new_size.width as isize, new_size.height as isize);
            }

            app.update(pixels.get_frame());

            window.request_redraw();
        }
    });
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
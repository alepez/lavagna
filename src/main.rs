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

    let canvas_size = LogicalSize::new(640, 480);

    let window = {
        let default_size = LogicalSize::new(640., 480.);
        let min_size = LogicalSize::new(128., 128.);

        WindowBuilder::new()
            .with_title("Lavagna")
            .with_inner_size(default_size)
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

            if let Some(size) = input.window_resized() {
                debug!("Resize pixels to {}x{}", size.width, size.height);
                pixels.resize_surface(size.width, size.height);
            }

            app.update(pixels.get_frame());

            window.request_redraw();
        }
    });
}

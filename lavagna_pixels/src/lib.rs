#![deny(clippy::all)]
#![forbid(unsafe_code)]

use lavagna_collab::{CollabOpt, CollaborationChannel, SupportedCollaborationChannel};
use lavagna_core::doc::MutSketch;
use lavagna_core::doc::OwnedSketch;
use lavagna_core::{App, CommandSender};
use log::error;
pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use std::cell::RefCell;
use std::rc::Rc;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, Touch, TouchPhase, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorIcon, Window, WindowBuilder};

pub struct Opt {
    pub collab: Option<CollabOpt>,
}

pub fn run(opt: Opt) -> Result<(), Error> {
    log::info!("lavagna start");

    let event_loop = EventLoop::new();
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

    let pen_id = opt.collab.as_ref().map(|x| x.pen_id).unwrap_or_default();

    let collab = opt
        .collab
        .map(|x| x.url)
        .as_deref()
        .map(SupportedCollaborationChannel::new)
        .unwrap_or_default();

    let collab = Rc::new(RefCell::new(collab));

    let mut app = App::new(pen_id);

    {
        let collab = collab.clone();
        app.connect_command_sender(Box::new(move |cmd| {
            collab.borrow_mut().send_command(cmd);
        }));
    }

    let mut frozen_sketch: Option<OwnedSketch> = None;
    let mut pixels: Option<Pixels> = None;

    event_loop.run(move |event, _, control_flow| {
        match event {
            // Resumed on Android
            Event::Resumed => {
                canvas_size = window.inner_size();
                pixels = resume(&window, canvas_size, frozen_sketch.take());

                // Prevent drawing a line from the last location when resuming
                app.force_release();
            }
            // Suspended on Android
            Event::Suspended => {
                frozen_sketch = sketch_from_pixels(pixels.take(), canvas_size);
            }
            // Window resized on Desktop (Linux/Windows/iOS)
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if let Some(pixels) = pixels.as_mut() {
                    if canvas_size != new_size {
                        resize_buffer(pixels, canvas_size, new_size);
                    }
                } else {
                    frozen_sketch = sketch_from_pixels(pixels.take(), canvas_size);
                    pixels = resume(&window, new_size, frozen_sketch.take());
                    window.request_redraw();
                }

                canvas_size = new_size;
            }
            _ => (),
        }

        let mut exit = false;

        if let Some(pixels) = pixels.as_mut() {
            match event {
                Event::MainEventsCleared => {
                    // All events from winit have been received, now it's time
                    // to handle events from collaborators.
                    handle_commands_from_collaborators(&collab, &mut app);
                }
                Event::RedrawRequested(_) => {
                    exit = redraw(pixels, canvas_size, &mut app).is_err();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    exit = true;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::Touch(Touch {
                            phase, location, ..
                        }),
                    ..
                } => {
                    match phase {
                        TouchPhase::Started => app.press(),
                        TouchPhase::Ended => app.release(),
                        _ => (),
                    }
                    move_cursor_to_position(location, pixels, &mut app);
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => move_cursor_to_position(position, pixels, &mut app),
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            state,
                            button: MouseButton::Left,
                            ..
                        },
                    ..
                } => match state {
                    ElementState::Pressed => app.press(),
                    ElementState::Released => app.release(),
                },
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(key_released),
                                    ..
                                },
                            ..
                        },
                    ..
                } => match key_released {
                    VirtualKeyCode::Escape => exit = true,
                    VirtualKeyCode::X => app.clear_all(),
                    VirtualKeyCode::C => app.change_color(),
                    VirtualKeyCode::U => app.resume_last_snapshot(),
                    VirtualKeyCode::S => app.take_snapshot(),
                    _ => (),
                },
                _ => (),
            }
        }

        if exit {
            *control_flow = ControlFlow::Exit;
        }

        if app.needs_update() {
            window.request_redraw();
        }
    });
}

fn resume(
    window: &Window,
    new_size: PhysicalSize<u32>,
    frozen_sketch: Option<OwnedSketch>,
) -> Option<Pixels> {
    let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &window);
    let mut pixels = Pixels::new(new_size.width, new_size.height, surface_texture).ok()?;

    pixels.get_frame().fill(0x00);

    let mut new_sketch = MutSketch::new(pixels.get_frame(), new_size.width, new_size.height);

    if let Some(old_sketch) = &frozen_sketch {
        new_sketch.copy_from(&old_sketch.as_sketch());
    }

    Some(pixels)
}

fn sketch_from_pixels(
    pixels: Option<Pixels>,
    canvas_size: PhysicalSize<u32>,
) -> Option<OwnedSketch> {
    let mut pixels = pixels?;
    let frame = pixels.get_frame();
    let sketch = MutSketch::new(frame, canvas_size.width, canvas_size.height);
    Some(sketch.to_owned())
}

fn resize_buffer(pixels: &mut Pixels, canvas_size: PhysicalSize<u32>, new_size: PhysicalSize<u32>) {
    let old_sketch =
        MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height).to_owned();

    pixels.get_frame().fill(0x00);
    pixels.resize_surface(new_size.width, new_size.height);
    pixels.resize_buffer(new_size.width, new_size.height);

    let mut new_sketch = MutSketch::new(pixels.get_frame(), new_size.width, new_size.height);

    new_sketch.copy_from(&old_sketch.as_sketch());
}

fn handle_commands_from_collaborators(
    collab: &Rc<RefCell<SupportedCollaborationChannel>>,
    app: &mut App,
) {
    while let Ok(cmd) = collab.borrow_mut().rx().try_recv() {
        app.send_command(cmd);
    }
}

fn redraw(pixels: &mut Pixels, canvas_size: PhysicalSize<u32>, app: &mut App) -> Result<(), ()> {
    let sketch = MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height);
    app.update(sketch);

    pixels
        .render()
        .map_err(|e| error!("pixels.render() failed: {}", e))
}

fn move_cursor_to_position(position: PhysicalPosition<f64>, pixels: &Pixels, app: &mut App) {
    if let Ok((x, y)) = pixels.window_pos_to_pixel(position.into()) {
        app.move_cursor(x as isize, y as isize);
    }
}

#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use log::error;
pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, Touch, TouchPhase, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorIcon, Window, WindowBuilder};

use lavagna_collab::{CollabOpt, CollabUri, CollaborationChannel, SupportedCollaborationChannel};
use lavagna_core::doc::MutSketch;
use lavagna_core::doc::OwnedSketch;
use lavagna_core::{App, CommandSender, Cursor, CursorPos};

use crate::gui::Framework;

mod gui;

pub struct Opt {
    pub collab: Option<CollabOpt>,
}

fn connect_collab_channel(app: &mut App, collab: Rc<RefCell<SupportedCollaborationChannel>>) {
    app.connect_command_sender(Box::new(move |cmd| {
        collab.borrow_mut().send_command(cmd);
    }));
}

fn add_collab_channel(
    app: &mut App,
    uri: &CollabUri,
) -> Rc<RefCell<SupportedCollaborationChannel>> {
    let collab = SupportedCollaborationChannel::new(uri);
    let collab = Rc::new(RefCell::new(collab));
    connect_collab_channel(app, collab.clone());
    collab
}

fn get_collab_uri(opt: &Opt) -> CollabUri {
    let collab_uri = opt
        .collab
        .as_ref()
        .and_then(|x| x.uri_provider.as_ref())
        .and_then(|x| x.uri())
        .unwrap_or_default();

    log::info!("uri: {:?}", &collab_uri);

    collab_uri
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

    let mut app = App::new(pen_id);

    let collab_uri = get_collab_uri(&opt);

    #[allow(unused_mut)] // Only when target os is android this will be muted
    let mut collab = add_collab_channel(&mut app, &collab_uri);

    let mut frozen_sketch: Option<OwnedSketch> = None;
    let mut pixels: Option<Pixels> = resume(&window, canvas_size, frozen_sketch.take());

    let mut framework = Framework::new(
        &event_loop,
        canvas_size.width,
        canvas_size.height,
        1.,
        pixels.as_ref().unwrap(),
    );

    let mut cursor = Cursor::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            // Resumed on Android
            Event::Resumed => {
                log::info!("Resumed");
                canvas_size = window.inner_size();
                pixels = resume(&window, canvas_size, frozen_sketch.take());
                framework.set_pixels(pixels.as_ref().unwrap());
                collab = add_collab_channel(&mut app, &collab_uri);

                // Prevent drawing a line from the last location when resuming
                cursor.pressed = false;
            }
            // Suspended on Android
            Event::Suspended => {
                frozen_sketch = sketch_from_pixels(pixels.take(), canvas_size);
                cursor.pressed = false;
            }
            // Window resized on Desktop (Linux/Windows/iOS)
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if let Some(pixels) = pixels.as_mut() {
                    if canvas_size != new_size {
                        resize_buffer(pixels, canvas_size, new_size);
                        framework.resize(canvas_size.width, canvas_size.height);
                    }
                } else {
                    frozen_sketch = sketch_from_pixels(pixels.take(), canvas_size);
                    pixels = resume(&window, new_size, frozen_sketch.take());
                    framework.set_pixels(pixels.as_ref().unwrap());
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
                    framework.prepare(&window);
                    exit = redraw(pixels, canvas_size, &mut app, &mut framework).is_err();
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
                        TouchPhase::Started => cursor.pressed = true,
                        TouchPhase::Ended => cursor.pressed = false,
                        _ => (),
                    }

                    cursor.pos = window_pos_to_cursor(location, pixels);
                    app.move_cursor(cursor);
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    cursor.pos = window_pos_to_cursor(position, pixels);
                    app.move_cursor(cursor);
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            state,
                            button: MouseButton::Left,
                            ..
                        },
                    ..
                } => match state {
                    ElementState::Pressed => {
                        cursor.pressed = true;
                        app.move_cursor(cursor)
                    }
                    ElementState::Released => {
                        cursor.pressed = false;
                        app.move_cursor(cursor)
                    }
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
                    VirtualKeyCode::N => app.shrink_pen(),
                    VirtualKeyCode::M => app.grow_pen(),
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

    pixels.get_frame_mut().fill(0x00);

    let mut new_sketch = MutSketch::new(pixels.get_frame_mut(), new_size.width, new_size.height);

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
    let frame = pixels.get_frame_mut();
    let sketch = MutSketch::new(frame, canvas_size.width, canvas_size.height);
    Some(sketch.to_owned())
}

fn resize_buffer(pixels: &mut Pixels, canvas_size: PhysicalSize<u32>, new_size: PhysicalSize<u32>) {
    let old_sketch = MutSketch::new(
        pixels.get_frame_mut(),
        canvas_size.width,
        canvas_size.height,
    )
    .to_owned();

    pixels.get_frame_mut().fill(0x00);
    pixels.resize_surface(new_size.width, new_size.height);
    pixels.resize_buffer(new_size.width, new_size.height);

    let mut new_sketch = MutSketch::new(pixels.get_frame_mut(), new_size.width, new_size.height);

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

fn redraw(
    pixels: &mut Pixels,
    canvas_size: PhysicalSize<u32>,
    app: &mut App,
    framework: &mut Framework,
) -> Result<(), ()> {
    let sketch = MutSketch::new(
        pixels.get_frame_mut(),
        canvas_size.width,
        canvas_size.height,
    );
    app.update(sketch);

    pixels
        .render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);
            framework.render(encoder, render_target, context);
            Ok(())
        })
        .map_err(|e| error!("pixels.render() failed: {}", e))
}

fn window_pos_to_cursor(position: PhysicalPosition<f64>, pixels: &Pixels) -> CursorPos {
    let pos = pixels.window_pos_to_pixel(position.into());
    let (x, y) = pos.unwrap_or_default();
    CursorPos {
        x: x as isize,
        y: y as isize,
    }
}

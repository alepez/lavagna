#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

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

use crate::gui::Gui;

mod gui;

pub struct Opt {
    pub collab: Option<CollabOpt>,
}

pub fn run(opt: Opt) -> Result<(), Error> {
    log::info!("lavagna start");

    let event_loop = EventLoop::new();
    let canvas_size = PhysicalSize::new(640, 480);

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
    let collab = add_collab_channel(&mut app, &collab_uri);
    let gui = Gui::new(&event_loop);

    let mut running = PixelsApp {
        app,
        collab,
        frozen_sketch: None,
        visible: None,
        gui,
        window,
        exit: false,
    };

    event_loop.run(move |event, _, control_flow| {
        running.update(event);

        if running.exit {
            *control_flow = ControlFlow::Exit;
        }
    });
}

struct PixelsApp {
    window: Window,
    app: App,
    collab: Rc<RefCell<SupportedCollaborationChannel>>,
    frozen_sketch: Option<OwnedSketch>,
    gui: Gui,
    exit: bool,
    visible: Option<Visible>,
}

struct Visible {
    pixels: Pixels,
    cursor: Cursor,
    canvas_size: PhysicalSize<u32>,
}

impl PixelsApp {
    fn update(&mut self, event: winit::event::Event<()>) {
        #[cfg(feature = "gui")]
        if let Event::WindowEvent { event, .. } = &event {
            self.gui.handle_event(event);
        }

        self.handle_event(&event);

        if self.visible.is_some() {
            self.handle_event_when_visible(&event);
            self.handle_gui_events();
        }

        if self.app.needs_update() {
            self.window.request_redraw();
        }
    }

    fn handle_gui_events(&mut self) {
        if let Some(event) = self.gui.take_event() {
            match event {
                gui::Event::ChangeColor => self.app.change_color(),
                gui::Event::ClearAll => self.app.clear_all(),
                gui::Event::ShrinkPen => self.app.shrink_pen(),
                gui::Event::GrowPen => self.app.grow_pen(),
            }
        }
    }

    fn handle_event(&mut self, event: &winit::event::Event<()>) {
        match event {
            // Resumed on Android
            Event::Resumed => self.resume(),
            // Suspended on Android
            Event::Suspended => self.suspend(),
            // Window resized on Desktop (Linux/Windows/iOS)
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                if self.visible.is_some() {
                    self.resize();
                } else {
                    self.resume();
                }
            }
            _ => (),
        }
    }

    fn handle_event_when_visible(&mut self, event: &winit::event::Event<()>) {
        let visible = self.visible.as_mut().unwrap();
        let pixels = &mut visible.pixels;
        let cursor = &mut visible.cursor;

        match *event {
            Event::MainEventsCleared => {
                // All events from winit have been received, now it's time
                // to handle events from collaborators.
                handle_commands_from_collaborators(&self.collab, &mut self.app);
            }
            Event::RedrawRequested(_) => {
                self.redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                self.exit = true;
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
                self.app.move_cursor(cursor);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cursor.pos = window_pos_to_cursor(position, pixels);
                self.app.move_cursor(cursor);
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
                    self.app.move_cursor(cursor)
                }
                ElementState::Released => {
                    cursor.pressed = false;
                    self.app.move_cursor(cursor)
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
                VirtualKeyCode::Escape => self.exit = true,
                VirtualKeyCode::X => self.app.clear_all(),
                VirtualKeyCode::C => self.app.change_color(),
                VirtualKeyCode::N => self.app.shrink_pen(),
                VirtualKeyCode::M => self.app.grow_pen(),
                VirtualKeyCode::U => self.app.resume_last_snapshot(),
                VirtualKeyCode::S => self.app.take_snapshot(),
                _ => (),
            },
            _ => (),
        }
    }

    fn redraw(&mut self) {
        #[cfg(feature = "gui")]
        self.gui.prepare(&self.window);

        let visible = self.visible.as_mut().unwrap();
        let pixels = &mut visible.pixels;
        let canvas_size = visible.canvas_size;

        let sketch = MutSketch::new(
            pixels.get_frame_mut(),
            canvas_size.width,
            canvas_size.height,
        );

        self.app.update(sketch);

        if let Err(err) = pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);

            #[cfg(feature = "gui")]
            self.gui.render(encoder, render_target, context);

            Ok(())
        }) {
            log::error!("pixels.render() failed: {}", err);
            self.exit = true;
        }
    }

    fn suspend(&mut self) {
        log::debug!("Suspend");
        if let Some(visible) = self.visible.take() {
            self.frozen_sketch = Some(sketch_from_pixels(visible.pixels, visible.canvas_size));
        }
    }

    fn resize(&mut self) {
        log::debug!("Resize");

        let visible = self.visible.as_mut().unwrap();
        let new_size = self.window.inner_size();

        if visible.canvas_size != new_size {
            resize_buffer(&mut visible.pixels, visible.canvas_size, new_size);
            visible.canvas_size = new_size;
            self.gui.show(&visible.pixels, visible.canvas_size);
        }
    }

    fn resume(&mut self) {
        log::debug!("Resume");
        let new_size = self.window.inner_size();

        let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &self.window);
        let mut pixels = Pixels::new(new_size.width, new_size.height, surface_texture)
            .expect("Cannot create pixels from surface texture");

        pixels.get_frame_mut().fill(0x00);

        let mut new_sketch =
            MutSketch::new(pixels.get_frame_mut(), new_size.width, new_size.height);

        if let Some(old_sketch) = self.frozen_sketch.take() {
            new_sketch.copy_from(&old_sketch.as_sketch());
        }

        self.gui.show(&pixels, new_size);

        self.visible = Some(Visible {
            pixels,
            cursor: Cursor::new(),
            canvas_size: new_size,
        })
    }
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

fn sketch_from_pixels(mut pixels: Pixels, canvas_size: PhysicalSize<u32>) -> OwnedSketch {
    let frame = pixels.get_frame_mut();
    let sketch = MutSketch::new(frame, canvas_size.width, canvas_size.height);
    sketch.to_owned()
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

fn window_pos_to_cursor(position: PhysicalPosition<f64>, pixels: &Pixels) -> CursorPos {
    let pos = pixels.window_pos_to_pixel(position.into());
    let (x, y) = pos.unwrap_or_default();
    CursorPos {
        x: x as isize,
        y: y as isize,
    }
}

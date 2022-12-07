#![allow(unused)] // When feature gui is disabled, most of this module is unused
use pixels::{wgpu, Pixels, PixelsContext};
use winit::window::Window;
use winit::{dpi::PhysicalSize, event_loop::EventLoopWindowTarget};

use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_wgpu::renderer::{RenderPass, ScreenDescriptor};

pub(crate) struct Gui {
    egui_state: egui_winit::State,
    gui: State,
    visible: Option<Visible>,
}

struct State {
    emitted_event: Option<Event>,
}

struct Visible {
    egui_ctx: Context,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
}

pub enum Event {
    ChangeColor,
    ClearAll,
    ShrinkPen,
    GrowPen,
}

impl Gui {
    /// Create egui.
    pub(crate) fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
        let mut egui_state = egui_winit::State::new(event_loop);

        let gui = State::new();

        Self {
            egui_state,
            gui,
            visible: None,
        }
    }

    pub(crate) fn show(&mut self, pixels: &Pixels, size: PhysicalSize<u32>) {
        let scale_factor = 3.0;
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        self.egui_state.set_max_texture_side(max_texture_size);
        self.egui_state.set_pixels_per_point(scale_factor);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: scale_factor,
        };

        let textures = TexturesDelta::default();

        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);

        let visible = Visible {
            egui_ctx: Context::default(),
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
        };

        self.visible = Some(visible);
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let Some(visible) = &mut self.visible else {
            return;
        };

        self.egui_state.on_event(&visible.egui_ctx, event);
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) {
        let Some(visible) = &mut self.visible else {
            return;
        };

        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = visible.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx);
        });

        visible.textures.append(output.textures_delta);

        self.egui_state
            .handle_platform_output(window, &visible.egui_ctx, output.platform_output);

        visible.paint_jobs = visible.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        let Some(visible) = &mut self.visible else {
            return;
        };

        let rpass = &mut visible.rpass;

        // Upload all resources to the GPU.
        for (id, image_delta) in &visible.textures.set {
            rpass.update_texture(&context.device, &context.queue, *id, image_delta);
        }
        rpass.update_buffers(
            &context.device,
            &context.queue,
            &visible.paint_jobs,
            &visible.screen_descriptor,
        );

        // Record all render passes.
        rpass.execute(
            encoder,
            render_target,
            &visible.paint_jobs,
            &visible.screen_descriptor,
            None,
        );

        // Cleanup
        let textures = std::mem::take(&mut visible.textures);
        for id in &textures.free {
            rpass.free_texture(id);
        }
    }

    pub fn take_event(&mut self) -> Option<Event> {
        self.gui.emitted_event.take()
    }
}

impl State {
    fn new() -> Self {
        Self {
            emitted_event: None,
        }
    }

    fn ui(&mut self, ctx: &Context) {
        egui::Window::new("options").show(ctx, |ui| {
            if ui.button("color").clicked() {
                log::debug!("color");
                self.emit(Event::ChangeColor);
            }
            if ui.button("clear").clicked() {
                log::debug!("clear");
                self.emit(Event::ClearAll);
            }
            if ui.button("shrink").clicked() {
                log::debug!("shrink");
                self.emit(Event::ShrinkPen);
            }
            if ui.button("grow").clicked() {
                log::debug!("grow");
                self.emit(Event::GrowPen);
            }
        });
    }

    fn emit(&mut self, event: Event) {
        self.emitted_event = Some(event);
    }
}

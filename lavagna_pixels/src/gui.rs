#![allow(unused)] // When feature gui is disabled, most of this module is unused
use pixels::{wgpu, Pixels, PixelsContext};
use winit::window::Window;
use winit::{dpi::PhysicalSize, event_loop::EventLoopWindowTarget};

use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_wgpu::renderer::{RenderPass, ScreenDescriptor};

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Gui {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: Option<RenderPass>,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: State,
}

struct State {
    emitted_event: Option<Event>,
}

pub enum Event {
    ChangeColor,
    ClearAll,
    ShrinkPen,
    GrowPen,
}

impl Gui {
    /// Create egui.
    pub(crate) fn new<T>(event_loop: &EventLoopWindowTarget<T>, width: u32, height: u32) -> Self {
        let scale_factor = 3.0;
        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let textures = TexturesDelta::default();
        let gui = State::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass: None,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    pub(crate) fn set_pixels(&mut self, pixels: &Pixels) {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;
        self.egui_state.set_max_texture_side(max_texture_size);
        self.rpass = Some(RenderPass::new(
            pixels.device(),
            pixels.render_texture_format(),
            1,
        ));
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.screen_descriptor.size_in_pixels = [size.width, size.height];
        }
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        if self.rpass.is_none() {
            return;
        }

        let rpass = self.rpass.as_mut().unwrap();

        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            rpass.update_texture(&context.device, &context.queue, *id, image_delta);
        }
        rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        );

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
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

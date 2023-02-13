use std::{rc::Rc, time::Instant};

use anyhow::Result;
use glium::{
    self,
    backend::Facade,
    glutin::{
        self,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    Display, Surface,
};
use imgui::{FontConfig, FontSource, Textures};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

pub(crate) struct System {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
}

impl System {
    pub fn new(title: &str) -> Result<Self> {
        let event_loop = EventLoop::new();
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::LogicalSize::new(1024, 768));
        let display = Display::new(window_builder, context, &event_loop)?;

        let mut imgui = imgui::Context::create();

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
        }

        let font_size = 13.0;
        imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../resources/Roboto-Regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                // make the font a bit heavier to compensate for lack of gamma correction
                rasterizer_multiply: 1.5,
                // oversample to improve quality at the cost of a larger font atlas
                oversample_h: 4,
                oversample_v: 4,
                ..Default::default()
            }),
        }]);

        let renderer = Renderer::init(&mut imgui, &display)?;

        Ok(System {
            event_loop,
            display,
            imgui,
            platform,
            renderer,
        })
    }

    pub fn main_loop<R>(mut self, mut run_ui: R)
    where
        R: FnMut(
                &mut imgui::Ui,
                &mut Textures<Texture>,
                &Rc<glium::backend::Context>,
            ) -> Option<ControlFlow>
            + 'static,
    {
        let mut last_frame = Instant::now();

        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    self.imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::MainEventsCleared => {
                    let gl_window = self.display.gl_window();
                    self.platform
                        .prepare_frame(self.imgui.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let gl_ctx = self.display.get_context();
                    let textures = self.renderer.textures();
                    let ui = self.imgui.frame();

                    if let Some(cf) = run_ui(ui, textures, gl_ctx) {
                        *control_flow = cf;
                    }

                    let gl_window = self.display.gl_window();
                    let mut target = self.display.draw();
                    target.clear_color_srgb(0.015, 0.015, 0.02, 1.0);
                    self.platform.prepare_render(ui, gl_window.window());
                    let draw_data = self.imgui.render();
                    self.renderer
                        .render(&mut target, draw_data)
                        .expect("Rending failed");
                    target.finish().expect("Failed to swap buffers");
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                event => {
                    let gl_window = self.display.gl_window();
                    self.platform
                        .handle_event(self.imgui.io_mut(), gl_window.window(), &event);
                }
            });
    }
}

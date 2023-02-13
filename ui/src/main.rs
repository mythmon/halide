use std::rc::Rc;
use anyhow::Result;
use glium::{backend::Facade, texture::RawImage2d, uniforms::SamplerBehavior};
use halide_raytracer::Renderer;
use imgui::{Condition, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;
use system::System;
use timer::Timer;

mod system;
mod timer;

fn main() -> Result<()> {
    let system = System::new("Halide")?;
    let mut interface = App::default();

    system.main_loop(move |ui, textures, gl_ctx| {
        interface.on_ui_render(ui, textures, gl_ctx);
        None
    });

    Ok(())
}

struct App {
    viewport_id: Option<TextureId>,
    viewport_size: [f32; 2],
    image_size: [f32; 2],
    timer: Timer,
    renderer: Renderer,
}

impl Default for App {
    fn default() -> Self {
        Self {
            viewport_id: None,
            viewport_size: [400.0, 400.0],
            image_size: [0.0, 0.0],
            timer: Timer::new(),
            renderer: Renderer::new(400, 400)
        }
    }
}

impl App {
    fn on_ui_render<F: Facade>(
        &mut self,
        ui: &mut Ui,
        textures: &mut Textures<Texture>,
        gl_ctx: &F,
    ) {
        self.render(textures, gl_ctx).ok();

        {
            let _padding_style = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
            ui.window("Viewport")
                .size(self.viewport_size, Condition::FirstUseEver)
                .scroll_bar(false)
                .build(|| {
                    self.viewport_size = ui.content_region_avail();
                    if let Some(viewport_id) = self.viewport_id {
                        imgui::Image::new(viewport_id, self.image_size)
                            // flip Y-coordinate
                            .uv0([0., 1.])
                            .uv1([1., 0.])
                        .build(ui);
                    }
                });
        }

        ui.window("Settings")
            .size([300.0, 110.0], Condition::FirstUseEver)
            .build(|| {
                if ui.button("Render") {
                    self.render(textures, gl_ctx).expect("could not render");
                }
                ui.text(format!("viewport: {:.0}x{:.0}", self.viewport_size[0], self.viewport_size[1]));


                if !self.timer.is_empty() {
                    ui.text("Last render:");
                    for (name, duration) in self.timer.get_durations() {
                        ui.text(format!("  {name}: {:.1}ms", duration.as_secs_f32() * 1000.0));
                    }
                }
            });
    }

    fn render<F: Facade>(&mut self, textures: &mut Textures<Texture>, gl_ctx: &F) -> Result<()> {
        self.timer.reset();
        let width = self.viewport_size[0] as u32;
        let height = self.viewport_size[1] as u32;

        self.renderer.resize(width, height);
        let data = self.renderer.render();

        self.timer.stage_end("generate data");

        let raw = RawImage2d {
            data,
            width,
            height,
            format: glium::texture::ClientFormat::U8U8U8U8,
        };
        let gl_texture = glium::Texture2d::with_mipmaps(gl_ctx, raw, glium::texture::MipmapsOption::NoMipmap)?;
        let texture = Texture {
            texture: Rc::new(gl_texture),
            sampler: SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Linear,
                minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                ..Default::default()
            },
        };
        self.timer.stage_end("update texture");

        self.viewport_id = Some(textures.insert(texture));
        self.image_size = self.viewport_size;

        Ok(())
    }
}

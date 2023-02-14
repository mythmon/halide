use anyhow::Result;
use glam::Vec3;
use glium::{backend::Facade, texture::RawImage2d, uniforms::SamplerBehavior};
use halide_raytracer::{Camera, Renderer, Scene};
use imgui::{Condition, TextureId, Textures, Ui};
use imgui_glium_renderer::Texture;
use std::rc::Rc;
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
    scene: Scene,
    camera: Camera,
    sphere_color_ui: [f32; 3],
    light_direction_ui: [f32; 3],
    camera_position_ui: [f32; 3],
    camera_direction_ui: [f32; 3],
}

impl Default for App {
    fn default() -> Self {
        let mut rv = Self {
            viewport_id: None,
            viewport_size: [400.0, 400.0],
            image_size: [0.0, 0.0],
            timer: Timer::new(),
            renderer: Renderer::new(400, 400),
            scene: Scene::default(),
            camera: Camera::default(),
            sphere_color_ui: [0.0; 3],
            light_direction_ui: [0.0; 3],
            camera_position_ui: [0.0; 3],
            camera_direction_ui: [0.0; 3],
        };

        rv.scene
            .sphere_color()
            .write_to_slice(&mut rv.sphere_color_ui);
        rv.scene
            .light_direction()
            .write_to_slice(&mut rv.light_direction_ui);
        rv.camera
            .position()
            .write_to_slice(&mut rv.camera_position_ui);
        rv.camera
            .look_direction()
            .write_to_slice(&mut rv.camera_direction_ui);

        rv
    }
}

impl App {
    fn on_ui_render<F: Facade>(
        &mut self,
        ui: &mut Ui,
        textures: &mut Textures<Texture>,
        gl_ctx: &F,
    ) {
        {
            // scope for style tokens
            let _padding_style = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
            ui.window("Viewport")
                .size(self.viewport_size, Condition::FirstUseEver)
                .scroll_bar(false)
                .build(|| {
                    self.render(textures, gl_ctx).ok();
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

        ui.window("Debug")
            .size([200.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(format!(
                    "Viewport size: {:.0}x{:.0}",
                    self.viewport_size[0], self.viewport_size[1]
                ));
                ui.text("Last render:");
                for (name, duration) in self.timer.get_durations() {
                    ui.text(format!(
                        "  {name}: {:.1}ms",
                        duration.as_secs_f32() * 1000.0
                    ));
                }
            });

        ui.window("Settings")
            .size([300., 300.], Condition::FirstUseEver)
            .build(|| {
                if ui.color_edit3("Sphere color", &mut self.sphere_color_ui) {
                    self.scene
                        .set_sphere_color(Vec3::from(self.sphere_color_ui));
                };

                if imgui::Drag::new("Light direction")
                    .range(-1., 1.)
                    .speed(0.01)
                    .build_array(ui, &mut self.light_direction_ui)
                {
                    self.scene
                        .set_light_direction(Vec3::from(self.light_direction_ui));
                }

                if imgui::Drag::new("Camera position")
                    .range(-10., 10.)
                    .speed(0.1)
                    .build_array(ui, &mut self.camera_position_ui)
                {
                    self.camera
                        .set_position(Vec3::from(self.camera_position_ui))
                }

                if imgui::Drag::new("Camera direction")
                    .range(-1., 1.)
                    .speed(0.01)
                    .build_array(ui, &mut self.camera_direction_ui)
                {
                    self.camera
                        .set_look_direction(Vec3::from(self.camera_direction_ui));
                }
            });
    }

    fn render<F: Facade>(&mut self, textures: &mut Textures<Texture>, gl_ctx: &F) -> Result<()> {
        self.timer.reset();
        let width = self.viewport_size[0] as u32;
        let height = self.viewport_size[1] as u32;

        self.renderer.resize(width, height);
        self.camera.set_size(width, height);
        let data = self.renderer.render(&self.scene, &self.camera);

        self.timer.stage_end("generate data");

        let raw = RawImage2d {
            data,
            width,
            height,
            format: glium::texture::ClientFormat::U8U8U8U8,
        };
        let gl_texture =
            glium::Texture2d::with_mipmaps(gl_ctx, raw, glium::texture::MipmapsOption::NoMipmap)?;
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

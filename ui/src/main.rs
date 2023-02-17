use anyhow::Result;
use glam::Vec3;
use glium::{backend::Facade, texture::RawImage2d, uniforms::SamplerBehavior};
use halide_raytracer::{Camera, Material, Renderer, Scene, Sphere};
use imgui::{Condition, Key, MouseButton, TextureId, Textures};
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
}

impl Default for App {
    fn default() -> Self {
        let mut scene = Scene::default();

        let ground_material = scene.add_material(Material {
            albedo: Vec3::new(0.7, 0.7, 0.7),
            ..Default::default()
        });
        let ball_material = scene.add_material(Material {
            albedo: Vec3::new(0.9, 0.2, 0.1),
            ..Default::default()
        });

        scene.add_sphere(Sphere {
            center: Vec3::new(0., -10_000., 0.),
            radius: 10_000.,
            material_index: ground_material,
        });

        scene.add_sphere(Sphere {
            center: Vec3::new(0., 0.5, 0.),
            radius: 0.5,
            material_index: ball_material,
        });

        let mut camera = Camera::default();
        camera.set_position((0., 0.75, 4.).into());

        Self {
            viewport_id: None,
            viewport_size: [400.0, 400.0],
            image_size: [0.0, 0.0],
            timer: Timer::new(),
            renderer: Renderer::new(400, 400),
            scene,
            camera,
        }
    }
}

impl App {
    fn on_ui_render<F: Facade>(
        &mut self,
        ui: &mut imgui::Ui,
        textures: &mut Textures<Texture>,
        gl_ctx: &F,
    ) {
        let dt = ui.io().delta_time;
        let mut camera_offset = Vec3::ZERO;
        let mut camera_rotate = [0.0, 0.0];

        let mut camera_position_ui: Vec3 = self.camera.position();
        let mut camera_direction_ui: Vec3 = self.camera.look_direction();
        let mut light_direction_ui: Vec3 = self.scene.light_direction();

        if ui.is_mouse_down(MouseButton::Right) {
            if ui.is_key_down(Key::D) {
                camera_offset += Vec3::X;
            }
            if ui.is_key_down(Key::A) {
                camera_offset += Vec3::NEG_X;
            }
            if ui.is_key_down(Key::E) {
                camera_offset += Vec3::Y;
            }
            if ui.is_key_down(Key::Q) {
                camera_offset += Vec3::NEG_Y;
            }
            if ui.is_key_down(Key::W) {
                camera_offset += Vec3::Z;
            }
            if ui.is_key_down(Key::S) {
                camera_offset += Vec3::NEG_Z;
            }

            let drag = ui.mouse_drag_delta_with_button(MouseButton::Right);
            ui.reset_mouse_drag_delta(MouseButton::Right);
            if drag[0].abs() > 0. || drag[1].abs() > 0. {
                camera_rotate = [-drag[1], -drag[0]];
            }

            if camera_offset != Vec3::ZERO {
                camera_offset = camera_offset.normalize();
                camera_position_ui = *self.camera.relative_move(camera_offset, dt);
                self.renderer.reset_accumulation();
            }
            if camera_rotate != [0.0, 0.0] {
                self.camera
                    .relative_turn(camera_rotate, dt)
                    .write_to_slice(camera_direction_ui.as_mut());
                self.renderer.reset_accumulation();
            }
        }

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
                if imgui::Drag::new("Light direction")
                    .range(-1., 1.)
                    .speed(0.01)
                    .build_array(ui, light_direction_ui.as_mut())
                {
                    self.scene.set_light_direction(light_direction_ui);
                    self.renderer.reset_accumulation();
                }

                ui.checkbox("Accumulation", &mut self.renderer.use_accumulation);
                ui.same_line();
                if ui.button("Reset") {
                    self.renderer.reset_accumulation()
                }

                let mut local_num_threads = self.renderer.num_threads();
                if imgui::Drag::new("Thread count")
                    .range(1, num_cpus::get() * 2)
                    .speed(0.15)
                    .build(ui, &mut local_num_threads)
                {
                    self.renderer.set_num_threads(local_num_threads);
                }

                if imgui::Drag::new("Camera position")
                    .range(-10., 10.)
                    .speed(0.1)
                    .build_array(ui, camera_position_ui.as_mut())
                {
                    self.camera.set_position(camera_position_ui);
                    self.renderer.reset_accumulation();
                }

                if imgui::Drag::new("Camera direction")
                    .range(-1., 1.)
                    .speed(0.01)
                    .build_array(ui, camera_direction_ui.as_mut())
                {
                    self.camera.set_look_direction(camera_direction_ui);
                    self.renderer.reset_accumulation();
                }

                ui.separator();

                let sphere_count = self.scene.spheres().len();
                let material_count = self.scene.materials().len();
                for (idx, sphere) in self.scene.spheres_mut().iter_mut().enumerate() {
                    let _id = ui.push_id_usize(idx);
                    ui.text(format!("Sphere {idx}"));
                    if imgui::Drag::new("Position")
                        .range((-10.0..10.0).start, (-10.0..10.0).end)
                        .speed(0.1)
                        .build_array(ui, sphere.center.as_mut())
                    {
                        self.renderer.reset_accumulation();
                    }
                    if imgui::Drag::new("Radius")
                        .range(0.1, 3.0)
                        .speed(0.03)
                        .build(ui, &mut sphere.radius)
                    {
                        self.renderer.reset_accumulation();
                    }
                    if imgui::Drag::new("Material")
                        .range(0, material_count - 1)
                        .speed(0.1)
                        .build(ui, &mut sphere.material_index)
                    {
                        self.renderer.reset_accumulation();
                    }
                }

                ui.separator();

                for (idx, material) in self.scene.materials_mut().iter_mut().enumerate() {
                    let _id = ui.push_id_usize(idx);
                    ui.text(format!("Material {idx}"));
                    if ui.color_edit3("Albedo", material.albedo.as_mut()) {
                        self.renderer.reset_accumulation();
                    }
                    if imgui::Drag::new("Roughness")
                        .range(0.0, 1.0)
                        .speed(0.01)
                        .build(ui, &mut material.roughness)
                    {
                        self.renderer.reset_accumulation();
                    }
                    if imgui::Drag::new("Metallic")
                        .range(0.0, 1.0)
                        .speed(0.01)
                        .build(ui, &mut material.metallic)
                    {
                        self.renderer.reset_accumulation();
                    }
                    if idx < sphere_count - 1 {
                        ui.separator();
                    }
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

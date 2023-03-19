use crate::{geom::Ray, hittable::HitPayload, util::color_rgb, Camera, Scene};
use glam::Vec3;
use rayon::{prelude::*, ThreadPool};
use std::borrow::Cow;

pub struct Renderer {
    image_data: Vec<u32>,
    accumulation: Vec<Vec3>,
    frame_count: f32,
    width: u32,
    height: u32,
    pub use_accumulation: bool,
    pool: ThreadPool,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        let length = width as usize * height as usize;
        let mut accumulation = Vec::with_capacity(length);
        accumulation.resize(length, Vec3::ZERO);

        Self {
            image_data: Vec::with_capacity(width as usize * height as usize),
            accumulation,
            frame_count: 0.,
            width,
            height,
            use_accumulation: true,
            pool: rayon::ThreadPoolBuilder::default().build().unwrap(),
        }
    }

    #[inline(always)]
    fn image_len(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if (self.width, self.height) != (width, height) {
            self.width = width;
            self.height = height;
            self.reset_accumulation();
            self.image_data.truncate(0);
            self.image_data.resize(self.image_len(), 0);
        }
    }

    pub fn reset_accumulation(&mut self) {
        self.accumulation.truncate(0);
        self.accumulation.resize(self.image_len(), Vec3::ZERO);
        self.frame_count = 0.0;
    }

    pub fn num_threads(&self) -> usize {
        self.pool.current_num_threads()
    }

    pub fn set_num_threads(&mut self, num_threads: usize) {
        self.pool = rayon::ThreadPoolBuilder::default()
            .num_threads(num_threads)
            .build()
            .unwrap();
    }

    pub fn render<'a>(&mut self, scene: &'a Scene, camera: &'a Camera) -> Cow<[u32]> {
        self.render_accumulate(scene, camera, 1)
    }

    pub fn render_accumulate<'a>(
        &mut self,
        scene: &'a Scene,
        camera: &'a Camera,
        frames: usize,
    ) -> Cow<[u32]> {
        let ctx = RenderFrame { scene, camera };

        if !self.use_accumulation {
            self.reset_accumulation();
        }

        self.image_data.resize(self.image_len(), 0);
        self.accumulation.resize(self.image_len(), Vec3::ZERO);

        for _ in 0..frames {
            self.frame_count += 1.;

            let dirs = camera.get_ray_directions();
            let rays = dirs
                .iter()
                .map(|direction| Ray {
                    direction: *direction,
                    origin: camera.position(),
                })
                .collect::<Vec<_>>();

            self.image_data.resize(self.image_len(), 0);
            self.pool.install(|| {
                (&mut self.accumulation, rays)
                    .into_par_iter()
                    .for_each(|(acc, ray)| {
                        *acc += ctx.per_pixel(ray);
                    });
            });
        }

        let frame_count = self.frame_count;
        self.pool.install(|| {
            (&mut self.accumulation, &mut self.image_data)
                .into_par_iter()
                .for_each(|(acc, output)| {
                    *output = color_rgb(*acc / frame_count);
                });
        });

        Cow::Borrowed(self.image_data.as_slice())
    }
}

struct RenderFrame<'a> {
    scene: &'a Scene,
    camera: &'a Camera,
}

impl<'a> RenderFrame<'a> {
    /// Called once per pixel to figure out its color.
    fn per_pixel(&self, ray: Ray) -> Vec3 {
        self.ray_color(ray, 16)
    }

    fn ray_color(&self, ray: Ray, bounce_budget: u32) -> Vec3 {
        const SKY_COLOR: Vec3 = Vec3::new(0.6, 0.7, 0.9);

        if bounce_budget == 0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            match self.trace_ray(&ray) {
                ref hit @ HitPayload::Hit { ref material_index, .. } => {
                    let material = self.scene.material(*material_index);
                    if let Some(scatter) = material.scatter(hit, &ray) {
                        self.ray_color(scatter.ray, bounce_budget - 1) * scatter.attenuation
                    } else {
                        Vec3::ZERO
                    }
                }
                HitPayload::Miss => SKY_COLOR,
                HitPayload::Inside => Vec3::ZERO,
            }
        }
    }

    /// Shoot a ray from a given location and return information the closest hit, if any.
    fn trace_ray(&self, ray: &Ray) -> HitPayload {
        let look_clip = self.camera.look_clip();
        self.scene
            .hittables()
            .iter()
            .map(|hittable| hittable.check_hit(ray, look_clip))
            .fold(HitPayload::Miss, |acc, next| {
                match (acc, next) {
                    (acc @ HitPayload::Hit { .. }, next @ HitPayload::Hit { .. }) => {
                        match (&acc, &next) {
                            (
                                HitPayload::Hit {
                                    hit_distance: d_acc,
                                    ..
                                },
                                HitPayload::Hit {
                                    hit_distance: d_next,
                                    ..
                                },
                            ) if d_next < d_acc => next,
                            _ => acc,
                        }
                    }
                    (hit @ HitPayload::Hit { .. }, HitPayload::Miss)
                    | (HitPayload::Hit { .. }, hit @ HitPayload::Inside)
                    | (HitPayload::Miss, hit @ HitPayload::Hit { .. })
                    | (hit @ HitPayload::Miss, HitPayload::Miss)
                    | (HitPayload::Miss, hit @ HitPayload::Inside)
                    | (hit @ HitPayload::Inside, HitPayload::Hit { .. })
                    | (hit @ HitPayload::Inside, HitPayload::Miss)
                    | (hit @ HitPayload::Inside, HitPayload::Inside) => hit,
                }
            })
    }
}

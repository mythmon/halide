use crate::{
    geom::Ray,
    util::{color_rgb, Vec3Ext},
    Camera, Scene,
};
use glam::Vec3;
use rand::Rng;
use rayon::{prelude::*, ThreadPool};
use std::{borrow::Cow, cmp::Ordering};

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

#[derive(PartialEq)]
pub enum HitPayload {
    Hit {
        /// The proportion along the ray, not a world distance.
        hit_distance: f32,
        world_normal: Vec3,
        world_position: Vec3,
        // object_index: usize,
        material_index: usize,
    },
    Miss,
}

impl PartialOrd for HitPayload {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (
                HitPayload::Hit {
                    hit_distance: d_self,
                    ..
                },
                HitPayload::Hit {
                    hit_distance: d_other,
                    ..
                },
            ) => d_self.partial_cmp(d_other),
            (HitPayload::Hit { .. }, HitPayload::Miss) => Some(Ordering::Less),
            (HitPayload::Miss, HitPayload::Hit { .. }) => Some(Ordering::Greater),
            (HitPayload::Miss, HitPayload::Miss) => {
                if self == other {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a> RenderFrame<'a> {
    /// Called once per pixel to figure out its color.
    fn per_pixel(&self, ray: Ray) -> Vec3 {
        self.ray_color(ray, 16)
    }

    fn ray_color(&self, mut ray: Ray, bounce_budget: u32) -> Vec3 {
        const SKY_COLOR: Vec3 = Vec3::new(0.6, 0.7, 0.9);

        if bounce_budget == 0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            match self.trace_ray(&ray) {
                HitPayload::Hit {
                    world_normal,
                    world_position,
                    material_index,
                    ..
                } => {
                    let material = self.scene.material(material_index);

                    let mut rng = rand::thread_rng();
                    let normal_offset: Vec3 = material.roughness * rng.gen::<Vec3>();
                    let reflection_normal = (world_normal + normal_offset)
                        .try_normalize()
                        .unwrap_or(world_normal);
                    ray.direction = (-ray.direction).reflect(reflection_normal);
                    ray.origin = world_position + ray.direction * 0.0001;

                    self.ray_color(ray, bounce_budget - 1) * material.albedo
                }
                HitPayload::Miss => SKY_COLOR,
            }
        }
    }

    /// Shoot a ray from a given location and return information about any potential hits.
    fn trace_ray(&self, ray: &Ray) -> HitPayload {
        let look_clip = self.camera.look_clip();
        self.scene
            .hittables()
            .iter()
            .map(|hittable| hittable.check_hit(ray, look_clip))
            .fold(
                HitPayload::Miss,
                |acc, next| if next < acc { next } else { acc },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::HitPayload;
    use glam::Vec3;

    #[test]
    fn order_of_hits() {
        let x = HitPayload::Hit {
            hit_distance: 1.0,
            world_normal: Vec3::X,
            world_position: Vec3::ZERO,
            material_index: 0,
        };
        let y = HitPayload::Hit {
            hit_distance: 2.0,
            world_normal: Vec3::X,
            world_position: Vec3::ZERO,
            material_index: 0,
        };
        let z = HitPayload::Miss;

        assert!(x < y);
        assert!(x < z);
        assert!(y < z);
    }
}

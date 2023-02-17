use crate::{
    geom::Ray,
    util::{color_rgb, Vec3Ext},
    Camera, Scene,
};
use glam::Vec3;
use rand::Rng;
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
        self.pool = rayon::ThreadPoolBuilder::default().num_threads(num_threads).build().unwrap();
    }

    pub fn render<'a>(&mut self, scene: &'a Scene, camera: &'a Camera) -> Cow<[u32]> {
        let ctx = RenderFrame { scene, camera };

        if !self.use_accumulation {
            self.reset_accumulation();
        }

        self.image_data.resize(self.image_len(), 0);
        self.accumulation.resize(self.image_len(), Vec3::ZERO);
        self.frame_count += 1.;

        let frame_count = self.frame_count;
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
            (&mut self.accumulation, &mut self.image_data, rays)
                .into_par_iter()
                .for_each(|(acc, output, ray)| {
                    *acc += per_pixel(&ctx, ray);
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

impl<'a> RenderFrame<'a> {}

/// Called once per pixel. Generates rays.
fn per_pixel(ctx @ RenderFrame { scene, .. }: &RenderFrame, mut ray: Ray) -> Vec3 {
    const BOUNCES: u32 = 16;
    const SKY_COLOR: Vec3 = Vec3::new(0.6, 0.7, 0.9);
    let mut multiplier = 1.0;

    let mut final_color = Vec3::ZERO;
    let mut rng = rand::thread_rng();
    for _ in 0..BOUNCES {
        match trace_ray(ctx, &ray) {
            HitPayload::Hit {
                object_index,
                world_normal,
                world_position,
            } => {
                let sphere = &scene.sphere(object_index);
                let material = &scene.material(sphere.material_index);
                let light_intensity = world_normal.dot(-scene.light_direction()).max(0.0);
                let color = material.albedo * light_intensity;
                final_color += color * multiplier;
                multiplier *= 0.7;

                ray.origin = world_position + world_normal * 0.0001;
                let normal_offset: Vec3 = 0.5 * material.roughness * rng.gen::<Vec3>();
                let reflection_normal = (world_normal + normal_offset).normalize();
                ray.direction = ray.direction.reflect(reflection_normal);
            }
            HitPayload::Miss => {
                final_color += SKY_COLOR * multiplier;
                break;
            }
        };
    }

    final_color
}

enum HitPayload {
    Hit {
        // hit_distance: f32,
        world_normal: Vec3,
        world_position: Vec3,
        object_index: usize,
    },
    Miss,
}

/// Shoot a ray from a given location and return information about any potential hits.
fn trace_ray(ctx @ RenderFrame { scene, camera }: &RenderFrame, ray: &Ray) -> HitPayload {
    let mut closest = None;

    for (index, sphere) in scene.spheres().iter().enumerate() {
        let offset_center = ray.origin - sphere.center;

        // solve the equation of the ray set equal to the equation of a sphere centered on the origin.
        // a, b, and c are the quadratic equation co-effiecients
        let a = ray.direction.length_squared();
        let b = 2. * ray.direction.dot(offset_center);
        let c = offset_center.length_squared() - sphere.radius.powi(2);

        let discrim = b.powi(2) - 4. * a * c;

        if discrim < 0. {
            continue;
        } else {
            // finish the quadratic equation, though we only need the least result
            let t0 = (-b - discrim.sqrt()) / (2. * a);
            if camera.look_clip().contains(&t0) {
                match closest {
                    Some((_, min_t)) if t0 < min_t => {
                        closest = Some((index, t0));
                    }
                    None => {
                        closest = Some((index, t0));
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some((object_index, t)) = closest {
        on_hit(ctx, ray, object_index, t)
    } else {
        on_miss(ray)
    }
}

/// invoked when something is hit
fn on_hit(
    RenderFrame { scene, .. }: &RenderFrame,
    ray: &Ray,
    object_index: usize,
    hit_distance: f32,
) -> HitPayload {
    let sphere = &scene.spheres()[object_index];
    let hit_pos = ray.origin + ray.direction * hit_distance;
    let world_normal = (hit_pos - sphere.center).normalize();
    HitPayload::Hit {
        // hit_distance,
        world_normal,
        world_position: hit_pos,
        object_index,
    }
}

/// invoked when a ray misses all objects
fn on_miss(_ray: &Ray) -> HitPayload {
    HitPayload::Miss
}

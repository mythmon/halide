use crate::{
    geom::Ray,
    util::{color_rgb, Vec3Ext},
    Camera, Scene,
};
use glam::Vec3;
use std::borrow::Cow;

pub struct Renderer {
    // in ABGR order
    pub(crate) image_data: Vec<u32>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            image_data: Vec::default(),
            width,
            height,
        }
    }

    #[inline(always)]
    pub(crate) fn image_len(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if (self.width, self.height) != (width, height) {
            self.image_data.truncate(0);
            self.image_data.reserve(self.image_len());
            self.width = width;
            self.height = height;
        }
    }

    pub fn render<'a>(&mut self, scene: &'a Scene, camera: &'a Camera) -> Cow<[u32]> {
        let ctx = RenderFrame { scene, camera };

        self.image_data.truncate(0);
        self.image_data
            .reserve_exact((self.width * self.height) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                let color = per_pixel(&ctx, x, y);
                self.image_data.push(color_rgb(&color));
            }
        }

        Cow::Borrowed(self.image_data.as_slice())
    }
}

struct RenderFrame<'a> {
    scene: &'a Scene,
    camera: &'a Camera,
}

impl<'a> RenderFrame<'a> {}

/// Called once per pixel. Generates rays.
fn per_pixel(ctx @ RenderFrame { scene, camera }: &RenderFrame, x: u32, y: u32) -> Vec3 {
    let mut ray = Ray {
        origin: camera.position(),
        direction: *camera.get_ray_direction(x, y),
    };
    const BOUNCES: u32 = 2;
    const SKY_COLOR: Vec3 = Vec3::ZERO;
    let mut multiplier = 1.0;

    let mut final_color = Vec3::ZERO;
    for _ in 0..BOUNCES {
        match trace_ray(ctx, &ray) {
            HitPayload::Hit {
                object_index,
                world_normal,
                world_position,
            } => {
                let sphere = &scene.spheres()[object_index];
                let light_intensity = world_normal.dot(-scene.light_direction()).max(0.0);
                let color = sphere.albedo * light_intensity;
                final_color += color * multiplier;
                multiplier *= 0.7;

                ray.origin = world_position + world_normal * 0.0001;
                ray.direction = ray.direction.reflect(world_normal);
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

use crate::{geom::Ray, util::color_rgb, Camera, Scene};
use glam::Vec3;
use std::borrow::Cow;

pub struct Renderer {
    // in ABGR order
    pub(crate) image_data: Vec<u32>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

struct RenderContext<'a> {
    scene: &'a Scene,
    ray: Ray,
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
        let mut ctx = RenderContext {
            scene,
            ray: Ray {
                origin: camera.position(),
                ..Default::default()
            },
        };

        let dirs = camera.get_ray_directions();
        self.image_data.truncate(0);
        let iter = dirs.iter().map(|ray_dir| {
            ctx.ray.direction = *ray_dir;
            let color = trace_ray(&ctx).clamp(Vec3::ZERO, Vec3::ONE);
            color_rgb(&color)
        });
        self.image_data.extend(iter);

        Cow::Borrowed(self.image_data.as_slice())
    }
}

fn trace_ray(RenderContext { scene, ray }: &RenderContext) -> Vec3 {
    // solve the equation of the ray set equal to the equation of a sphere centered on the origin.
    // a, b, and c are the quadratic equation co-effiecients
    let a = ray.direction.length_squared();
    let b = 2. * ray.direction.dot(ray.origin);
    let c = ray.origin.length_squared() - scene.sphere_radius().powi(2);

    let discrim = b.powi(2) - 4. * a * c;

    if discrim < 0. {
        Vec3::ZERO
    } else {
        // finish the quadratic equation, though we only need the least result
        let t0 = (-b - discrim.sqrt()) / (2. * a);
        let hit_pos = ray.origin + ray.direction * t0;
        let normal = hit_pos.normalize();
        let d = normal.dot(-scene.light_direction()).max(0.0);
        scene.sphere_color() * d
    }
}

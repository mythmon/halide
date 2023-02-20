use std::ops::Range;
use crate::{geom::Ray, renderer::HitPayload, Sphere};

pub enum Hittable {
    Sphere(Sphere),
}

impl Hittable {
    pub fn check_hit(&self, ray: &Ray, look_clip: &Range<f32>) -> HitPayload {
        match self {
            Hittable::Sphere(sphere) => Self::check_hit_sphere(sphere, ray, look_clip)
        }
    }

    fn check_hit_sphere(sphere: &Sphere, ray: &Ray, look_clip: &Range<f32>) -> HitPayload {
        let offset_center = ray.origin - sphere.center;

        // solve the equation of the ray set equal to the equation of a sphere centered on the origin.
        // a, b, and c are the quadratic equation co-effiecients
        let a = ray.direction.length_squared();
        let b = 2. * ray.direction.dot(offset_center);
        let c = offset_center.length_squared() - sphere.radius.powi(2);

        let discrim = b.powi(2) - 4. * a * c;

        if discrim < 0. {
            HitPayload::Miss
        } else {
            // finish the quadratic equation, though we only need the least result
            let t0 = (-b - discrim.sqrt()) / (2. * a);
            if look_clip.contains(&t0) {
                let world_position = ray.origin + ray.direction * t0;
                HitPayload::Hit {
                    hit_distance: t0,
                    world_normal: (world_position - sphere.center).normalize(),
                    world_position,
                    material_index: sphere.material_index,
                }
            } else {
                HitPayload::Miss
            }
        }
    }
}

impl From<Sphere> for Hittable {
    fn from(value: Sphere) -> Self {
        Self::Sphere(value)
    }
}
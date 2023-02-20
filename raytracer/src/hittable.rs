use glam::Vec3;
use std::ops::Range;

use crate::{geom::Ray, Sphere};

pub enum Hittable {
    Sphere(Sphere),
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum FaceSide {
    Front,
    Back,
}

#[derive(PartialEq)]
pub enum HitPayload {
    Hit {
        /// The proportion along the ray, not a world distance.
        hit_distance: f32,
        world_normal: Vec3,
        world_position: Vec3,
        material_index: usize,
        side: FaceSide,
    },
    Miss,
    Inside,
}

impl Hittable {
    #[inline]
    pub fn check_hit(&self, ray: &Ray, look_clip: &Range<f32>) -> HitPayload {
        match self {
            Hittable::Sphere(sphere) => Self::check_hit_sphere(sphere, ray, look_clip),
        }
    }

    #[inline]
    fn check_hit_sphere(sphere: &Sphere, ray: &Ray, look_clip: &Range<f32>) -> HitPayload {
        let offset_center = ray.origin - sphere.center;

        if offset_center.length() < sphere.radius {
            HitPayload::Inside
        } else {
            // solve the equation of the ray set equal to the equation of a sphere centered on the origin.
            // a, b, and c are the quadratic equation co-effiecients
            let a = ray.direction.length_squared();
            let half_b = offset_center.dot(ray.direction);
            let c = offset_center.length_squared() - sphere.radius.powi(2);

            let discrim = half_b.powi(2) - a * c;

            if discrim < 0. {
                HitPayload::Miss
            } else {
                // finish the quadratic equation, though we only need the least result
                let sqrtd = discrim.sqrt();

                let mut t = (-half_b - sqrtd) / a;
                if !look_clip.contains(&t) {
                    t = (-half_b + sqrtd) / a;
                }

                if look_clip.contains(&t) {
                    let world_position = ray.origin + ray.direction * t;
                    let world_normal = (world_position - sphere.center).normalize();

                    let (side, outward_normal) = if ray.direction.dot(world_normal) > 0.0 {
                        (FaceSide::Back, -world_normal)
                    } else {
                        (FaceSide::Front, world_normal)
                    };

                    HitPayload::Hit {
                        hit_distance: t,
                        world_normal: outward_normal,
                        world_position,
                        material_index: sphere.material_index,
                        side,
                    }
                } else {
                    HitPayload::Miss
                }
            }
        }
    }
}

impl From<Sphere> for Hittable {
    fn from(value: Sphere) -> Self {
        Self::Sphere(value)
    }
}

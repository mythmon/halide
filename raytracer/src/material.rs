use glam::Vec3;

use crate::{geom::Ray, hittable::HitPayload, util::Vec3Ext};

pub enum Material {
    Null,
    Lambertian { albedo: Vec3 }
}

pub struct ScatterPayload {
    pub ray: Ray,
    pub attenuation: Vec3,
}

impl Material {
    #[inline]
    pub fn scatter(&self, hit: &HitPayload, _ray: &Ray) -> Option<ScatterPayload> {
        match self {
            Material::Null => None,
            Material::Lambertian { albedo } => self.scatter_lambertian(hit, albedo)
        }
    }

    #[inline]
    fn scatter_lambertian(&self, hit: &HitPayload, albedo: &Vec3) -> Option<ScatterPayload> {
        match hit {
            HitPayload::Hit { world_normal, world_position, .. } => {
                let mut rng = rand::thread_rng();
                let direction = (*world_normal + Vec3::random_unit(&mut rng)).normalize();
                let scatter_ray = Ray {
                    origin: *world_position + direction * 0.001,
                    direction,
                };
                Some(ScatterPayload { ray: scatter_ray, attenuation: *albedo })
            }
            HitPayload::Miss => None,
            HitPayload::Inside => None,
        }
    }
}
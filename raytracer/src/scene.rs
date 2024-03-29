use crate::{hittable::Hittable, material::Material};
use glam::Vec3;

pub struct Scene {
    hittables: Vec<Hittable>,
    materials: Vec<Material>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            hittables: Default::default(),
            materials: vec![Material::Null],
        }
    }
}

impl Scene {
    pub fn hittables(&self) -> &[Hittable] {
        self.hittables.as_slice()
    }

    pub fn hittables_mut(&mut self) -> &mut [Hittable] {
        &mut self.hittables
    }

    pub fn hittable(&self, idx: usize) -> &Hittable {
        &self.hittables[idx]
    }

    pub fn add_hittable<H: Into<Hittable>>(&mut self, hittable: H) -> usize {
        self.hittables.push(hittable.into());
        self.hittables.len() - 1
    }

    pub fn materials(&self) -> &[Material] {
        self.materials.as_slice()
    }

    pub fn materials_mut(&mut self) -> &mut [Material] {
        &mut self.materials
    }

    pub fn material(&self, idx: usize) -> &Material {
        &self.materials[idx]
    }

    pub fn add_material(&mut self, material: Material) -> usize {
        self.materials.push(material);
        self.materials.len() - 1
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_index: usize,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 1.0,
            material_index: 0,
        }
    }
}

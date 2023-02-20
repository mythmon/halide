use glam::Vec3;

#[derive(Default)]
pub struct Scene {
    spheres: Vec<Sphere>,
    materials: Vec<Material>,
}

impl Scene {
    pub fn spheres(&self) -> &[Sphere] {
        self.spheres.as_slice()
    }

    pub fn spheres_mut(&mut self) -> &mut [Sphere] {
        &mut self.spheres
    }

    pub fn sphere(&self, idx: usize) -> &Sphere {
        &self.spheres[idx]
    }

    pub fn add_sphere(&mut self, sphere: Sphere) -> usize {
        self.spheres.push(sphere);
        self.spheres.len() - 1
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

pub struct Material {
    pub albedo: Vec3,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ONE,
            roughness: 0.5,
            metallic: 0.0,
        }
    }
}

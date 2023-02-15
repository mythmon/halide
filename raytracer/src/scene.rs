use glam::Vec3;

pub struct Scene {
    spheres: Vec<Sphere>,
    light_direction: Vec3,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            spheres: Vec::new(),
            light_direction: Vec3::NEG_ONE.normalize(),
        }
    }
}

impl Scene {
    pub fn light_direction(&self) -> Vec3 {
        self.light_direction
    }

    pub fn set_light_direction(&mut self, light_direction: Vec3) {
        if let Some(normalized) = light_direction.try_normalize() {
            self.light_direction = normalized;
        }
    }

    pub fn spheres(&self) -> &[Sphere] {
        self.spheres.as_slice()
    }

    pub fn spheres_mut(&mut self) -> &mut [Sphere] {
        &mut self.spheres
    }

    pub fn add_sphere(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub albedo: Vec3,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 1.0,
            albedo: Vec3::ONE,
        }
    }
}

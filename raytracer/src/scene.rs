use glam::Vec3;

pub struct Scene {
    sphere_color: Vec3,
    sphere_radius: f32,
    light_direction: Vec3,
}

impl Default for Scene {
    fn default() -> Self {
        Self { sphere_color: Vec3::new(0.27, 0.51, 0.71), sphere_radius: 0.5, light_direction: Vec3::NEG_ONE.normalize() }
    }
}

impl Scene {
    pub fn light_direction(&self) -> Vec3 {
        self.light_direction
    }

    pub fn set_light_direction(&mut self, light_direction: Vec3) {
        self.light_direction = light_direction.normalize();
    }

    pub fn sphere_color(&self) -> Vec3 {
        self.sphere_color
    }

    pub fn set_sphere_color(&mut self, sphere_color: Vec3) {
        self.sphere_color = sphere_color;
    }

    pub fn sphere_radius(&self) -> f32 {
        self.sphere_radius
    }

    pub fn set_sphere_radius(&mut self, sphere_radius: f32) {
        self.sphere_radius = sphere_radius;
    }
}

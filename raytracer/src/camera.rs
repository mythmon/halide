use glam::Vec3;

pub struct Camera {
    position: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self { position: Vec3::NEG_Z * 2. }
    }
}

impl Camera {
    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
}
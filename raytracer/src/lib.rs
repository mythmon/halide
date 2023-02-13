use std::borrow::Cow;
use glam::{Vec2, Vec3};

pub struct Renderer {
    // in ABGR order
    image_data: Vec<u32>,
    width: u32,
    height: u32,
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
    fn image_len(&self) -> usize {
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

    pub fn render(&mut self) -> Cow<[u32]> {
        self.image_data.resize(self.image_len(), 0);
        let wp = self.width as f32;
        let hp = self.height as f32;
        for y in 0..self.height {
            let yp = y as f32;
            for x in 0..self.width {
                let xp = x as f32;
                // -1,-1 to 1,1
                let coord = Vec2::new(xp / wp, yp / hp) * 2.0 - Vec2::new(1., 1.);
                let i = (x + y * self.width) as usize;
                self.image_data[i] = self.per_pixel(coord);
            }
        }
        Cow::Borrowed(self.image_data.as_slice())
    }

    fn per_pixel(&self, coord: Vec2) -> u32 {
        let ray_direction = coord.extend(-1.).normalize();
        let ray_origin = Vec3::new(0.0, 0.0, 2.0);
        let radius = 0.5_f32;

        // quadratic equation co-effiecients
        let a = ray_direction.length_squared();
        let b = 2. * ray_direction.dot(ray_origin);
        let c = ray_origin.length_squared() - radius.powi(2);

        let discrim = b.powi(2) - 4. * a * c;

        if discrim >= 0. {
            0xFF_FF_00_FF
        } else {
            0xFF_00_00_00
        }
    }
}

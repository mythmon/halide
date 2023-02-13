use glam::{Vec2, Vec3, Vec4};
use std::borrow::Cow;

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

    pub fn render(&mut self, sphere_color: &[f32; 3]) -> Cow<[u32]> {
        self.image_data.resize(self.image_len(), 0);
        let wp = self.width as f32;
        let hp = self.height as f32;
        let aspect = wp / hp;

        for y in 0..self.height {
            let yp = y as f32;
            for x in 0..self.width {
                let xp = x as f32;
                // screen uv coordinate with y in [-1,1] and x in [-aspect,aspect]
                let coord = Vec2::new(((xp / wp * 2.) - 1.) * aspect, (yp / hp * 2.) - 1.);
                let i = (x + y * self.width) as usize;
                let color = self.per_pixel(coord, sphere_color).clamp(Vec4::ZERO, Vec4::ONE);
                self.image_data[i] = color_rgba(&color);
            }
        }
        Cow::Borrowed(self.image_data.as_slice())
    }

    fn per_pixel(&self, coord: Vec2, sphere_color: &[f32; 3]) -> Vec4 {
        let ray_direction = coord.extend(-1.);
        let ray_origin = Vec3::new(0.0, 0.0, 1.0);
        let radius = 0.5_f32;
        let light_direction = Vec3::NEG_ONE.normalize();

        // solve the equation of the ray set equal to the equation of a sphere centered on the origin.
        // a, b, and c are the quadratic equation co-effiecients
        let a = ray_direction.length_squared();
        let b = 2. * ray_direction.dot(ray_origin);
        let c = ray_origin.length_squared() - radius.powi(2);

        let discrim = b.powi(2) - 4. * a * c;

        if discrim < 0. {
            Vec3::ZERO.extend(1.)
        } else {
            // finish the quadratic equation, though we only need the least result
            let t0 = (-b - discrim.sqrt()) / (2. * a);
            let hit_pos = ray_origin + ray_direction * t0;
            let normal = hit_pos.normalize();
            let d = normal.dot(-light_direction).max(0.0);
            (Vec3::new(sphere_color[0], sphere_color[1], sphere_color[2]) * d).extend(1.0)
        }
    }
}

fn color_rgba(c: &Vec4) -> u32 {
    let r = (c.x * 255.) as u32;
    let g = (c.y * 255.) as u32;
    let b = (c.z * 255.) as u32;
    let a = (c.w * 255.) as u32;
    a << 24 | b << 16 | g << 8 | r
}

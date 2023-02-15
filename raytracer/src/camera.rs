use glam::{Mat4, Quat, Vec2, Vec3, Vec4Swizzles};
use std::{
    cell::{Ref, RefCell},
    ops::Deref,
};

pub struct Camera {
    position: Vec3,
    look_direction: Vec3,
    right_direction: Vec3,
    up_direction: Vec3,
    vertical_fov: f32,
    width: u32,
    height: u32,
    near_clip: f32,
    far_clip: f32,
    cached_directions: RefCell<Option<Vec<Vec3>>>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::Z * 3.,
            look_direction: Vec3::NEG_Z,
            right_direction: Vec3::X,
            up_direction: Vec3::Y,
            vertical_fov: 20.,
            width: 640,
            height: 480,
            near_clip: 0.01,
            far_clip: 100.,
            cached_directions: RefCell::new(None),
        }
    }
}

impl Camera {
    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Move the cameras origin. `offset` is mapped to the coordinate system of
    /// the view, with X being to the right, Y being up, and Z being backwards.
    pub fn relative_move(&mut self, offset: Vec3, ts: f32) {
        const MOVE_SPEED: f32 = 2.;
        let rotated = offset.x * self.right_direction
            + offset.y * self.up_direction
            + offset.z * self.look_direction;
        self.position += MOVE_SPEED * rotated * ts;
        self.clear_ray_cache();
    }

    pub fn relative_turn(&mut self, [pitch, yaw]: [f32; 2], ts: f32) {
        const TURN_SPEED: f32 = 0.2;
        let scale = TURN_SPEED * ts;
        let q = Quat::from_axis_angle(self.right_direction, pitch * scale)
            * Quat::from_axis_angle(self.up_direction, yaw * scale).normalize();

        self.look_direction = q * self.look_direction;
        self.right_direction = q * self.right_direction;
        self.up_direction = q * self.up_direction;
        self.clear_ray_cache();
    }

    pub fn look_direction(&self) -> Vec3 {
        self.look_direction
    }

    pub fn set_look_direction(&mut self, look_direction: Vec3) {
        if let Some(normalized) = look_direction.try_normalize() {
            if normalized != self.look_direction {
                self.look_direction = normalized;
                self.clear_ray_cache();
            }
        }
    }

    pub fn vertical_fov(&self) -> f32 {
        self.vertical_fov
    }

    pub fn set_vertical_fov(&mut self, vertical_fov: f32) {
        if self.vertical_fov != vertical_fov {
            self.vertical_fov = vertical_fov;
            self.clear_ray_cache();
        }
    }

    pub fn size(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.clear_ray_cache()
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn get_ray_directions(&self) -> impl Deref<Target = [Vec3]> + '_ {
        if self.cached_directions.borrow().is_none() {
            self.compute_ray_directions();
        }
        Ref::map(self.cached_directions.borrow(), |b| {
            b.as_ref().unwrap().as_slice()
        })
    }

    fn clear_ray_cache(&self) {
        self.cached_directions.replace(None);
    }

    fn compute_ray_directions(&self) {
        const V_UP: Vec3 = Vec3::new(0., 1., 0.);

        let view = Mat4::look_to_rh(self.position, self.look_direction, V_UP);
        let view_inverse = view.inverse();

        let projection = Mat4::perspective_rh(
            self.vertical_fov,
            self.aspect_ratio(),
            self.near_clip,
            self.far_clip,
        );
        let projection_inverse = projection.inverse();

        let mut ray_directions = Vec::with_capacity(self.width as usize * self.height as usize);

        let wp = self.width as f32;
        let hp = self.height as f32;
        for y in 0..self.height {
            let yp = y as f32;
            for x in 0..self.width {
                let xp = x as f32;
                // screen uv coordinate with x and y in [-1,1]
                let coord = Vec2::new(xp / wp, yp / hp) * 2. - Vec2::ONE;

                let target = projection_inverse * coord.extend(1.).extend(1.);
                let direction = view_inverse * (target.xyz() / target.w).normalize().extend(0.);
                ray_directions.push(direction.xyz());
            }
        }

        let _ = self.cached_directions.borrow_mut().insert(ray_directions);
    }
}

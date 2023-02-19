use glam::{Vec3, Vec4};

pub(crate) fn color_rgba(c: &Vec4) -> u32 {
    let c = c.clamp(Vec4::ZERO, Vec4::ONE);
    let r = (c.x * 255.) as u32;
    let g = (c.y * 255.) as u32;
    let b = (c.z * 255.) as u32;
    let a = (c.w * 255.) as u32;
    a << 24 | b << 16 | g << 8 | r
}

pub(crate) fn color_rgb(c: Vec3) -> u32 {
    color_rgba(&c.extend(1.))
}

pub trait Vec3Ext {
    fn reflect(self, normal: Self) -> Self;
}

impl Vec3Ext for Vec3 {
    /// Returns the vector reflected across the given normal.
    fn reflect(self, normal: Self) -> Self {
        assert!(normal.is_normalized());
        let rej = self.reject_from_normalized(normal);
        self - 2.0 * rej
    }
}

#[cfg(test)]
mod tests {
    use crate::util::Vec3Ext;
    use float_eq::assert_float_eq;
    use glam::Vec3;

    #[test]
    fn reflect() {
        let x = Vec3::X;
        let normal = Vec3::new(1., 1., 0.).normalize();
        let y = x.reflect(normal);
        assert_float_eq!(y.to_array(), Vec3::Y.to_array(), abs <= [0.001, 0.001, 0.001]);
    }
}

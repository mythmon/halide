use glam::{Vec3, Vec4};

pub(crate) fn color_rgba(c: &Vec4) -> u32 {
    let c = c.clamp(Vec4::ZERO, Vec4::ONE);
    let r = (c.x * 255.) as u32;
    let g = (c.y * 255.) as u32;
    let b = (c.z * 255.) as u32;
    let a = (c.w * 255.) as u32;
    a << 24 | b << 16 | g << 8 | r
}

pub(crate) fn color_rgb(c: &Vec3) -> u32 {
    color_rgba(&c.extend(1.))
}

pub trait Vec3Ext {
    fn reflect(self, normal: Self) -> Self;
}

impl Vec3Ext for Vec3 {
    /// Returns the vector reflected across the given normal.
    fn reflect(self, normal: Self) -> Self {
        assert!(normal.is_normalized());
        self - 2.0 * self.dot(normal) * normal
    }
}

pub fn xy_index<X: Into<u32>, Y: Into<u32>, W: Into<u32>>(x: X, y: Y, w: W) -> usize {
    let x = x.into() as usize;
    let y = y.into() as usize;
    let w = w.into() as usize;
    x + w * y
}

#[cfg(test)]
mod tests {
    use crate::util::Vec3Ext;
    use glam::Vec3;

    #[test]
    fn reflect() {
        let x = Vec3::X;
        let y = x.reflect(Vec3::new(1., 1., 0.).normalize());
        assert!((y - Vec3::Y).length() < 0.001, "{y}");
    }
}

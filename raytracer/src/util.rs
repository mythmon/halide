use glam::{Vec4, Vec3};

pub(crate) fn color_rgba(c: &Vec4) -> u32 {
    let r = (c.x * 255.) as u32;
    let g = (c.y * 255.) as u32;
    let b = (c.z * 255.) as u32;
    let a = (c.w * 255.) as u32;
    a << 24 | b << 16 | g << 8 | r
}

pub(crate) fn color_rgb(c: &Vec3) -> u32 {
    color_rgba(&c.extend(1.))
}
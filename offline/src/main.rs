use anyhow::Result;
use glam::Vec3;
use halide_raytracer::{Camera, Material, Renderer, Scene, Sphere};
use png_pong::PngRaster;

fn main() -> Result<()> {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    let mut renderer = Renderer::new(WIDTH, HEIGHT);

    let mut scene = Scene::default();

    let ground_material = scene.add_material(Material {
        albedo: Vec3::new(0.7, 0.7, 0.7),
        ..Default::default()
    });
    let ball_material = scene.add_material(Material {
        albedo: Vec3::new(0.9, 0.2, 0.1),
        ..Default::default()
    });

    scene.add_sphere(Sphere {
        center: Vec3::new(0., -10_000., 0.),
        radius: 10_000.,
        material_index: ground_material,
    });

    scene.add_sphere(Sphere {
        center: Vec3::new(0., 0.5, 0.),
        radius: 0.5,
        material_index: ball_material,
    });

    let mut camera = Camera::default();
    camera.set_size(WIDTH, HEIGHT);
    camera.set_position((0., 0.75, 4.).into());

    // image data is packed u32s in ABGR order, and y goes from bottom to top
    let image_data = renderer.render_accumulate(&scene, &camera, 64);
    // buffer is unpacked u8s in RGB(A) order, and y goes from top to bottom
    let mut buffer = Vec::new();
    buffer.resize(image_data.len() * 4, 0);
    // Convert from u32 to u8, and also flip the y axis.
    for (idx1, p) in image_data.into_iter().enumerate() {
        let x = idx1 % (WIDTH as usize);
        let y = (HEIGHT as usize) - (idx1 / (WIDTH as usize)) - 1;
        let idx2 = (x + y * (WIDTH as usize)) * 4;
        let bytes = p.to_le_bytes();
        for offset in 0..4 {
            buffer[idx2 + offset] = bytes[offset];
        }
    }
    // convert to a pix raster, and then from RGBA to RGB.
    let raster = pix::Raster::<pix::rgb::SRgba8>::with_u8_buffer(WIDTH, HEIGHT, buffer);
    let converted = pix::Raster::<pix::rgb::SRgb8>::with_raster(&raster);

    // encode and output the image
    let png_raster = PngRaster::Rgb8(converted);
    let mut out_data = Vec::new();
    let mut encoder = png_pong::Encoder::new(&mut out_data).into_step_enc();
    let step = png_pong::Step {
        raster: png_raster,
        delay: 0,
    };
    encoder.encode(&step)?;
    std::fs::write("image.png", out_data)?;

    Ok(())
}

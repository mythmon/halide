use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::Vec3;
use halide_raytracer::{Camera, Material, Renderer, Scene, Sphere};
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 480;
    let mut renderer = Renderer::new(WIDTH, HEIGHT);
    renderer.set_num_threads(1);

    let mut scene = Scene::default();
    let ground_material = scene.add_material(Material::Lambertian {
        albedo: Vec3::new(0.9, 0.2, 0.1),
    });
    let ball_material = scene.add_material(Material::Lambertian {
        albedo: Vec3::new(0.7, 0.7, 0.7),
    });

    scene.add_hittable(Sphere {
        center: Vec3::new(0., -10_000., 0.),
        radius: 10_000.,
        material_index: ground_material,
    });

    scene.add_hittable(Sphere {
        center: Vec3::new(-1.1, 0.5, 0.),
        radius: 0.5,
        material_index: ball_material,
    });
    scene.add_hittable(Sphere {
        center: Vec3::new(0., 0.5, 0.),
        radius: 0.5,
        material_index: ball_material,
    });
    scene.add_hittable(Sphere {
        center: Vec3::new(1.1, 0.5, 0.),
        radius: 0.5,
        material_index: ball_material,
    });

    let mut camera = Camera::default();
    camera.set_size(WIDTH, HEIGHT);
    camera.set_position((0., 0.75, 4.).into());

    c.bench_function("sphere demo", move |b| {
        b.iter(|| {
            renderer.reset_accumulation();
            black_box(renderer.render(&scene, &camera));
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
);
criterion_main!(benches);

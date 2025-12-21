use divan::{Bencher, black_box};
use lopencraft::core::math::aligned_box3::AlignedBox3;
use lopencraft::core::math::segment3::Segment3;
use lopencraft::core::math::vec3::Vec3;

#[divan::bench]
fn intersect_base_cube(bencher: Bencher) {
  let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 0.0), 0.5);
  let segments = [
    Segment3::start_direction_len(Vec3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0), 5.0),
    Segment3::start_direction_len(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0), 5.0),
    Segment3::start_direction_len(Vec3::new(0.0, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 5.0),
    Segment3::start_direction_len(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 5.0),
    Segment3::start_direction_len(Vec3::new(-3.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 5.0),
    Segment3::start_direction_len(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), 5.0),
  ];

  bencher.bench_local(move || {
    for segment in &segments {
      black_box(black_box(&cube).find_intersecting_face(black_box(&segment)));
    }
  });
}

fn main() {
  divan::main();
}

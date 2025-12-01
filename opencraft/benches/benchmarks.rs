use divan::{Bencher, black_box};
use lopencraft::core::math::aligned_box3::AlignedBox3;
use lopencraft::core::math::intersect::Intersects;
use lopencraft::core::math::segment3::Segment3;
use lopencraft::core::math::vec3::Vec3;

// Bechmarks are split into fast and slow cases. In fast cases, functions exit
// early to show the best possible execution time. In slow cases, all of the
// tests in the function body are performed, which places an upper bound on the
// execution time.

#[divan::bench]
fn find_intersecting_face_base_cube_fast_all_directions(bencher: Bencher) {
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

#[divan::bench]
fn find_intersecting_face_slow(bencher: Bencher) {
  let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 10.0), 0.5);
  let segment =
    Segment3::start_direction_len(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 5.0);

  bencher.bench_local(move || {
    black_box(black_box(&cube).find_intersecting_face(black_box(&segment)));
  });
}

#[divan::bench]
fn intersects_box_segment_slow(bencher: Bencher) {
  let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 10.0), 0.5);
  let segment =
    Segment3::start_direction_len(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 5.0);

  bencher.bench_local(move || {
    black_box(black_box(&cube).intersects(black_box(&segment)));
  });
}

fn main() {
  divan::main();
}

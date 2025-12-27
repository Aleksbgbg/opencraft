use divan::{Bencher, black_box};
use lopencraft::core::math::aligned_box3::AlignedBox3;
use lopencraft::core::math::angle::Angle;
use lopencraft::core::math::frustum3::Frustum3;
use lopencraft::core::math::intersect::Intersects;
use lopencraft::core::math::projection::Perspective;
use lopencraft::core::math::rotor3::Rotor3;
use lopencraft::core::math::segment3::Segment3;
use lopencraft::core::math::vec2::Vec2;
use lopencraft::core::math::vec3::Vec3;
use lopencraft::model::block::Block;
use lopencraft::model::chunk::Chunk;
use lopencraft::model::position::{BlockPosition, ChunkPosition};
use lopencraft::model::terrain::Generate;
use lopencraft::renderer::chunk_mesh::ChunkMesh;
use lopencraft::renderer::texture_atlas::TextureAtlas;
use std::collections::HashMap;
use std::sync::Arc;

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

#[divan::bench]
fn intersects_frustum_box_slow(bencher: Bencher) {
  let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 5.0), 1.0);
  let frustum = Frustum3::new(
    Vec3::new(0.0, 0.0, 0.0),
    Rotor3::identity(),
    &Perspective::new(Vec2::new(2560.0, 1440.0), Angle::degrees(110.0), 1.0, 10.0),
  );

  bencher.bench_local(move || {
    black_box(black_box(&frustum).intersects(black_box(&cube)));
  });
}

#[divan::bench]
fn intersects_frustum_box_fast(bencher: Bencher) {
  let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 500.0), 1.0);
  let frustum = Frustum3::new(
    Vec3::new(0.0, 0.0, 0.0),
    Rotor3::identity(),
    &Perspective::new(Vec2::new(2560.0, 1440.0), Angle::degrees(110.0), 1.0, 10.0),
  );

  bencher.bench_local(move || {
    black_box(black_box(&frustum).intersects(black_box(&cube)));
  });
}

struct AllGrass;

impl Generate for AllGrass {
  fn generate(&self, _block: BlockPosition) -> Block {
    Block::Grass
  }
}

#[divan::bench]
fn chunk_mesh_generate_slow(bencher: Bencher) {
  let chunk = Chunk::load(
    ChunkPosition::default(),
    Arc::new(AllGrass),
    HashMap::default(),
  );
  let texture_atlas = TextureAtlas::grass_only();

  bencher.bench_local(move || {
    let mut mesh = ChunkMesh::generate(black_box(&chunk));
    black_box(mesh.generate_vertices(black_box(&texture_atlas), black_box(&chunk)));
  });
}

#[divan::bench]
fn chunk_mesh_update_incremental_slow(bencher: Bencher) {
  let chunk = Chunk::load(
    ChunkPosition::default(),
    Arc::new(AllGrass),
    HashMap::default(),
  );
  let mut mesh = ChunkMesh::generate(&chunk);
  let texture_atlas = TextureAtlas::grass_only();

  bencher.bench_local(move || {
    black_box(&mut mesh).update_incremental(black_box(&chunk), black_box(BlockPosition::default()));
    black_box(mesh.generate_vertices(black_box(&texture_atlas), black_box(&chunk)));
  });
}

fn main() {
  divan::main();
}

//! Benchmarks for topologic-fast

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use topologic_core::prelude::*;
use topologic_core::topology::TopologyStore;
use topologic_core::geometry::Point3;
use topologic_core::boolean::Boolean;
use topologic_core::spatial::BvhTree;

fn bench_vertex_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_creation");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("vertices", count), count, |b, &count| {
            b.iter(|| {
                let store = TopologyStore::new();
                for i in 0..count {
                    let x = i as f64 * 0.1;
                    Vertex::by_coordinates(&store, x, x, x);
                }
                black_box(store.stats().vertices)
            });
        });
    }

    group.finish();
}

fn bench_face_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("face_creation");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("rectangles", count), count, |b, &count| {
            b.iter(|| {
                let store = TopologyStore::new();
                for i in 0..count {
                    let x = i as f64;
                    Face::rectangle(&store, Point3::new(x, 0.0, 0.0), 1.0, 1.0);
                }
                black_box(store.stats().faces)
            });
        });
    }

    group.finish();
}

fn bench_cell_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell_creation");

    group.bench_function("box", |b| {
        b.iter(|| {
            let store = TopologyStore::new();
            let cell = Cell::box_cell(&store, Point3::ZERO, 1.0, 1.0, 1.0);
            black_box(Cell::volume(&store, cell))
        });
    });

    group.bench_function("sphere_16", |b| {
        b.iter(|| {
            let store = TopologyStore::new();
            let cell = Cell::sphere(&store, Point3::ZERO, 1.0, 16, 8);
            black_box(Cell::volume(&store, cell))
        });
    });

    group.bench_function("cylinder_32", |b| {
        b.iter(|| {
            let store = TopologyStore::new();
            let cell = Cell::cylinder(&store, Point3::ZERO, 1.0, 2.0, 32);
            black_box(Cell::volume(&store, cell))
        });
    });

    group.finish();
}

fn bench_boolean_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("boolean");

    group.bench_function("union", |b| {
        b.iter(|| {
            let store = TopologyStore::new();
            let cell1 = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);
            let cell2 = Cell::box_cell(&store, Point3::new(1.0, 0.0, 0.0), 2.0, 2.0, 2.0);
            let result = Boolean::union(&store, cell1, cell2);
            black_box(result.success)
        });
    });

    group.bench_function("intersection", |b| {
        b.iter(|| {
            let store = TopologyStore::new();
            let cell1 = Cell::box_cell(&store, Point3::ZERO, 2.0, 2.0, 2.0);
            let cell2 = Cell::box_cell(&store, Point3::new(1.0, 0.0, 0.0), 2.0, 2.0, 2.0);
            let result = Boolean::intersection(&store, cell1, cell2);
            black_box(result.success)
        });
    });

    group.finish();
}

fn bench_bvh_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("bvh");

    // Create a cell complex with many faces
    let store = TopologyStore::new();
    let mut faces = Vec::new();

    for i in 0..100 {
        for j in 0..100 {
            let x = i as f64;
            let y = j as f64;
            faces.push(Face::rectangle(&store, Point3::new(x, y, 0.0), 0.9, 0.9));
        }
    }

    let bvh = BvhTree::build(&store, &faces);

    group.bench_function("aabb_query", |b| {
        b.iter(|| {
            use topologic_core::spatial::Aabb;
            let query = Aabb::new(
                Point3::new(45.0, 45.0, -1.0),
                Point3::new(55.0, 55.0, 1.0),
            );
            let results = bvh.query_aabb(&query);
            black_box(results.len())
        });
    });

    group.bench_function("ray_query", |b| {
        b.iter(|| {
            use topologic_core::geometry::Vector3;
            let results = bvh.query_ray(
                Point3::new(50.0, 50.0, 10.0),
                Vector3::new(0.0, 0.0, -1.0),
            );
            black_box(results.len())
        });
    });

    group.finish();
}

fn bench_area_volume(c: &mut Criterion) {
    let mut group = c.benchmark_group("measurements");

    let store = TopologyStore::new();
    let cell = Cell::box_cell(&store, Point3::ZERO, 10.0, 10.0, 10.0);

    group.bench_function("cell_volume", |b| {
        b.iter(|| {
            black_box(Cell::volume(&store, cell))
        });
    });

    group.bench_function("cell_area", |b| {
        b.iter(|| {
            black_box(Cell::area(&store, cell))
        });
    });

    let face = Face::rectangle(&store, Point3::ZERO, 10.0, 10.0);

    group.bench_function("face_area", |b| {
        b.iter(|| {
            black_box(Face::area(&store, face))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vertex_creation,
    bench_face_creation,
    bench_cell_creation,
    bench_boolean_operations,
    bench_bvh_query,
    bench_area_volume,
);

criterion_main!(benches);

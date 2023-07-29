use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion};
use trait_vec::trait_vec::PolyPtrVec;

fn bench_normal_vec(c: &mut BenchmarkGroup<WallTime>) {
    let mut normal_vec: Vec<Box<[usize]>> = Vec::with_capacity(1024);
    normal_vec.push(Box::new([1; 1]));
    normal_vec.push(Box::new([2; 2]));
    normal_vec.push(Box::new([3; 3]));
    normal_vec.push(Box::new([4; 4]));
    normal_vec.push(Box::new([5; 5]));
    normal_vec.push(Box::new([6; 6]));
    normal_vec.push(Box::new([7; 7]));
    normal_vec.push(Box::new([8; 8]));
    normal_vec.push(Box::new([9; 9]));
    normal_vec.push(Box::new([10; 10]));

    c.bench_function("Vec<Box<dyn>>: optimistic", |b| {
        b.iter(|| {
            let sum = normal_vec.iter().flat_map(|slice| slice.iter()).sum::<usize>();
            black_box(sum);
        })
    });

    let mut waste: Vec<Box<[usize]>> = Vec::with_capacity(1024);
    let mut vec_with_gaps: Vec<Box<[usize]>> = Vec::with_capacity(1024);
    vec_with_gaps.push(Box::new([1; 1]));
    waste.push(Box::new([1; 1]));
    waste.push(Box::new([2; 2]));
    vec_with_gaps.push(Box::new([2; 2]));
    waste.push(Box::new([3; 3]));
    vec_with_gaps.push(Box::new([3; 3]));
    vec_with_gaps.push(Box::new([4; 4]));
    waste.push(Box::new([4; 4]));
    waste.push(Box::new([5; 5]));
    vec_with_gaps.push(Box::new([5; 5]));
    waste.push(Box::new([6; 6]));
    vec_with_gaps.push(Box::new([6; 6]));
    waste.push(Box::new([7; 7]));
    vec_with_gaps.push(Box::new([7; 7]));
    waste.push(Box::new([8; 8]));
    vec_with_gaps.push(Box::new([8; 8]));
    vec_with_gaps.push(Box::new([9; 9]));
    waste.push(Box::new([9; 9]));
    waste.push(Box::new([10; 10]));
    vec_with_gaps.push(Box::new([10; 10]));

    c.bench_function("Vec<Box<dyn>>: gaps", |b| {
        b.iter(|| {
            let sum = vec_with_gaps.iter().flat_map(|slice| slice.iter()).sum::<usize>();
            black_box(sum);
        })
    });

    black_box(waste);
    black_box(normal_vec);
    black_box(vec_with_gaps);
}

fn bench_poly_vec(c: &mut BenchmarkGroup<WallTime>) {
    let mut poly_vec_ref = PolyPtrVec::<[usize]>::with_capacity(1024);
    poly_vec_ref.push([1; 1]);
    poly_vec_ref.push([2; 2]);
    poly_vec_ref.push([3; 3]);
    poly_vec_ref.push([4; 4]);
    poly_vec_ref.push([5; 5]);
    poly_vec_ref.push([6; 6]);
    poly_vec_ref.push([7; 7]);
    poly_vec_ref.push([8; 8]);
    poly_vec_ref.push([9; 9]);
    poly_vec_ref.push([10; 10]);
    let result = || poly_vec_ref.iter().flat_map(|slice| slice).sum::<usize>();

    c.bench_function("Vec<dyn>: ptr", |b| {
        b.iter(|| {
            black_box(result());
        })
    });

    black_box(poly_vec_ref);
}

pub fn bench_static_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_arrays");
    bench_normal_vec(&mut group);
    bench_poly_vec(&mut group);
    group.finish();
}


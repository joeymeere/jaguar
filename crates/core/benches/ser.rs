use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use jaguar::JaguarSerializer;

fn bench_serialize_u32_slice(c: &mut Criterion) {
    let test_data: Vec<u32> = (0..10000).collect();
    
    c.bench_function("serialize_u32_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_u32_slice(black_box(&test_data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_u8_slice(c: &mut Criterion) {
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    c.bench_function("serialize_u8_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_u8_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_u16_slice(c: &mut Criterion) {
    let data: Vec<u16> = (0..10000).map(|i| (i % 65536) as u16).collect();
    c.bench_function("serialize_u16_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_u16_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_u64_slice(c: &mut Criterion) {
    let data: Vec<u64> = (0..10000).map(|i| i as u64).collect();
    c.bench_function("serialize_u64_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_u64_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_i8_slice(c: &mut Criterion) {
    let data: Vec<i8> = (0..10000).map(|i| (i % 128) as i8).collect();
    c.bench_function("serialize_i8_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_i8_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_i16_slice(c: &mut Criterion) {
    let data: Vec<i16> = (0..10000).map(|i| (i % 32768) as i16).collect();
    c.bench_function("serialize_i16_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_i16_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_i64_slice(c: &mut Criterion) {
    let data: Vec<i64> = (0..10000).map(|i| i as i64).collect();
    c.bench_function("serialize_i64_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_i64_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_f32_slice(c: &mut Criterion) {
    let data: Vec<f32> = (0..10000).map(|i| i as f32).collect();
    c.bench_function("serialize_f32_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_f32_slice(black_box(&data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_bool_slice(c: &mut Criterion) {
    let test_data: Vec<bool> = (0..10000).map(|i| i % 3 == 0).collect();
    
    c.bench_function("serialize_bool_slice", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_bool_slice(black_box(&test_data)).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_serialize_varint(c: &mut Criterion) {
    let test_data: Vec<u64> = (0..10000).map(|i| if i % 2 == 0 { i } else { i * 1000 }).collect();
    
    c.bench_function("serialize_varint", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            for v in black_box(&test_data) {
                ser.write_varint(*v).unwrap();
            }
            black_box(ser.finish())
        })
    });
}

/*
fn bench_serialize_string_vec(c: &mut Criterion) {
    let test_data: Vec<String> = (0..1000)
        .map(|i| format!("test_string_{}", i))
        .collect();
    
    c.bench_function("serialize_string_vec", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_string_vec(black_box(&test_data)).unwrap();
            black_box(ser.finish())
        })
    });
}
    */

criterion_group!(
    benches,
    bench_serialize_u8_slice,
    bench_serialize_u16_slice,
    bench_serialize_u32_slice,
    bench_serialize_u64_slice,
    bench_serialize_i8_slice,
    bench_serialize_i16_slice,
    bench_serialize_i64_slice,
    bench_serialize_f32_slice,
    bench_serialize_bool_slice,
    bench_serialize_varint,
    //bench_serialize_string_vec
);
criterion_main!(benches);
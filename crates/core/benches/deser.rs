use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use jaguar::{JaguarSerializer, JaguarDeserializer};

fn bench_deserialize_u32_slice(c: &mut Criterion) {
    let test_data: Vec<u32> = (0..10000).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_u32_slice(&test_data).unwrap();
    let serialized = ser.finish();
    
    c.bench_function("deserialize_u32_slice", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_u32_vec().unwrap())
        })
    });
}

fn bench_deserialize_u8_vec(c: &mut Criterion) {
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_u8_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_u8_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_u8_vec().unwrap())
        })
    });
}

fn bench_deserialize_u16_vec(c: &mut Criterion) {
    let data: Vec<u16> = (0..10000).map(|i| (i % 65536) as u16).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_u16_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_u16_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_u16_vec().unwrap())
        })
    });
}

fn bench_deserialize_u64_vec(c: &mut Criterion) {
    let data: Vec<u64> = (0..10000).map(|i| i as u64).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_u64_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_u64_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_u64_vec().unwrap())
        })
    });
}

fn bench_deserialize_i8_vec(c: &mut Criterion) {
    let data: Vec<i8> = (0..10000).map(|i| (i % 128) as i8).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_i8_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_i8_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_i8_vec().unwrap())
        })
    });
}

fn bench_deserialize_i16_vec(c: &mut Criterion) {
    let data: Vec<i16> = (0..10000).map(|i| (i % 32768) as i16).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_i16_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_i16_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_i16_vec().unwrap())
        })
    });
}

fn bench_deserialize_i64_vec(c: &mut Criterion) {
    let data: Vec<i64> = (0..10000).map(|i| i as i64).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_i64_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_i64_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_i64_vec().unwrap())
        })
    });
}

fn bench_deserialize_f32_vec(c: &mut Criterion) {
    let data: Vec<f32> = (0..10000).map(|i| i as f32).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_f32_slice(&data).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_f32_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_f32_vec().unwrap())
        })
    });
}

fn bench_deserialize_bool_slice(c: &mut Criterion) {
    let test_data: Vec<bool> = (0..10000).map(|i| i % 3 == 0).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_bool_slice(&test_data).unwrap();
    let serialized = ser.finish();
    
    c.bench_function("deserialize_bool_slice", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_bool_vec().unwrap())
        })
    });
}

fn bench_deserialize_varint(c: &mut Criterion) {
    let test_data: Vec<u64> = (0..10000).map(|i| if i % 2 == 0 { i } else { i * 1000 }).collect();
    let mut ser = JaguarSerializer::new();
    for v in &test_data {
        ser.write_varint(*v).unwrap();
    }
    let serialized = ser.finish();
    
    c.bench_function("deserialize_varint", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            for _ in 0..test_data.len() {
                black_box(de.read_varint().unwrap());
            }
        })
    });
}

/*
fn bench_deserialize_string_vec(c: &mut Criterion) {
    let test_data: Vec<String> = (0..1000)
        .map(|i| format!("test_string_{}", i))
        .collect();
    let mut ser = JaguarSerializer::new();
    ser.write_string_vec(&test_data).unwrap();
    let serialized = ser.finish();
    
    c.bench_function("deserialize_string_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(de.read_string_vec().unwrap())
        })
    });
}
    */

criterion_group!(
    benches,
    bench_deserialize_u8_vec,
    bench_deserialize_u16_vec,
    bench_deserialize_u32_slice,
    bench_deserialize_u64_vec,
    bench_deserialize_i8_vec,
    bench_deserialize_i16_vec,
    bench_deserialize_i64_vec,
    bench_deserialize_f32_vec,
    bench_deserialize_bool_slice,
    bench_deserialize_varint,
    //bench_deserialize_string_vec
);
criterion_main!(benches);
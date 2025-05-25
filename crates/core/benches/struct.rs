use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jaguar::{JaguarSerialize, JaguarDeserialize, JaguarSerializer, JaguarDeserializer};

#[derive(JaguarSerialize, JaguarDeserialize, Debug, Clone, PartialEq)]
struct SimpleStruct {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: i8,
    f: i16,
    g: i32,
    h: i64,
    i: f32,
    j: f64,
    k: bool,
}

#[derive(JaguarSerialize, JaguarDeserialize, Debug, Clone, PartialEq)]
struct ComplexStruct {
    id: u64,
    name: String,
    values: Vec<u32>,
    flags: Vec<bool>,
    nested: Vec<SimpleStruct>,
    metadata: Vec<(u8, String)>,
}

fn create_simple_struct() -> SimpleStruct {
    SimpleStruct {
        a: 42,
        b: 12345,
        c: 987654321,
        d: 1234567890123456789,
        e: -42,
        f: -12345,
        g: -987654321,
        h: -1234567890123456789,
        i: 3.14159,
        j: 2.71828,
        k: true,
    }
}

fn create_complex_struct() -> ComplexStruct {
    ComplexStruct {
        id: 123456789,
        name: "Test Struct".to_string(),
        values: (0..100).collect(),
        flags: (0..100).map(|i| i % 2 == 0).collect(),
        nested: (0..10).map(|_| create_simple_struct()).collect(),
        metadata: (0..5).map(|i| (i as u8, format!("item_{}", i))).collect(),
    }
}

fn bench_serialize_simple_struct(c: &mut Criterion) {
    let data = create_simple_struct();
    c.bench_function("serialize_simple_struct", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            black_box(data.clone()).serialize(&mut ser).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_deserialize_simple_struct(c: &mut Criterion) {
    let data = create_simple_struct();
    let mut ser = JaguarSerializer::new();
    data.serialize(&mut ser).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_simple_struct", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(SimpleStruct::deserialize(&mut de).unwrap())
        })
    });
}

fn bench_serialize_complex_struct(c: &mut Criterion) {
    let data = create_complex_struct();
    c.bench_function("serialize_complex_struct", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            black_box(data.clone()).serialize(&mut ser).unwrap();
            black_box(ser.finish())
        })
    });
}

fn bench_deserialize_complex_struct(c: &mut Criterion) {
    let data = create_complex_struct();
    let mut ser = JaguarSerializer::new();
    data.serialize(&mut ser).unwrap();
    let serialized = ser.finish();

    c.bench_function("deserialize_complex_struct", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            black_box(ComplexStruct::deserialize(&mut de).unwrap())
        })
    });
}

fn bench_serialize_simple_struct_vec(c: &mut Criterion) {
    let data: Vec<SimpleStruct> = (0..1000).map(|_| create_simple_struct()).collect();
    c.bench_function("serialize_simple_struct_vec", |b| {
        b.iter(|| {
            let mut ser = JaguarSerializer::new();
            ser.write_varint(data.len() as u64).unwrap();
            for item in &data {
                item.serialize(&mut ser).unwrap();
            }
            black_box(ser.finish())
        })
    });
}

fn bench_deserialize_simple_struct_vec(c: &mut Criterion) {
    let data: Vec<SimpleStruct> = (0..1000).map(|_| create_simple_struct()).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_varint(data.len() as u64).unwrap();
    for item in &data {
        item.serialize(&mut ser).unwrap();
    }
    let serialized = ser.finish();

    c.bench_function("deserialize_simple_struct_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            let len = de.read_varint().unwrap() as usize;
            let mut result = Vec::with_capacity(len);
            for _ in 0..len {
                result.push(SimpleStruct::deserialize(&mut de).unwrap());
            }
            black_box(result)
        })
    });
}

fn bench_deserialize_complex_struct_vec(c: &mut Criterion) {
    let data: Vec<ComplexStruct> = (0..1000).map(|_| create_complex_struct()).collect();
    let mut ser = JaguarSerializer::new();
    ser.write_varint(data.len() as u64).unwrap();
    for item in &data {
        item.serialize(&mut ser).unwrap();
    }
    let serialized = ser.finish();

    c.bench_function("deserialize_complex_struct_vec", |b| {
        b.iter(|| {
            let mut de = JaguarDeserializer::new(black_box(&serialized));
            let len = de.read_varint().unwrap() as usize;
            let mut result = Vec::with_capacity(len);
            for _ in 0..len {
                result.push(ComplexStruct::deserialize(&mut de).unwrap());
            }
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    bench_serialize_simple_struct,
    bench_deserialize_simple_struct,
    bench_serialize_complex_struct,
    bench_deserialize_complex_struct,
    bench_serialize_simple_struct_vec,
    bench_deserialize_simple_struct_vec,
    bench_deserialize_complex_struct_vec,
);
criterion_main!(benches);

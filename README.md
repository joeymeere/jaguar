# Jaguar

A lightweight binary serialization library designed for high-performance operations in resource-constrained environments like sBPF programs and embedded systems.

## Installation

Run:

```bash
cargo add jaguar
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
jaguar = "0.1.0"
```

## Usage

With primitive types:

```rust
use jaguar::{JaguarSerializer, JaguarDeserializer};

let pubkey = [1u8; 32];

let mut ser = JaguarSerializer::new();
pubkey.serialize(&mut ser).unwrap();
let data = ser.finish();

let mut de = JaguarDeserializer::new(&data);
let decoded = <[u8; 32]>::deserialize(&mut de).unwrap();
```

With custom data structures:

```rust
use jaguar::{JaguarSerialize, JaguarDeserialize, JaguarSerializer, JaguarDeserializer};

#[derive(JaguarSerialize, JaguarDeserialize)]
struct MyData {
    authority: [u8; 32],
    amount: u64,
    bump: u8,
}

let mut ser = JaguarSerializer::new();
let instance = MyData { /* struct field values */ }.serialize(&mut ser).unwrap();

let mut de = JaguarDeserializer::new(&instance);
let value = MyData::deserialize(&de).unwrap();
```

## Supported Types

- Primitive integers (u8/i8, u16/i16, u64/i64, etc...)
- Booleans
- Floats (f32, f64)
- Strings and byte slices
- Arrays and vectors of supported types
- Custom structs
- Enums

## Performance

Benchmarks on an M1 Mac Pro:

### Serialization

- `SimpleStruct`: ~60.04ns
- `ComplexStruct`: ~1.121µs
- `Vec<SimpleStruct>`: ~46.12µs
- `Vec<u8>`: ~348.49ns,
- `Vec<i8>`: ~19.31µs
- `Vec<u16>`: ~22.01µs
- `Vec<i16>`: ~20.50µs
- `Vec<u64>`: ~22.14µs
- `Vec<i64>`: ~20.55µs
- `Vec<bool>`: ~4.07µs

### Deserialization

- `SimpleStruct`: ~46.7ns
- `ComplexStruct`: ~1.153µs
- `Vec<SimpleStruct>`: ~39.4µs
- `Vec<u8>`: ~171ns,
- `Vec<i8>`: ~11.36µs
- `Vec<u16>`: ~9.71µs
- `Vec<i16>`: ~12.09µs
- `Vec<u64>`: ~9.82µs
- `Vec<i64>`: ~11.92µs
- `Vec<bool>`: ~4.07µs

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) for details on the process for submitting pull requests.

## License

This project is MIT licensed - see [LICENSE](LICENSE) for details.

# Jaguar Serialization Format Specification

## Overview

Jaguar is a binary serialization format designed for resource-constrained environments. Priorities are size efficiency, fast operations, and compatibility with Solana programs and embedded systems.

## Format Specification

### 1. Varint Encoding

Varints are used for encoding integers and lengths. The first 7 bits contain the value, and the trailing bit indicates whether to continue.

```
Byte format:
[0xxxxxxx] - Single byte (0-127)
[1xxxxxxx][0xxxxxxx] - Two bytes (128-16383)
[1xxxxxxx][1xxxxxxx][0xxxxxxx] - Three bytes (16384-2097151)
...and so on
```

### 2. Signed Integer Encoding

Signed integers leverage zigzag encoding for representing negative numbers:

```
zigzag(n) = (n << 1) ^ (n >> 63)  // for i64
```

This maps:

- 0 → 0
- -1 → 1
- 1 → 2
- -2 → 3
- 2 → 4
  ...and so on

### 3. Boolean Encoding

Single booleans are encoded as a single byte:

- `false` → `0x00`
- `true` → `0x01`

Boolean arrays are bit-packed, with 8 booleans per byte:

```
[length: varint][packed_bools: bytes]
```

### 4. Float Encoding

Floats use a special encoding for common values:

```
Format:
[0x00] - 0.0
[0x01] - 1.0
[0x02] - -1.0
[0xFF][IEEE-754 bytes] - Other values
```

### 5. String and Byte Slice Encoding

```
Format:
[length: varint][data: bytes]
```

### 6. Array/Vector Encoding

```
Format:
[length: varint][elements...]
```

## Implementation Details

### Errors

Jaguar has the following error conditions:

- `BufferTooSmall`: Insufficient space for operation
- `InvalidData`: Corrupted or invalid data
- `InvalidLength`: Invalid length field
- `UnsupportedType`: Type not supported by format

### Performance Optimizations

1. **Varint Encoding**

   - Branch prediction hints
   - Bounds check elimination

2. **Boolean Packing**

   - Direct bit manipulation
   - SIMD operations when possible

3. **Float Optimization**
   - Special handling for common values
   - Direct IEEE-754 encoding for others

## Platform Compatibility

The format is designed to work well within resource constrained environments, leveraging optimizations like:

- `no-std` by default
- No dynamic allocation in core path
- Opt-in size optimization

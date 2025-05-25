#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use core::mem;
use core::ptr;

#[cfg(feature = "derive")]
pub use jaguar_derive::*;

#[cfg(feature = "std")]
extern crate std;

#[derive(Debug, Clone, PartialEq)]
pub enum SerError {
    BufferTooSmall,
    InvalidData,
    InvalidLength,
    UnsupportedType,
}

/// Compact binary serializer, optimized for resource-constrained environments like
/// Solana programs and embedded systems.
/// 
/// --------
/// 
/// ### Usage
/// 
/// ```rust
/// use jaguar::{JaguarSerializer, JaguarDeserializer};
///
/// let pubkey = [1u8; 32];
///
/// let mut ser = JaguarSerializer::new();
/// pubkey.serialize(&mut ser).unwrap();
/// let data = ser.finish();
/// ```
/// 
/// --------
/// 
/// ### Derive Usage
/// 
/// ```rust
/// #[derive(JaguarSerialize, JaguarDeserialize)]
/// struct MyStruct {
///     pubkey: [u8; 32],
/// }
/// 
/// let my_struct = MyStruct { pubkey };
/// let mut ser = JaguarSerializer::new();
/// my_struct.serialize(&mut ser).unwrap().finish();
/// ```
pub struct JaguarSerializer {
    buffer: Vec<u8>,
    pos: usize,
}

/// Deserializer for raw bytes initially serialized by JaguarSerializer.
/// 
/// --------
/// 
/// ### Usage
/// 
/// ```rust
/// use jaguar::{JaguarSerializer, JaguarDeserializer};
/// 
/// let data = vec![1, 2, 3];
/// let mut ser = JaguarSerializer::new();
/// ser.write_u8_slice(&data);
/// let deser = JaguarDeserializer::new(&ser.finish());
/// let data = deser.read_u8_slice().unwrap();
/// ```
/// 
/// --------
/// 
/// ### Derive Usage
/// 
/// ```rust
/// #[derive(JaguarSerialize, JaguarDeserialize)]
/// struct MyStruct {
///     data: [u8; 3],
/// }
/// 
/// let my_struct = MyStruct { data };
/// let mut ser = JaguarSerializer::new();
/// my_struct.serialize(&mut ser).unwrap().finish();
/// let deser = JaguarDeserializer::new(&ser.finish());
/// let data = deser.read_u8_slice().unwrap();
/// ```
pub struct JaguarDeserializer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl JaguarSerializer {
    /// Creates a new serializer with a default capacity of 1024 bytes.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Creates a new serializer with the specified initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            pos: 0,
        }
    }

    /// Finalizes and returns the serialized data.
    /// 
    /// This truncates the internal buffer to the actual size of the
    /// serialized data and returns ownership of the buffer.
    #[inline]
    pub fn finish(mut self) -> Vec<u8> {
        self.buffer.truncate(self.pos);
        self.buffer
    }

    /// Returns a slice containing the currently serialized data.
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.buffer[..self.pos]
    }

    /// Resets the serializer to its initial state, allowing reuse.
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    #[inline]
    fn ensure_space(&mut self, needed: usize) {
        let required = self.pos + needed;
        if self.buffer.len() < required {
            self.buffer.resize(required.max(self.buffer.len() * 2), 0);
        }
    }

    #[inline]
    unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
        let dest = self.buffer.as_mut_ptr().add(self.pos);
        ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
        self.pos += bytes.len();
    }

    /// Writes a single byte to the serialized output.
    #[inline]
    pub fn write_u8(&mut self, value: u8) -> Result<(), SerError> {
        self.ensure_space(1);
        unsafe {
            *self.buffer.as_mut_ptr().add(self.pos) = value;
            self.pos += 1;
        }
        Ok(())
    }

    /// Writes a slice of booleans as a bit-packed sequence.
    pub fn write_bool_slice(&mut self, slice: &[bool]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        let bytes_needed = (slice.len() + 7) / 8;
        self.ensure_space(bytes_needed);
        
        let mut byte = 0u8;
        let _bit = 0;
        let mut pos = 0;
        
        while pos + 8 <= slice.len() {
            let chunk = &slice[pos..pos + 8];
            byte = 0;
            for (i, &b) in chunk.iter().enumerate() {
                if b {
                    byte |= 1 << i;
                }
            }
            unsafe {
                *self.buffer.as_mut_ptr().add(self.pos) = byte;
            }
            self.pos += 1;
            pos += 8;
        }
        
        if pos < slice.len() {
            byte = 0;
            for (i, &b) in slice[pos..].iter().enumerate() {
                if b {
                    byte |= 1 << i;
                }
            }
            unsafe {
                *self.buffer.as_mut_ptr().add(self.pos) = byte;
            }
            self.pos += 1;
        }
        
        Ok(())
    }

    /// Varint encoding for unsigned integers.
    #[inline]
    pub fn write_varint(&mut self, mut value: u64) -> Result<(), SerError> {
        self.ensure_space(10);
        unsafe {
            let mut ptr = self.buffer.as_mut_ptr().add(self.pos);
            // common case: value < 128
            if value < 0x80 {
                *ptr = value as u8;
                self.pos += 1;
                return Ok(());
            }
            while value >= 0x80 {
                *ptr = (value as u8) | 0x80;
                ptr = ptr.add(1);
                value >>= 7;
                self.pos += 1;
            }
            *ptr = value as u8;
            self.pos += 1;
        }
        Ok(())
    }

    /// Writes a signed integer using variable-length encoding.
    /// 
    /// This uses zigzag encoding to represent signed integers,
    /// where the sign bit is interleaved with magnitude bits.
    #[inline]
    pub fn write_signed_varint(&mut self, value: i64) -> Result<(), SerError> {
        let encoded = ((value << 1) ^ (value >> 63)) as u64;
        self.write_varint(encoded)
    }

    /// Writes a boolean value as a single byte.
    #[inline]
    pub fn write_bool(&mut self, value: bool) -> Result<(), SerError> {
        self.write_u8(if value { 1 } else { 0 })
    }

    /// Writes a 32-bit float with special handling for common values.
    /// 
    /// This optimizes for common float values (0.0, 1.0, -1.0),
    /// using a single byte marker. All other values are stored in full IEEE-754
    /// format with a marker byte.
    #[inline]
    pub fn write_f32(&mut self, value: f32) -> Result<(), SerError> {
        if value == 0.0 {
            return self.write_u8(0);
        } else if value == 1.0 {
            return self.write_u8(1);
        } else if value == -1.0 {
            return self.write_u8(2);
        }
        
        self.write_u8(255)?; 
        self.ensure_space(4);
        unsafe {
            let bytes = mem::transmute::<f32, [u8; 4]>(value);
            self.write_bytes_unchecked(&bytes);
        }
        Ok(())
    }

    /// Writes a 64-bit float.
    /// 
    /// Similar to write_f32, but for double-precision floats.
    #[inline]
    pub fn write_f64(&mut self, value: f64) -> Result<(), SerError> {
        if value == 0.0 {
            return self.write_u8(0);
        } else if value == 1.0 {
            return self.write_u8(1);
        } else if value == -1.0 {
            return self.write_u8(2);
        }
        
        self.write_u8(255)?;
        self.ensure_space(8);
        unsafe {
            let bytes = mem::transmute::<f64, [u8; 8]>(value);
            self.write_bytes_unchecked(&bytes);
        }
        Ok(())
    }

    /// Writes a string as a length-prefixed UTF-8 byte sequence.
    #[inline]
    pub fn write_str(&mut self, s: &str) -> Result<(), SerError> {
        let bytes = s.as_bytes();
        self.write_varint(bytes.len() as u64)?;
        self.ensure_space(bytes.len());
        unsafe {
            self.write_bytes_unchecked(bytes);
        }
        Ok(())
    }

    /// Writes a byte slice as a length-prefixed sequence.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SerError> {
        self.write_varint(bytes.len() as u64)?;
        self.ensure_space(bytes.len());
        unsafe {
            self.write_bytes_unchecked(bytes);
        }
        Ok(())
    }

    /// Writes a slice of 32-bit integers.
    #[inline]
    pub fn write_u32_slice(&mut self, slice: &[u32]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        let bytes_needed = slice.len() * 4;
        self.ensure_space(bytes_needed);
        unsafe {
            let dest = self.buffer.as_mut_ptr().add(self.pos);
            ptr::copy_nonoverlapping(slice.as_ptr() as *const u8, dest, bytes_needed);
            self.pos += bytes_needed;
        }
        Ok(())
    }

    /// Writes a vector of strings, each encoded as a length-prefixed UTF-8 sequence.
    #[inline]
    pub fn write_string_vec(&mut self, vec: &[String]) -> Result<(), SerError> {
        self.write_varint(vec.len() as u64)?;
        for s in vec {
            self.write_str(s)?;
        }
        Ok(())
    }

    /// Writes a slice of 8-bit integers.
    #[inline]
    pub fn write_u8_slice(&mut self, slice: &[u8]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        self.ensure_space(slice.len());
        unsafe {
            self.write_bytes_unchecked(slice);
        }
        Ok(())
    }

    /// Writes a slice of 16-bit integers using varlen encoding.
    #[inline]
    pub fn write_u16_slice(&mut self, slice: &[u16]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_varint(value as u64)?;
        }
        Ok(())
    }

    /// Writes a slice of 64-bit integers using varlen encoding.
    #[inline]
    pub fn write_u64_slice(&mut self, slice: &[u64]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_varint(value)?;
        }
        Ok(())
    }

    /// Writes a slice of signed 8-bit integers.
    #[inline]
    pub fn write_i8_slice(&mut self, slice: &[i8]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_signed_varint(value as i64)?;
        }
        Ok(())
    }

    /// Writes a slice of signed 16-bit integers.
    #[inline]
    pub fn write_i16_slice(&mut self, slice: &[i16]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_signed_varint(value as i64)?;
        }
        Ok(())
    }

    /// Writes a slice of signed 64-bit integers.
    #[inline]
    pub fn write_i64_slice(&mut self, slice: &[i64]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_signed_varint(value)?;
        }
        Ok(())
    }

    /// Writes a slice of 32-bit floats.
    #[inline]
    pub fn write_f32_slice(&mut self, slice: &[f32]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_f32(value)?;
        }
        Ok(())
    }

    /// Writes a slice of 64-bit floats.
    #[inline]
    pub fn write_f64_slice(&mut self, slice: &[f64]) -> Result<(), SerError> {
        self.write_varint(slice.len() as u64)?;
        for &value in slice {
            self.write_f64(value)?;
        }
        Ok(())
    }
}

impl<'a> JaguarDeserializer<'a> {
    /// Creates a new deserializer from a byte slice.
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Returns `true` if there is more data to read.
    #[inline]
    pub fn has_data(&self) -> bool {
        self.pos < self.data.len()
    }

    /// Returns the current read position in the data.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Reads a single byte from the input.
    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, SerError> {
        if self.pos >= self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    /// Reads a vector of 32-bit integers.
    #[inline]
    pub fn read_u32_vec(&mut self) -> Result<Vec<u32>, SerError> {
        let len = self.read_varint()? as usize;
        let bytes_needed = len * 4;
        if self.pos + bytes_needed > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        
        let mut vec = Vec::with_capacity(len);
        unsafe {
            vec.set_len(len);
            ptr::copy_nonoverlapping(
                self.data.as_ptr().add(self.pos),
                vec.as_mut_ptr() as *mut u8,
                bytes_needed
            );
        }
        self.pos += bytes_needed;
        Ok(vec)
    }

    /// Reads a vector of booleans.
    #[inline]
    pub fn read_bool_vec(&mut self) -> Result<Vec<bool>, SerError> {
        let len = self.read_varint()? as usize;
        let bytes_needed = (len + 7) / 8;
        if self.pos + bytes_needed > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        
        let mut vec = Vec::with_capacity(len);
        let mut pos = 0;
    
        while pos + 8 <= len {
            let byte = self.data[self.pos];
            self.pos += 1;
            for i in 0..8 {
                vec.push((byte & (1 << i)) != 0);
            }
            pos += 8;
        }
        
        if pos < len {
            let byte = self.data[self.pos];
            self.pos += 1;
            for i in 0..(len - pos) {
                vec.push((byte & (1 << i)) != 0);
            }
        }
        
        Ok(vec)
    }

    /// Deserialization for fixed-size arrays of primitive types.
    #[inline]
    pub fn read_fixed_array<T: Copy, const N: usize>(&mut self) -> Result<[T; N], SerError> {
        let bytes_needed = N * mem::size_of::<T>();
        if self.pos + bytes_needed > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        
        let mut result = [unsafe { mem::zeroed() }; N];
        unsafe {
            ptr::copy_nonoverlapping(
                self.data.as_ptr().add(self.pos),
                result.as_mut_ptr() as *mut u8,
                bytes_needed
            );
        }
        self.pos += bytes_needed;
        Ok(result)
    }

    /// Varint decoding for unsigned integers.
    #[inline]
    pub fn read_varint(&mut self) -> Result<u64, SerError> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut count = 0;
        loop {
            if self.pos >= self.data.len() {
                return Err(SerError::BufferTooSmall);
            }
            let byte = self.data[self.pos];
            self.pos += 1;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            count += 1;
            if shift >= 64 || count > 9 {
                return Err(SerError::InvalidData);
            }
        }
        Ok(result)
    }

    /// Reads a varlen encoded signed integer.
    #[inline]
    pub fn read_signed_varint(&mut self) -> Result<i64, SerError> {
        let encoded = self.read_varint()?;
        Ok(((encoded >> 1) as i64) ^ (-((encoded & 1) as i64)))
    }

    /// Reads a boolean value.
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool, SerError> {
        Ok(self.read_u8()? != 0)
    }

    /// Reads a 32-bit float.
    #[inline]
    pub fn read_f32(&mut self) -> Result<f32, SerError> {
        let marker = self.read_u8()?;
        match marker {
            0 => Ok(0.0),
            1 => Ok(1.0),
            2 => Ok(-1.0),
            255 => {
                if self.pos + 4 > self.data.len() {
                    return Err(SerError::BufferTooSmall);
                }
                unsafe {
                    let bytes = ptr::read_unaligned(self.data.as_ptr().add(self.pos) as *const [u8; 4]);
                    self.pos += 4;
                    Ok(mem::transmute::<[u8; 4], f32>(bytes))
                }
            }
            _ => Err(SerError::InvalidData),
        }
    }

    /// Reads a 64-bit float.
    #[inline]
    pub fn read_f64(&mut self) -> Result<f64, SerError> {
        let marker = self.read_u8()?;
        match marker {
            0 => Ok(0.0),
            1 => Ok(1.0),
            2 => Ok(-1.0),
            255 => {
                if self.pos + 8 > self.data.len() {
                    return Err(SerError::BufferTooSmall);
                }
                unsafe {
                    let bytes = ptr::read_unaligned(self.data.as_ptr().add(self.pos) as *const [u8; 8]);
                    self.pos += 8;
                    Ok(mem::transmute::<[u8; 8], f64>(bytes))
                }
            }
            _ => Err(SerError::InvalidData),
        }
    }

    /// Reads a length-prefixed UTF-8 string.
    #[inline]
    pub fn read_str(&mut self) -> Result<&'a str, SerError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        
        #[cfg(feature = "std")]
        {
            std::str::from_utf8(slice).map_err(|_| SerError::InvalidData)
        }
        #[cfg(not(feature = "std"))]
        {
            core::str::from_utf8(slice).map_err(|_| SerError::InvalidData)
        }
    }

    /// Reads a length-prefixed byte slice.
    #[inline]
    pub fn read_bytes(&mut self) -> Result<&'a [u8], SerError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    /// Reads a vector of strings.
    #[inline]
    pub fn read_string_vec(&mut self) -> Result<Vec<String>, SerError> {
        #[cfg(not(feature = "std"))]
        {
            return Err(SerError::UnsupportedType);
        }
        #[cfg(feature = "std")]
        {
            let len = self.read_varint()? as usize;
            let mut vec = Vec::with_capacity(len);
            
            for _ in 0..len {
                vec.push(self.read_str()?.to_string());
            }
            
            Ok(vec)
        }
    }

    /// Reads a vector of 8-bit integers.
    #[inline]
    pub fn read_u8_vec(&mut self) -> Result<Vec<u8>, SerError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        let mut vec = Vec::with_capacity(len);
        unsafe {
            vec.set_len(len);
            ptr::copy_nonoverlapping(
                self.data.as_ptr().add(self.pos),
                vec.as_mut_ptr(),
                len
            );
        }
        self.pos += len;
        Ok(vec)
    }

    /// Reads a vector of 16-bit integers.
    #[inline]
    pub fn read_u16_vec(&mut self) -> Result<Vec<u16>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_varint()? as u16);
        }
        Ok(vec)
    }

    /// Reads a vector of 64-bit integers.
    #[inline]
    pub fn read_u64_vec(&mut self) -> Result<Vec<u64>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_varint()?);
        }
        Ok(vec)
    }

    /// Reads a vector of signed 8-bit integers.
    #[inline]
    pub fn read_i8_vec(&mut self) -> Result<Vec<i8>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_signed_varint()? as i8);
        }
        Ok(vec)
    }

    /// Reads a vector of signed 16-bit integers.
    #[inline]
    pub fn read_i16_vec(&mut self) -> Result<Vec<i16>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_signed_varint()? as i16);
        }
        Ok(vec)
    }

    /// Reads a vector of signed 64-bit integers.
    #[inline]
    pub fn read_i64_vec(&mut self) -> Result<Vec<i64>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_signed_varint()?);
        }
        Ok(vec)
    }

    /// Reads a vector of 32-bit floats.
    #[inline]
    pub fn read_f32_vec(&mut self) -> Result<Vec<f32>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_f32()?);
        }
        Ok(vec)
    }

    /// Reads a vector of 64-bit floats.
    #[inline]
    pub fn read_f64_vec(&mut self) -> Result<Vec<f64>, SerError> {
        let len = self.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(self.read_f64()?);
        }
        Ok(vec)
    }
}

pub trait JaguarSerialize {
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError>;
}

pub trait JaguarDeserialize<'a>: Sized {
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError>;
}

impl JaguarSerialize for u8 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_u8(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for u8 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_u8()
    }
}

impl JaguarSerialize for u32 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_varint(*self as u64)
    }
}

impl<'a> JaguarDeserialize<'a> for u32 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(de.read_varint()? as u32)
    }
}

impl JaguarSerialize for i32 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_signed_varint(*self as i64)
    }
}

impl<'a> JaguarDeserialize<'a> for i32 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(de.read_signed_varint()? as i32)
    }
}

impl JaguarSerialize for String {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_str(self)
    }
}

impl<'a> JaguarDeserialize<'a> for String {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(alloc::string::String::from(de.read_str()?))
    }
}

pub fn serialize<T: JaguarSerialize>(value: &T) -> Result<Vec<u8>, SerError> {
    let mut ser = JaguarSerializer::new();
    value.serialize(&mut ser)?;
    Ok(ser.finish())
}

pub fn deserialize<'a, T: JaguarDeserialize<'a>>(data: &'a [u8]) -> Result<T, SerError> {
    let mut de = JaguarDeserializer::new(data);
    T::deserialize(&mut de)
}

impl JaguarSerialize for u128 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        let high = (*self >> 64) as u64;
        let low = *self as u64;
        ser.write_varint(high)?;
        ser.write_varint(low)
    }
}

impl<'a> JaguarDeserialize<'a> for u128 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        let high = de.read_varint()?;
        let low = de.read_varint()?;
        Ok(((high as u128) << 64) | (low as u128))
    }
}

impl JaguarSerialize for u16 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_varint(*self as u64)
    }
}

impl<'a> JaguarDeserialize<'a> for u16 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(de.read_varint()? as u16)
    }
}

impl JaguarSerialize for u64 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_varint(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for u64 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_varint()
    }
}

impl JaguarSerialize for i8 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_signed_varint(*self as i64)
    }
}

impl<'a> JaguarDeserialize<'a> for i8 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(de.read_signed_varint()? as i8)
    }
}

impl JaguarSerialize for i16 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_signed_varint(*self as i64)
    }
}

impl<'a> JaguarDeserialize<'a> for i16 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(de.read_signed_varint()? as i16)
    }
}

impl JaguarSerialize for i64 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_signed_varint(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for i64 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_signed_varint()
    }
}

impl JaguarSerialize for f32 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_f32(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for f32 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_f32()
    }
}

impl JaguarSerialize for f64 {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_f64(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for f64 {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_f64()
    }
}

impl JaguarSerialize for bool {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_bool(*self)
    }
}

impl<'a> JaguarDeserialize<'a> for bool {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        de.read_bool()
    }
}

impl<T: JaguarSerialize> JaguarSerialize for Vec<T> {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_varint(self.len() as u64)?;
        for item in self {
            item.serialize(ser)?;
        }
        Ok(())
    }
}

impl<'a, T: JaguarDeserialize<'a>> JaguarDeserialize<'a> for Vec<T> {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        let len = de.read_varint()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(de)?);
        }
        Ok(vec)
    }
}

impl<A: JaguarSerialize, B: JaguarSerialize> JaguarSerialize for (A, B) {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        self.0.serialize(ser)?;
        self.1.serialize(ser)
    }
}

impl<'a, A: JaguarDeserialize<'a>, B: JaguarDeserialize<'a>> JaguarDeserialize<'a> for (A, B) {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        let a = A::deserialize(de)?;
        let b = B::deserialize(de)?;
        Ok((a, b))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrRef<'a>(pub &'a str);

impl<'a> JaguarSerialize for StrRef<'a> {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.write_str(self.0)
    }
}

impl<'a> JaguarDeserialize<'a> for StrRef<'a> {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        Ok(StrRef(de.read_str()?))
    }
}

// Add implementations for fixed-length arrays
impl<const N: usize> JaguarSerialize for [u8; N] {
    #[inline]
    fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
        ser.ensure_space(N);
        unsafe {
            ser.write_bytes_unchecked(self);
        }
        Ok(())
    }
}

impl<'a, const N: usize> JaguarDeserialize<'a> for [u8; N] {
    #[inline]
    fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
        if de.pos + N > de.data.len() {
            return Err(SerError::BufferTooSmall);
        }
        let mut result = [0u8; N];
        unsafe {
            ptr::copy_nonoverlapping(
                de.data.as_ptr().add(de.pos),
                result.as_mut_ptr(),
                N
            );
        }
        de.pos += N;
        Ok(result)
    }
}

macro_rules! impl_fixed_array {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> JaguarSerialize for [$t; N] {
                #[inline]
                fn serialize(&self, ser: &mut JaguarSerializer) -> Result<(), SerError> {
                    ser.write_varint(N as u64)?;
                    for item in self {
                        item.serialize(ser)?;
                    }
                    Ok(())
                }
            }

            impl<'a, const N: usize> JaguarDeserialize<'a> for [$t; N] {
                #[inline]
                fn deserialize(de: &mut JaguarDeserializer<'a>) -> Result<Self, SerError> {
                    let len = de.read_varint()? as usize;
                    if len != N {
                        return Err(SerError::InvalidLength);
                    }
                    let mut result = Vec::with_capacity(N);
                    for _ in 0..N {
                        result.push(<$t>::deserialize(de)?);
                    }
                    Ok(result.try_into().unwrap())
                }
            }
        )*
    };
}

impl_fixed_array!(u16, u32, u64, i8, i16, i32, i64, f32, f64, bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encode() {
        let mut ser = JaguarSerializer::new();

        ser.write_varint(0).unwrap();
        ser.write_varint(127).unwrap();
        ser.write_varint(128).unwrap();
        ser.write_varint(16383).unwrap();
    
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        
        assert_eq!(de.read_varint().unwrap(), 0);
        assert_eq!(de.read_varint().unwrap(), 127);
        assert_eq!(de.read_varint().unwrap(), 128);
        assert_eq!(de.read_varint().unwrap(), 16383);
    }

    #[test]
    fn test_string_roundtrip() {
        let original = "Hello, world! 🚀";

        let mut ser = JaguarSerializer::new();
        ser.write_str(original).unwrap();
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        let decoded = de.read_str().unwrap();
        
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_float_compression() {
        let mut ser = JaguarSerializer::new();

        ser.write_f32(0.0).unwrap();
        ser.write_f32(1.0).unwrap();
        ser.write_f32(-1.0).unwrap();
        ser.write_f32(3.14159).unwrap();
        
        let data = ser.data();

        assert_eq!(data[0], 0);
        assert_eq!(data[1], 1);
        assert_eq!(data[2], 2); 
        assert_eq!(data[3], 255); // needs full encoding
    }

    #[test]
    fn test_bool_slice_roundtrip() {
        let bools: Vec<bool> = (0..10000).map(|i| i % 3 == 0).collect();
        let mut ser = JaguarSerializer::new();
        ser.write_bool_slice(&bools).unwrap();
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        let decoded = de.read_bool_vec().unwrap();
        assert_eq!(bools, decoded);

        // check for size reduction
        assert!(data.len() < bools.len() / 2, "size should be reduced by at least 2x");
    }

    #[test]
    fn test_varint_micro_benchmark() {
        let values: Vec<u64> = (0..10000).map(|i| if i % 2 == 0 { i as u64 } else { (i as u64) * 1000 }).collect();
        let mut ser = JaguarSerializer::new();

        for v in &values {
            ser.write_varint(*v).unwrap();
        }

        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        for orig in &values {
            let decoded = de.read_varint().unwrap();
            assert_eq!(*orig, decoded);
        }
    }

    #[test]
    fn test_u128_roundtrip() {
        let value = u128::MAX;
        let mut ser = JaguarSerializer::new();
        value.serialize(&mut ser).unwrap();
        let data = ser.finish();
        
        let mut de = JaguarDeserializer::new(&data);
        let decoded = u128::deserialize(&mut de).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_fixed_array_roundtrip() {
        // [u8; 32]
        let pubkey = [1u8; 32];

        let mut ser = JaguarSerializer::new();
        pubkey.serialize(&mut ser).unwrap();
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        let decoded = <[u8; 32]>::deserialize(&mut de).unwrap();

        assert_eq!(pubkey, decoded);

        // [u32; 4]
        let ints: [u32; 4] = [1, 2, 3, 4];

        let mut ser = JaguarSerializer::new();
        ints.serialize(&mut ser).unwrap();
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        let decoded = <[u32; 4]>::deserialize(&mut de).unwrap();

        assert_eq!(ints, decoded);

        // [bool; 8]
        let bools: [bool; 8] = [true, false, true, false, true, false, true, false];

        let mut ser = JaguarSerializer::new();
        bools.serialize(&mut ser).unwrap();
        let data = ser.finish();

        let mut de = JaguarDeserializer::new(&data);
        let decoded = <[bool; 8]>::deserialize(&mut de).unwrap();

        assert_eq!(bools, decoded);
    }

    #[test]
    fn test_fixed_array_invalid_length() {
        let data = [1u32, 2, 3];
        let mut ser = JaguarSerializer::new();
        ser.write_varint(3).unwrap();

        for &x in &data {
            x.serialize(&mut ser).unwrap();
        }
        let serialized = ser.finish();
        
        let mut de = JaguarDeserializer::new(&serialized);

        assert!(matches!(<[u32; 4]>::deserialize(&mut de), Err(SerError::InvalidLength)));
    }
}

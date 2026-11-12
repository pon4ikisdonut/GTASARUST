use crate::error::FormatError;

/// Bounds-checked little-endian cursor over an immutable byte slice.
#[derive(Debug, Clone)]
pub struct BinReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn seek(&mut self, pos: usize) -> Result<(), FormatError> {
        if pos > self.data.len() {
            return Err(FormatError::UnexpectedEof {
                offset: self.pos,
                needed: pos.saturating_sub(self.pos),
                remaining: self.remaining(),
            });
        }
        self.pos = pos;
        Ok(())
    }

    pub fn skip(&mut self, count: usize) -> Result<(), FormatError> {
        let next = self
            .pos
            .checked_add(count)
            .ok_or(FormatError::UnexpectedEof {
                offset: self.pos,
                needed: count,
                remaining: self.remaining(),
            })?;
        self.seek(next)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], FormatError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(FormatError::UnexpectedEof {
                offset: self.pos,
                needed: len,
                remaining: self.remaining(),
            })?;
        if end > self.data.len() {
            return Err(FormatError::UnexpectedEof {
                offset: self.pos,
                needed: len,
                remaining: self.remaining(),
            });
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], FormatError> {
        let slice = self.read_bytes(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub fn read_u8(&mut self) -> Result<u8, FormatError> {
        Ok(self.read_array::<1>()?[0])
    }

    pub fn read_i16_le(&mut self) -> Result<i16, FormatError> {
        Ok(i16::from_le_bytes(self.read_array::<2>()?))
    }

    pub fn read_u16_le(&mut self) -> Result<u16, FormatError> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, FormatError> {
        Ok(i32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, FormatError> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_f32_le(&mut self) -> Result<f32, FormatError> {
        Ok(f32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_vec3_le(&mut self) -> Result<[f32; 3], FormatError> {
        Ok([
            self.read_f32_le()?,
            self.read_f32_le()?,
            self.read_f32_le()?,
        ])
    }

    pub fn read_fixed_str(&mut self, len: usize) -> Result<String, FormatError> {
        let bytes = self.read_bytes(len)?;
        let nul = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Ok(String::from_utf8_lossy(&bytes[..nul]).trim().to_string())
    }
}

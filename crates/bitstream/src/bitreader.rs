use anyhow::{Result, anyhow};

#[allow(dead_code)]
#[derive(Clone)]
pub struct BitReader<'input> {
    pub byte_buf: &'input [u8], // Source data to read bits from
    pub byte_index: usize,      // The current byte in the slice
    pub bit_offset: u8,         // The current bit within the byte
}

/// Conceptually, a bit-level cursor over a stream of bytes.
#[allow(dead_code)]
impl<'input> BitReader<'input> {
    pub fn from_bytes(data: &'input [u8]) -> BitReader<'input> {
        Self {
            byte_buf: data,
            byte_index: 0,
            bit_offset: 7,
        }
    }

    /// Advance the internal bit + byte index
    pub fn read(&mut self, n: usize) -> Result<u32> {
        let val = self.peek(n)?; // Reuse our peek method to read the correct value.
        self.advance(n)?; // Then, move forward by n bits.

        Ok(val)
    }

    /// Doesn't change internal position, but allows a read of N bits ahead.
    /// TODO: We can make this much more efficient later on at the optimization stage.
    pub fn peek(&self, n: usize) -> Result<u32> {
        let mut byte_index = self.byte_index;
        let mut bit_offset: usize = self.bit_offset as usize;
        let mut read_out = 0u32;
        let mut bits_read = 0;
        let bits_remaining = (self.byte_buf.len() - byte_index) * 8 + (7 - bit_offset);
        if bits_remaining < n {
            return Err(anyhow!("Not enough space to read!"));
        }

        while bits_read < n {
            if bit_offset == 7 && bits_read + 8 <= n {
                read_out |= (self.byte_buf[byte_index] as u32) << (n - bits_read - 8);
                bits_read += 8;
                byte_index += 1;
            } else {
                let cur_byte = self.byte_buf[byte_index];
                for _ in 0..(n - bits_read).min(bit_offset + 1) {
                    let bit = (cur_byte >> bit_offset) & 1;
                    read_out |= (bit as u32) << (n - bits_read - 1);
                    bits_read += 1;

                    if bit_offset == 0 {
                        byte_index += 1;
                        bit_offset = 7;
                        break;
                    } else {
                        bit_offset -= 1;
                    }
                }
            }
        }

        Ok(read_out)
    }

    /// Decrements the internal bit/byte index. Reads N bits backwards.
    pub fn rewind(&mut self, n: usize) -> Result<()> {
        let prior_bits = self.byte_index * 8 + (7 - self.bit_offset as usize);
        if prior_bits < n {
            return Err(anyhow!("Too many bits to rewind backwards"));
        }

        let new_global_index = prior_bits - n;
        self.byte_index = new_global_index / 8;
        self.bit_offset = 7 - (new_global_index % 8) as u8;

        Ok(())
    }

    /// Return the current bits position
    pub fn position(&self) -> usize {
        self.byte_index * 8 + (7 - self.bit_offset as usize)
    }

    /// Unsigned Exp-Golomb
    pub fn read_ue(&mut self) -> Result<u32> {
        let mut leading_zero_bits = 0;

        // Count leading zeros
        while self.read(1)? == 0 {
            leading_zero_bits += 1;
            if leading_zero_bits > 31 {
                return Err(anyhow!("Too many leading zeros in Exp-Golomb"));
            }
        }

        if leading_zero_bits == 0 {
            return Ok(0);
        }

        // Read suffix bits
        let suffix = self.read(leading_zero_bits)?;
        Ok((1 << leading_zero_bits) - 1 + suffix)
    }

    /// Same encoding as read_ue, but maps unsigned to signed integers
    pub fn read_se(&mut self) -> Result<i32> {
        let ue_val = self.read_ue()?;
        let signed_val = if ue_val % 2 == 0 {
            -((ue_val / 2) as i32)
        } else {
            ((ue_val + 1) / 2) as i32
        };
        Ok(signed_val)
    }

    /// Move the cursor forward by n bits
    fn advance(&mut self, n: usize) -> Result<()> {
        let total_bits = self.byte_buf.len() * 8;
        let global_bit_index = self.byte_index * 8 + (7 - self.bit_offset as usize);

        if global_bit_index + n > total_bits {
            return Err(anyhow!("Not enough bits to advance!"));
        }

        let new_global_bit_index = global_bit_index + n;

        self.byte_index = new_global_bit_index / 8;
        self.bit_offset = 7 - (new_global_bit_index % 8) as u8;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_single_bit_at_a_time() -> anyhow::Result<()> {
        // 0b10101010 → bits: 1,0,1,0,1,0,1,0
        let data = &[0b10101010];
        let mut reader = BitReader::from_bytes(data);
        assert_eq!(reader.read(1)?, 1);
        assert_eq!(reader.read(1)?, 0);
        assert_eq!(reader.read(1)?, 1);
        assert_eq!(reader.read(1)?, 0);
        assert_eq!(reader.read(1)?, 1);
        assert_eq!(reader.read(1)?, 0);
        assert_eq!(reader.read(1)?, 1);
        assert_eq!(reader.read(1)?, 0);
        Ok(())
    }

    #[test]
    fn test_read_multiple_bits_across_byte_boundary() -> anyhow::Result<()> {
        // Data: two bytes: [0b11001100, 0b10101010]
        let data = &[0b11001100, 0b10101010];
        let mut reader = BitReader::from_bytes(data);

        // First byte
        assert_eq!(reader.read(4)?, 0b1100);
        assert_eq!(reader.read(4)?, 0b1100);

        // Second byte
        assert_eq!(reader.read(4)?, 0b1010);
        assert_eq!(reader.read(4)?, 0b1010);
        Ok(())
    }

    #[test]
    fn test_peek_does_not_advance() -> anyhow::Result<()> {
        let data = &[0b11110000];
        let mut reader = BitReader::from_bytes(data);

        // Ensure peek doesn't change position
        let peek_val = reader.peek(4)?;
        assert_eq!(peek_val, 0b1111);
        assert_eq!(reader.position(), 0);

        // Check to see if read changes position (and returns same val as peek prior)
        assert_eq!(reader.read(4)?, 0b1111);
        assert_eq!(reader.position(), 4);
        Ok(())
    }

    #[test]
    fn test_rewind_functionality() -> anyhow::Result<()> {
        let data = &[0b11110000];
        let mut reader = BitReader::from_bytes(data);

        assert_eq!(reader.read(4)?, 0b1111);
        let pos_after = reader.position();
        assert_eq!(pos_after, 4);

        reader.rewind(2)?;
        assert_eq!(reader.position(), pos_after - 2);

        let bits = reader.read(2)?;
        assert_eq!(bits, 0b11);
        Ok(())
    }

    #[test]
    fn test_position_tracking() -> anyhow::Result<()> {
        let data = &[0b10101010];
        let mut reader = BitReader::from_bytes(data);

        // Check that initial pos is 0
        assert_eq!(reader.position(), 0);

        reader.read(3)?;
        assert_eq!(reader.position(), 3);

        reader.read(5)?;
        assert_eq!(reader.position(), 8);
        Ok(())
    }

    #[test]
    fn test_read_ue_complex() -> anyhow::Result<()> {
        // Encode 10 in UE:
        // For n = 10, n+1 = 11 → binary "1011" (4 bits).
        // Leading zeros: 3 zeros, then "1011" gives "0001011".
        // Place these bits at the beginning of a byte; pad remaining bits.
        let encoded = &[0b00010110]; // "0001011" with extra 0 as padding.
        let mut reader = BitReader::from_bytes(encoded);
        assert_eq!(reader.read_ue()?, 10);
        Ok(())
    }

    #[test]
    fn test_read_ue_concatenated() -> anyhow::Result<()> {
        // We want: "1" (0), then "010" (1), then "011" (2), then "00100" (3)
        // Concatenated bits: 101001100100 (12 bits total)
        // First byte: 10100110 = 0xA6
        // Second byte: 0100 padded to 8 bits: 01000000 = 0x40
        let encoded = &[0xA6, 0x40];
        let mut reader = BitReader::from_bytes(encoded);
        assert_eq!(reader.read_ue()?, 0);
        assert_eq!(reader.read_ue()?, 1);
        assert_eq!(reader.read_ue()?, 2);
        assert_eq!(reader.read_ue()?, 3);
        Ok(())
    }

    #[test]
    fn test_read_se_individual() -> anyhow::Result<()> {
        // Mapping: UE 0 -> SE 0, UE 1 -> SE 1, UE 2 -> SE -1, UE 3 -> SE 2, UE 4 -> SE -2.
        let mut reader = BitReader::from_bytes(&[0b10000000]); // "1" → UE0 → SE 0
        assert_eq!(reader.read_se()?, 0);

        let mut reader = BitReader::from_bytes(&[0b01000000]); // "010" → UE1 → SE 1
        assert_eq!(reader.read_se()?, 1);

        let mut reader = BitReader::from_bytes(&[0b01100000]); // "011" → UE2 → SE -1
        assert_eq!(reader.read_se()?, -1);

        let mut reader = BitReader::from_bytes(&[0b00100000]); // "00100" → UE3 → SE 2
        assert_eq!(reader.read_se()?, 2);

        // For UE 4 -> SE -2, "00101" → 0b00101000:
        let mut reader = BitReader::from_bytes(&[0b00101000]);
        assert_eq!(reader.read_se()?, -2);

        Ok(())
    }

    #[test]
    fn test_error_on_insufficient_bits() {
        let data = &[0b00000000]; // 8 bits (so reading 9 should error out)
        let mut reader = BitReader::from_bytes(data);
        assert!(reader.read(9).is_err());
    }

    #[test]
    fn test_error_on_rewind_too_far() {
        let data = &[0b11110000];
        let mut reader = BitReader::from_bytes(data);

        reader.read(4).unwrap();
        assert!(reader.rewind(8).is_err()); // Shouldn't be allowed to rewind backwards 8 bits over 4 read so far
    }

    #[test]
    fn test_multiple_sequential_reads() -> anyhow::Result<()> {
        let data = &[0b11001100, 0b10101010, 0b11110000];
        let mut reader = BitReader::from_bytes(data);

        assert_eq!(reader.read(3)?, 0b110);
        assert_eq!(reader.read(5)?, 0b01100);

        // Now we are at the beginning of the second byte.
        assert_eq!(reader.read(4)?, 0b1010);

        // Read next 8 bits spanning the rest of the second byte and part of third.
        assert_eq!(reader.read(8)?, 0b10101111);

        Ok(())
    }
}

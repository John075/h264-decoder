use anyhow::{Result, anyhow};

pub struct BitReader<'input> {
    pub byte_buf: &'input [u8], // Source data to read bits from
    pub byte_index: usize,      // The current byte in the slice
    pub bit_offset: u8,         // The current bit within the byte
}

/// Conceptually, a bit-level cursor over a stream of bytes.
impl<'input> BitReader<'input> {
    /// Advance the internal bit + byte index
    pub fn read(&mut self, n: usize) -> Result<u32> {
        let val = self.peek(n)?; // Reuse our peek method to read the correct value.
        self.advance(n)?; // Then, move forward by n bits.

        return Ok(val);
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

        return Ok(read_out);
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

#[cfg(test)]
mod tests {
    use crate::bitreader::BitReader;
    use anyhow::Result;

    /// Helper to create a new BitReader with bit_offset starting at 7 (MSB).
    fn make_reader(data: &[u8]) -> BitReader {
        BitReader {
            byte_buf: data,
            byte_index: 0,
            bit_offset: 7,
        }
    }

    #[test]
    fn test_read_single_bit_at_a_time() -> Result<()> {
        // 0b10101010 → bits: 1,0,1,0,1,0,1,0
        let data = &[0b10101010];
        let mut reader = make_reader(data);
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
    fn test_read_multiple_bits_across_byte_boundary() -> Result<()> {
        // Data: two bytes: [0b11001100, 0b10101010]
        let data = &[0b11001100, 0b10101010];
        let mut reader = make_reader(data);

        // First byte
        assert_eq!(reader.read(4)?, 0b1100);
        assert_eq!(reader.read(4)?, 0b1100);

        // Second byte
        assert_eq!(reader.read(4)?, 0b1010);
        assert_eq!(reader.read(4)?, 0b1010);
        Ok(())
    }

    #[test]
    fn test_peek_does_not_advance() -> Result<()> {
        let data = &[0b11110000];
        let mut reader = make_reader(data);

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
    fn test_rewind_functionality() -> Result<()> {
        let data = &[0b11110000];
        let mut reader = make_reader(data);

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
    fn test_position_tracking() -> Result<()> {
        let data = &[0b10101010];
        let mut reader = make_reader(data);

        // Check that initial pos is 0
        assert_eq!(reader.position(), 0);

        reader.read(3)?;
        assert_eq!(reader.position(), 3);

        reader.read(5)?;
        assert_eq!(reader.position(), 8);
        Ok(())
    }

    #[test]
    fn test_read_ue_complex() -> Result<()> {
        // Encode 10 in UE:
        // For n = 10, n+1 = 11 → binary "1011" (4 bits).
        // Leading zeros: 3 zeros, then "1011" gives "0001011".
        // Place these bits at the beginning of a byte; pad remaining bits.
        let encoded = &[0b00010110]; // "0001011" with extra 0 as padding.
        let mut reader = make_reader(encoded);
        assert_eq!(reader.read_ue()?, 10);
        Ok(())
    }

    #[test]
    fn test_read_ue_concatenated() -> Result<()> {
        // We want: "1" (0), then "010" (1), then "011" (2), then "00100" (3)
        // Concatenated bits: 101001100100 (12 bits total)
        // First byte: 10100110 = 0xA6
        // Second byte: 0100 padded to 8 bits: 01000000 = 0x40
        let encoded = &[0xA6, 0x40];
        let mut reader = make_reader(encoded);
        assert_eq!(reader.read_ue()?, 0);
        assert_eq!(reader.read_ue()?, 1);
        assert_eq!(reader.read_ue()?, 2);
        assert_eq!(reader.read_ue()?, 3);
        Ok(())
    }

    #[test]
    fn test_read_se_individual() -> Result<()> {
        // Mapping: UE 0 -> SE 0, UE 1 -> SE 1, UE 2 -> SE -1, UE 3 -> SE 2, UE 4 -> SE -2.
        let mut reader = make_reader(&[0b10000000]); // "1" → UE0 → SE 0
        assert_eq!(reader.read_se()?, 0);

        let mut reader = make_reader(&[0b01000000]); // "010" → UE1 → SE 1
        assert_eq!(reader.read_se()?, 1);

        let mut reader = make_reader(&[0b01100000]); // "011" → UE2 → SE -1
        assert_eq!(reader.read_se()?, -1);

        let mut reader = make_reader(&[0b00100000]); // "00100" → UE3 → SE 2
        assert_eq!(reader.read_se()?, 2);

        // For UE 4 -> SE -2, "00101" → 0b00101000:
        let mut reader = make_reader(&[0b00101000]);
        assert_eq!(reader.read_se()?, -2);

        Ok(())
    }


    #[test]
    fn test_error_on_insufficient_bits() {
        let data = &[0b00000000]; // 8 bits (so reading 9 should error out)
        let mut reader = make_reader(data);
        assert!(reader.read(9).is_err());
    }

    #[test]
    fn test_error_on_rewind_too_far() {
        let data = &[0b11110000];
        let mut reader = make_reader(data);

        reader.read(4).unwrap();
        assert!(reader.rewind(8).is_err()); // Shouldn't be allowed to rewind backwards 8 bits over 4 read so far
    }

    #[test]
    fn test_multiple_sequential_reads() -> Result<()> {
        let data = &[0b11001100, 0b10101010, 0b11110000];
        let mut reader = make_reader(data);

        assert_eq!(reader.read(3)?, 0b110);
        assert_eq!(reader.read(5)?, 0b01100);

        // Now we are at the beginning of the second byte.
        assert_eq!(reader.read(4)?, 0b1010);

        // Read next 8 bits spanning the rest of the second byte and part of third.
        assert_eq!(reader.read(8)?, 0b10101111);

        Ok(())
    }
}

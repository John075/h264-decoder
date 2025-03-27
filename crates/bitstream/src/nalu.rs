use anyhow::anyhow;

/// Implemented as in 7.3.1 NAL unit syntax in Rec. ITU-T H.264 (04/2013)
/// Struct for holding NALU header information from a parsed byte
#[allow(dead_code)]
struct NaluHeader {
    forbidden_zero_bit: u8,
    /// Must be 0 to be considered valid
    nal_ref_idc: u8,
    nal_unit_type: u8,
}

#[allow(dead_code)]
impl NaluHeader {
    /// Creates a new NaluHeader struct. Parses the byte for the forbidden bit, ref idc and unit type.
    pub fn new(byte: u8) -> anyhow::Result<NaluHeader> {
        let forbidden_zero_bit = (byte >> 7) & 0x01;
        let nal_ref_idc = (byte >> 5) & 0x03;
        let nal_unit_type = byte & 0x1F;

        if forbidden_zero_bit == 1 {
            return Err(anyhow!("Forbidden bit in NALU Header cannot be 1"));
        }

        Ok(Self {
            forbidden_zero_bit,
            nal_ref_idc,
            nal_unit_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forbidden_bit_set() {
        // The forbidden bit is the top bit (bit 7).
        let data = 0x80;
        let result = NaluHeader::new(data);
        assert!(
            result.is_err(),
            "Expected an error when forbidden_zero_bit = 1"
        );
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Forbidden bit in NALU Header cannot be 1");
        }
    }

    #[test]
    fn test_valid_nalu_header() {
        //  forbidden_zero_bit = 0
        //  nal_ref_idc = 3 (bits 6..5 => 11)
        //  nal_unit_type = 5 (bits 4..0 => 5)
        let data = 0x65;
        let header = NaluHeader::new(data).expect("Should parse successfully");
        assert_eq!(header.forbidden_zero_bit, 0);
        assert_eq!(header.nal_ref_idc, 3);
        assert_eq!(header.nal_unit_type, 5);
    }

    #[test]
    fn test_other_byte_variations() {
        let data_zero = 0x00;
        let header_zero = NaluHeader::new(data_zero).unwrap();
        assert_eq!(header_zero.forbidden_zero_bit, 0);
        assert_eq!(header_zero.nal_ref_idc, 0);
        assert_eq!(header_zero.nal_unit_type, 0);

        // 0x7F => b0111_1111
        let data_7f = 0x7F;
        let header_7f = NaluHeader::new(data_7f).unwrap();
        assert_eq!(header_7f.forbidden_zero_bit, 0);
        assert_eq!(header_7f.nal_ref_idc, 3);
        assert_eq!(header_7f.nal_unit_type, 31);
    }
}

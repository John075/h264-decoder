#[allow(dead_code)]
pub fn split_annexb_nalus(data: &[u8]) -> Vec<&[u8]> {
    let mut nalus = Vec::new();
    let mut nalu_start: Option<usize> = None;
    let mut i = 0;

    while i + 3 <= data.len() {
        let start_code_len = if i + 4 <= data.len() && data[i..i + 4] == [0, 0, 0, 1] {
            4
        } else if data[i..i + 3] == [0, 0, 1] {
            3
        } else {
            i += 1;
            continue;
        };

        if let Some(start) = nalu_start {
            if start < i {
                nalus.push(&data[start..i]);
            }
        }

        nalu_start = Some(i + start_code_len);
        i += start_code_len;
    }

    // Push the final NALU if any
    if let Some(start) = nalu_start {
        if start < data.len() {
            nalus.push(&data[start..]);
        }
    }

    nalus
}

#[cfg(test)]
mod tests {
    use super::*;

    // Empty input should return an empty vector.
    #[test]
    fn test_empty_input() {
        let data: &[u8] = &[];
        let nalus = split_annexb_nalus(data);

        assert!(nalus.is_empty(), "Expected no NALUs for empty input");
    }

    // Data with no valid start code should return an empty vector.
    #[test]
    fn test_no_start_code() {
        let data = &[0x12, 0x34, 0x56, 0x78];
        let nalus = split_annexb_nalus(data);

        assert!(
            nalus.is_empty(),
            "Expected no NALUs when no start code is present"
        );
    }

    // A single NALU with a 3-byte start code.
    #[test]
    fn test_single_nalu_3byte_start() {
        // Start code 00 00 01 followed by payload [0x67, 0x68, 0x69]
        let data = &[0x00, 0x00, 0x01, 0x67, 0x68, 0x69];
        let nalus = split_annexb_nalus(data);

        assert_eq!(nalus.len(), 1);
        assert_eq!(nalus[0], &[0x67, 0x68, 0x69]);
    }

    // A single NALU with a 4-byte start code.
    #[test]
    fn test_single_nalu_4byte_start() {
        // Start code 00 00 00 01 followed by payload [0x65, 0x66, 0x67]
        let data = &[0x00, 0x00, 0x00, 0x01, 0x65, 0x66, 0x67];
        let nalus = split_annexb_nalus(data);

        assert_eq!(nalus.len(), 1);
        assert_eq!(nalus[0], &[0x65, 0x66, 0x67]);
    }

    // Multiple NALUs with mixed 3-byte and 4-byte start codes.
    #[test]
    fn test_multiple_nalus() {
        let data = &[
            0x00, 0x00, 0x00, 0x01, // 4-byte start code
            0x67, 0x68, 0x69, // payload 1
            0x00, 0x00, 0x01, // 3-byte start code
            0x65, 0x66, 0x67, // payload 2
            0x00, 0x00, 0x00, 0x01, // 4-byte start code
            0x68, 0x69, 0x6A, // payload 3
        ];

        let nalus = split_annexb_nalus(data);
        assert_eq!(nalus.len(), 3);
        assert_eq!(nalus[0], &[0x67, 0x68, 0x69]);
        assert_eq!(nalus[1], &[0x65, 0x66, 0x67]);
        assert_eq!(nalus[2], &[0x68, 0x69, 0x6A]);
    }

    // Adjacent start codes resulting in an empty NALU.
    #[test]
    fn test_adjacent_start_codes() {
        // Here, two start codes appear back-to-back; the empty slice should be skipped.
        let data = &[
            0x00, 0x00, 0x00, 0x01, // start code for NALU1
            0x67, 0x68, // payload for NALU1
            0x00, 0x00,
            0x01, // start code (expected to produce empty NALU, but will be skipped)
            0x00, 0x00, 0x00, 0x01, // start code for NALU2
            0x65, 0x66, // payload for NALU2
        ];

        let nalus = split_annexb_nalus(data);
        assert_eq!(nalus.len(), 2);
        assert_eq!(nalus[0], &[0x67, 0x68]);
        assert_eq!(nalus[1], &[0x65, 0x66]);
    }

    // Trailing data after the last start code.
    #[test]
    fn test_trailing_data() {
        // Two NALUs, where the second NALU extends to the end of the buffer.
        let data = &[
            0x00, 0x00, 0x01, // start code for NALU1
            0x65, 0x66, 0x67, 0x68, // payload for NALU1
            0x00, 0x00, 0x01, // start code for NALU2
            0x68, 0x69, 0x6A, // payload for NALU2 (till end of buffer)
        ];

        let nalus = split_annexb_nalus(data);
        assert_eq!(nalus.len(), 2);
        assert_eq!(nalus[0], &[0x65, 0x66, 0x67, 0x68]);
        assert_eq!(nalus[1], &[0x68, 0x69, 0x6A]);
    }

    // Partial start code at the very end of the data.
    #[test]
    fn test_partial_start_code_at_end() {
        // Should treat the partial bytes as normal data (no start code).
        let data = &[
            0x00, 0x00, 0x01, // start code for NALU1
            0x67, 0x68, // payload for NALU1
            0x00, 0x00, // partial potential start code (incomplete)
        ];

        let nalus = split_annexb_nalus(data);
        // Only one valid NALU should be extracted.
        assert_eq!(nalus.len(), 1);
        assert_eq!(nalus[0], &[0x67, 0x68, 0x00, 0x00]);
    }

    // Data with no start code at beginning, then a valid start code later.
    #[test]
    fn test_start_code_not_at_beginning() {
        let data = &[
            0xFF, 0xFF, 0xFF, // junk bytes
            0x00, 0x00, 0x01, // valid start code
            0x65, 0x66, 0x67, // payload for the only NALU
        ];

        let nalus = split_annexb_nalus(data);
        // We only extract the NALU after the start code.
        assert_eq!(nalus.len(), 1);
        assert_eq!(nalus[0], &[0x65, 0x66, 0x67]);
    }
}

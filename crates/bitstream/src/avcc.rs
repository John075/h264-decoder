use anyhow::anyhow;

#[derive(Debug)]
#[allow(dead_code)]
struct AVCHeader<'input> {
    version: u8,
    avc_profile: u8,
    avc_compatability: u8,
    avc_level: u8,
    nalu_length_size_minus_one: u8,
    sps: Vec<&'input [u8]>,
    pps: Vec<&'input [u8]>,
}

#[allow(dead_code)]
impl<'input> AVCHeader<'input> {
    /// Provides AVCC header parsing functionality
    /// Data Structure Reference: https://stackoverflow.com/questions/24884827/possible-locations-for-sequence-picture-parameter-sets-for-h-264-stream
    pub fn new(data: &'input [u8]) -> anyhow::Result<Self> {
        if data.len() < 7 {
            return Err(anyhow!("AVCC header is below minimum size"));
        }
        let version = data[0];
        let avc_profile = data[1];
        let avc_compatability = data[2];
        let avc_level = data[3];
        let nalu_header_byte = data[4];
        let nalu_length_size_minus_one = nalu_header_byte & 0b11;

        if version != 1 {
            return Err(anyhow!("Incorrect version in AVCC header: {}", version));
        }

        if nalu_header_byte & 0b11111100 != 0b11111100 {
            return Err(anyhow!(
                "Invalid reserved bits in AVCC NALU length size byte: {:#010b}",
                nalu_header_byte
            ));
        }

        let mut offset = 5;
        let sps_count = data[offset] & 0b0001_1111; //number of SPS NALUs
        if data.len() < (7 + sps_count) as usize {
            return Err(anyhow!("AVCC header is below minimum size"));
        }

        offset += 1;
        let sps = Self::parse_nalus(data, sps_count, &mut offset, "SPS")?;

        let pps_count = data[offset] & 0b0001_1111; //number of PPS NALUs
        if data.len() < offset + 1 + pps_count as usize {
            return Err(anyhow!("AVCC header is below minimum size"));
        }

        offset += 1;
        let pps = Self::parse_nalus(data, pps_count, &mut offset, "PPS")?;

        Ok(Self {
            version,
            avc_profile,
            avc_compatability,
            avc_level,
            nalu_length_size_minus_one,
            sps,
            pps,
        })
    }

    /// Reads all NALUs from an AVCC formatted stream
    fn parse_nalus<'a>(
        data: &'a [u8],
        count: u8,
        offset: &mut usize,
        label: &str,
    ) -> anyhow::Result<Vec<&'a [u8]>> {
        let mut nalus = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if *offset + 2 > data.len() {
                return Err(anyhow!("Not enough data for {} size field", label));
            }
            let size = u16::from_be_bytes([data[*offset], data[*offset + 1]]) as usize;
            *offset += 2;

            if *offset + size > data.len() {
                return Err(anyhow!("Not enough data for {} payload", label));
            }

            nalus.push(&data[*offset..*offset + size]);
            *offset += size;
        }
        Ok(nalus)
    }
}

/// Read in all the NALUs within an AVCC formatted stream
#[allow(dead_code)]
pub fn read_avcc_stream(data: &[u8], nalu_length_size: usize) -> anyhow::Result<Vec<&[u8]>> {
    if !(0..=3).contains(&nalu_length_size) {
        return Err(anyhow!("Invalid NALU length size: {}", nalu_length_size));
    }

    // In the worst case, we'll have the NALU length field + 1 byte per NALU
    let mut nalus = Vec::with_capacity(data.len() / (nalu_length_size + 1));

    let mut i: usize = 0;
    while i < data.len() {
        let amount_to_read = match nalu_length_size {
            1 => data[i] as usize,
            2 => u16::from_be_bytes([data[i], data[i + 1]]) as usize,
            3 => {
                (((data[i + 2] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i] as u32))
                    as usize
            }
            4 => u32::from_be_bytes([data[i + 3], data[i + 2], data[i + 1], data[i]]) as usize,
            _ => unreachable!(),
        };

        i += nalu_length_size;
        if i + amount_to_read > data.len() {
            return Err(anyhow!("NALU length exceeds available data"));
        }

        nalus.push(&data[i..i + amount_to_read]);
        i += amount_to_read;
    }

    Ok(nalus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    /// Builds a minimal AVCC header-like buffer in memory, returning raw bytes.
    fn build_avcc_header(
        version: u8,
        mut nalu_len_size_minus_one: u8,
        sps_list: &[&[u8]],
        pps_list: &[&[u8]],
    ) -> Vec<u8> {
        let avc_profile = 0x42; // "Baseline" example
        let avc_compat = 0x00;
        let avc_level = 0x1E;

        if nalu_len_size_minus_one <= 3 {
            nalu_len_size_minus_one |= 0b11111100;
        }

        let mut header = vec![
            version,
            avc_profile,
            avc_compat,
            avc_level,
            nalu_len_size_minus_one,
            0,
        ];

        // SPS count
        let sps_count = sps_list.len() as u8;
        header[5] = 0b11100000 | sps_count; // store sps_count in lower 5 bits

        for sps in sps_list {
            let len_bytes = (sps.len() as u16).to_be_bytes();
            header.push(len_bytes[0]);
            header.push(len_bytes[1]);
            header.extend_from_slice(sps);
        }

        // PPS count
        let pps_count = pps_list.len() as u8;
        header.push(0b11100000 | pps_count); // store in lower 5 bits

        for pps in pps_list {
            let len_bytes = (pps.len() as u16).to_be_bytes();
            header.push(len_bytes[0]);
            header.push(len_bytes[1]);
            header.extend_from_slice(pps);
        }

        header
    }

    #[test]
    fn test_avcc_header_minimal_ok() -> Result<()> {
        // 1 SPS with 2 bytes, 1 PPS with 1 byte
        let sps_data = &[&[0xAA, 0xBB][..]];
        let pps_data = &[&[0xCC][..]];

        let header_bytes = build_avcc_header(1, 1, sps_data, pps_data);

        let parsed = AVCHeader::new(&header_bytes)?;
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.nalu_length_size_minus_one, 1);
        assert_eq!(parsed.sps.len(), 1);
        assert_eq!(parsed.sps[0], &[0xAA, 0xBB]);
        assert_eq!(parsed.pps.len(), 1);
        assert_eq!(parsed.pps[0], &[0xCC]);
        Ok(())
    }

    #[test]
    fn test_avcc_header_multiple_sps_pps() -> Result<()> {
        // 2 SPS, 2 PPS
        let sps_data = &[&[0x01, 0x02][..], &[0x03, 0x04, 0x05][..]];
        let pps_data = &[&[0x11, 0x22, 0x33][..], &[0x44][..]];

        let header_bytes = build_avcc_header(1, 3, sps_data, pps_data);
        let parsed = AVCHeader::new(&header_bytes)?;
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.nalu_length_size_minus_one, 3);
        assert_eq!(parsed.sps.len(), 2);
        assert_eq!(parsed.sps[0], &[0x01, 0x02]);
        assert_eq!(parsed.sps[1], &[0x03, 0x04, 0x05]);
        assert_eq!(parsed.pps.len(), 2);
        assert_eq!(parsed.pps[0], &[0x11, 0x22, 0x33]);
        assert_eq!(parsed.pps[1], &[0x44]);
        Ok(())
    }

    #[test]
    fn test_avcc_header_incorrect_version() {
        let header_bytes = build_avcc_header(2, 0, &[], &[]);
        let err = AVCHeader::new(&header_bytes).unwrap_err();
        assert!(err.to_string().contains("Incorrect version"));
    }

    #[test]
    fn test_avcc_header_invalid_nalu_length_size() {
        // nalu_length_size_minus_one = 5 is invalid
        let header_bytes = build_avcc_header(1, 5, &[], &[]);
        let err = AVCHeader::new(&header_bytes).unwrap_err();
        assert!(
            err.to_string()
                .contains("Invalid reserved bits in AVCC NALU length size byte")
        );
    }

    #[test]
    fn test_avcc_header_incomplete_sps_field() {
        // We'll say we have 1 SPS in the header, but won't provide enough bytes for size
        let mut header_bytes = build_avcc_header(1, 0, &[], &[]);
        // We claimed sps_count = 1, but we don't actually put in the two size bytes + data
        // so let's manipulate the relevant byte ( index = 5 ) to set sps_count=1
        header_bytes[5] = 0b11100001; // sps_count = 1
        header_bytes.truncate(6); // minimal length, no size, no data

        let err = AVCHeader::new(&header_bytes).unwrap_err();
        assert!(
            err.to_string()
                .contains("AVCC header is below minimum size")
        );
    }

    #[test]
    fn test_avcc_header_incomplete_sps_payload() {
        // We'll say 1 SPS of length 3, but only provide 2 bytes
        let mut header_bytes = build_avcc_header(1, 0, &[], &[]);
        header_bytes[5] = 0b11100001;
        header_bytes.extend_from_slice(&3u16.to_be_bytes()); // size = 3
        // We'll only add 2 bytes of actual data
        header_bytes.push(0xAA);
        header_bytes.push(0xBB);

        let err = AVCHeader::new(&header_bytes).unwrap_err();
        assert!(err.to_string().contains("Not enough data for SPS payload"));
    }

    /// Helper to build raw AVCC-like stream:
    fn build_avcc_stream(nalu_length_size: usize, nalus: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        for nalu in nalus {
            let length = nalu.len() as u32;
            match nalu_length_size {
                1 => out.push(length as u8),
                2 => out.extend_from_slice(&(length as u16).to_be_bytes()),
                3 => {
                    let b1 = (length >> 16) as u8;
                    let b2 = (length >> 8) as u8;
                    let b3 = length as u8;
                    out.extend_from_slice(&[b1, b2, b3]);
                }
                4 => out.extend_from_slice(&(length).to_be_bytes()),
                _ => panic!("Invalid test setup"),
            }
            out.extend_from_slice(nalu);
        }
        out
    }

    #[test]
    fn test_read_avcc_stream_1_byte_size() -> Result<()> {
        // 2 NALUs: each length in 1 byte
        let stream = build_avcc_stream(1, &[&[0xAA, 0xBB], &[0xCC]]);
        let nalus = read_avcc_stream(&stream, 1)?;
        assert_eq!(nalus.len(), 2);
        assert_eq!(nalus[0], &[0xAA, 0xBB]);
        assert_eq!(nalus[1], &[0xCC]);
        Ok(())
    }

    #[test]
    fn test_read_avcc_stream_2_byte_size() -> Result<()> {
        // 1 NALU of length 3
        let stream = build_avcc_stream(2, &[&[0x01, 0x02, 0x03]]);
        let nalus = read_avcc_stream(&stream, 2)?;
        assert_eq!(nalus.len(), 1);
        assert_eq!(nalus[0], &[0x01, 0x02, 0x03]);
        Ok(())
    }

    #[test]
    fn test_read_avcc_stream_4_byte_size_multiple() -> Result<()> {
        // 2 NALUs, each length stored in 4 bytes
        let stream = build_avcc_stream(1, &[&[0xDE, 0xAD], &[0xBE, 0xEF, 0x01]]);
        let nalus = read_avcc_stream(&stream, 1)?;
        assert_eq!(nalus.len(), 2);
        assert_eq!(nalus[0], &[0xDE, 0xAD]);
        assert_eq!(nalus[1], &[0xBE, 0xEF, 0x01]);
        Ok(())
    }

    #[test]
    fn test_read_avcc_stream_empty() -> Result<()> {
        // No data
        let nalus = read_avcc_stream(&[], 1)?;
        assert_eq!(nalus.len(), 0);
        Ok(())
    }

    #[test]
    fn test_read_avcc_stream_invalid_nalu_length_size_param() {
        let err = read_avcc_stream(&[], 5).unwrap_err();
        assert!(err.to_string().contains("Invalid NALU length size"));
    }

    #[test]
    fn test_read_avcc_stream_nalu_exceeds_data() {
        // We claim NALU length is 5 bytes, but only provide 3
        let mut data = vec![5u8]; // length = 5
        data.extend_from_slice(&[0x01, 0x02, 0x03]); // only 3 bytes
        let err = read_avcc_stream(&data, 1).unwrap_err();
        assert!(
            err.to_string()
                .contains("NALU length exceeds available data")
        );
    }
}

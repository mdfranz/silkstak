const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = 0xEDB88320 ^ (crc >> 1);
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        let idx = ((crc as u8) ^ byte) as usize;
        crc = CRC32_TABLE[idx] ^ (crc >> 8);
    }
    !crc
}

pub fn crc32_hex(data: &[u8]) -> String {
    format!("{:08x}", crc32(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(b""), 0x00000000);
    }

    #[test]
    fn test_crc32_hello() {
        assert_eq!(crc32(b"hello"), 0x3610A686);
        assert_eq!(crc32_hex(b"hello"), "3610a686");
    }

    #[test]
    fn test_crc32_deterministic() {
        let a = crc32(b"same string");
        let b = crc32(b"same string");
        assert_eq!(a, b);
    }

    #[test]
    fn test_crc32_different() {
        let a = crc32(b"hello");
        let b = crc32(b"world");
        assert_ne!(a, b);
    }
}

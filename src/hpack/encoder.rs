pub(crate) struct Encoder;

impl Encoder {
    
    pub(crate) fn encode_int(&self, value: u64, prefix_bits: u32) -> Vec<u8> {
        let max_number = 2_u64.pow(prefix_bits) - 1;
        if value < max_number {
            return vec![value as u8];
        }

        let mut encoded = vec![max_number as u8];
        let mut updated = value - max_number;

        while updated >= 128 {
            encoded.push(((updated % 128) as u8) + 128);
            updated /= 128;
        }

        encoded.push(updated as u8);
        encoded
    }

    pub(crate) fn encode_literal(&self, name: &str, value: &str) -> Vec<u8> {
        let mut encoded = vec![0u8];
        encoded.append(&mut self.encode_int(name.len() as u64, 7));
        encoded.append(&mut name.as_bytes().to_vec());
        encoded.append(&mut self.encode_int(value.len() as u64, 7));
        encoded.append(&mut value.as_bytes().to_vec());

        encoded
    }

    pub(crate) fn encode_indexed(&self, index: u32) -> Vec<u8> {
        let mut bytes = self.encode_int(index as u64, 7);
        bytes[0] |= 0x80;
        return bytes;
    }
}

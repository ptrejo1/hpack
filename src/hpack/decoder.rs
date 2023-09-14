use crate::hpack::header::Header;
use crate::hpack::header_table::HeaderTable;

pub(crate) struct Decoder{
    header_table: HeaderTable
}

impl Decoder {

    pub(crate) fn new() -> Decoder {
        Decoder{header_table: HeaderTable::new_default()}
    }

    pub(crate) fn decode_int(&self, data: &[u8], prefix_bits: u32) -> Result<(u64, usize), String> {
        if data.is_empty() {
            return Err("Decode data is empty!".to_string());
        }

        let max_number = 2_u64.pow(prefix_bits) - 1;
        let mut consumed = 1;
        let mut number = data[consumed - 1] as u64;
        if number < max_number {
            return Ok((number, consumed));
        }

        let mut m = 0;
        loop {
            consumed += 1;
            let next = data[consumed - 1];
            number += (next as u64 & 127) * 2_u64.pow(m);
            m += 7;

            if next & 128 != 128 {
                break
            }
        }

        Ok((number, consumed))
    }

    pub(crate) fn decode_literal(&self, data: &[u8], prefix: u32) -> Result<(Header, usize), String> {
        let (idx, mut consumed) = self.decode_int(data, prefix).unwrap();

        let name_result: (String, usize);
        if idx == 0 {
            let name_data = &data[consumed..data.len()];
            name_result = self.decode_string(name_data).unwrap();
            consumed += name_result.1
        } else {
            return Err("Invalid Table Index!".to_string());
        }

        let value_data = &data[consumed..data.len()];
        let value_result = self.decode_string(value_data).unwrap();
        Ok((Header{name: name_result.0, value: value_result.0}, consumed + value_result.1))
    }

    fn decode_string(&self, data: &[u8]) -> Result<(String, usize), String> {
        let (length, consumed) = self.decode_int(data, 7).unwrap();
        let end_idx = consumed + length as usize;
        let bytes = data[consumed..end_idx].to_owned();

        if !bytes.is_empty() && bytes[0] & 128 > 0 {
            return Err("Huffman not supported".to_string());
        }

        let value = String::from_utf8(bytes);
        Ok((value.unwrap(), end_idx))
    }

    pub(crate) fn decode_indexed(&self, data: &[u8]) -> Result<(Header, usize), String> {
        let (index, consumed) = self.decode_int(data, 7).unwrap();
        Ok((self.header_table[index.try_into().unwrap()].clone(), consumed))
    }
}

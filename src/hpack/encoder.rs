use crate::hpack::header::{EncodableHeader, Header};
use crate::hpack::header_table::HeaderTable;

pub(crate) struct Encoder{
    pub header_table: HeaderTable,
    header_table_changes:Vec<usize>
}

impl Encoder {

    const INDEX_NEVER: u8 = 16;
    const INDEX_INCREMENTAL: u8 = 68;

    pub(crate) fn new() -> Self {
        Self {
            header_table: HeaderTable::new_default(),
            header_table_changes: vec![],
        }
    }

    pub(crate) fn encode(&mut self, headers: &[Header]) -> Vec<u8> {
        let tuples: Vec<EncodableHeader> = headers.iter()
            .map(|x| {
                EncodableHeader {
                    name: x.name.clone(),
                    value: x.value.clone(),
                    is_sensitive: false,
                }
            })
            .collect();

        self.encode_headers(tuples.as_slice())
    }

    pub(crate) fn encode_headers(&mut self, headers: &[EncodableHeader]) -> Vec<u8> {
        let mut encoded = self.encode_header_table_changes();
        let mut encoded_headers: Vec<u8> = headers.iter()
            .map(|x| self.encode_header(&x.name, &x.value, x.is_sensitive))
            .flatten()
            .collect();
        encoded.append(&mut encoded_headers);

        encoded
    }

    fn encode_header_table_changes(&self) -> Vec<u8> {
        return self.header_table_changes.iter().map(|size| {
            let mut bytes = self.encode_int(size.clone() as u64, 5);
            bytes[0] |= 0x20;
            bytes
        }).flatten().collect()
    }

    fn encode_header(&mut self, name: &str, value: &str, sensitive: bool) -> Vec<u8> {
        if let Some(x) = self.header_table.search_with_name_and_value(name, value) {
            return self.encode_indexed(x);
        }

        if let Some(x) = self.header_table.search_with_name(name) {
            let index_bit = if sensitive {
                Encoder::INDEX_NEVER
            } else {
                Encoder::INDEX_INCREMENTAL
            };

            if !sensitive {
                self.header_table.add(name, value);
            }

            return self.encode_indexed_literal(x as u64, value, index_bit);
        }

        self.encode_literal(name, value)
    }
    
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

    pub(crate) fn encode_indexed(&self, index: usize) -> Vec<u8> {
        let mut bytes = self.encode_int(index as u64, 7);
        bytes[0] |= 0x80;
        return bytes;
    }

    pub(crate) fn encode_indexed_literal(&self, index: u64, value: &str, index_bit: u8) -> Vec<u8> {
        let mut prefix: Vec<u8>;
        if index_bit == Encoder::INDEX_INCREMENTAL {
            prefix = self.encode_int(index, 6);
        } else {
            prefix = self.encode_int(index, 4);
        }

        prefix[0] |= index_bit;
        prefix.append(&mut self.encode_int(value.len() as u64, 7));
        prefix.append(&mut value.as_bytes().to_vec());
        prefix
    }

    pub(crate) fn header_table_size(&self) -> usize {
        self.header_table.max_size
    }

    pub(crate) fn set_header_table_size(&mut self, size: usize) {
        if self.header_table.max_size == size { return; }

        self.header_table_changes.push(size);
        self.header_table.set_max_size(size);
    }
}

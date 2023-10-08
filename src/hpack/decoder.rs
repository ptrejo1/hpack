use crate::hpack::header::Header;
use crate::hpack::header_table::HeaderTable;

pub(crate) struct Decoder{
    pub header_table: HeaderTable
}

impl Decoder {

    pub(crate) fn new() -> Decoder {
        Decoder {header_table: HeaderTable::new_default()}
    }

    pub(crate) fn decode(&mut self, data: &[u8]) -> Result<Vec<Header>, String> {
        let mut headers: Vec<Header> = vec![];
        let mut index: usize = 0;

        while index != data.len() {
            let byte = data[index];

            if byte & 0b1000_0000 == 0b1000_0000 {
                // Indexed Header Field Representation
                let (header, consumed) = match self.decode_indexed(&data[index..data.len()]) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
                headers.push(header);
                index += consumed
            } else if byte & 0b1100_0000 == 0b0100_0000 {
                // Literal Header Field with Incremental Indexing
                let (header, consumed) = match self.decode_literal(&data[index..data.len()], 6) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
                self.header_table.add(header.name.as_str(), header.value.as_str());
                headers.push(header);
                index += consumed;
            } else if byte & 0b1111_0000 == 0b0000_0000 {
                // Literal Header Field without Indexing
                let (header, consumed) = match self.decode_literal(&data[index..data.len()], 4) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
                headers.push(header);
                index += consumed;
            } else if byte & 0b1111_0000 == 0b0001_0000 {
                // Literal Header Field never Indexed
                index += 1;
                let (name, name_consumed) = match self.decode_string(&data[index..data.len()]) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
                index += name_consumed;
                let (value, value_consumed) = match self.decode_string(&data[index..data.len()]) {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
                headers.push(Header{name, value});
                index += value_consumed;
            } else if byte & 0b1110_0000 == 0b0010_0000 {
                // Dynamic Table Size Update
                let (new_size, consumed) = match self.decode_int(&data[index..data.len()], 5) {
                    Ok(x) => x,
                    Err(e) => return Err(e)
                };
                index += consumed;
                self.header_table.set_max_size(new_size as usize);
            } else {
                return Err("Unsupported decode!".to_string());
            }
        }

        Ok(headers)
    }

    pub(crate) fn decode_int(&self, data: &[u8], prefix_bits: u32) -> Result<(u64, usize), String> {
        if data.is_empty() {
            return Err("Decode data is empty!".to_string());
        }

        let max_number = 2_u64.pow(prefix_bits) - 1;
        let mut consumed = 1;
        let mask: u8 = 0b11111111 >> (8 - prefix_bits);
        let mut number = (data[consumed - 1] & mask) as u64;
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
        let (index, mut consumed) = self.decode_int(data, prefix)?;

        let name: String;
        if index == 0 {
            let name_data = &data[consumed..data.len()];
            let name_result = self.decode_string(name_data)?;
            name = name_result.0;
            consumed += name_result.1
        } else {
            let header = self.header_table[index.try_into().unwrap()].clone();
            name = header.name;
        }

        let value_data = &data[consumed..data.len()];
        let value_result = self.decode_string(value_data)?;
        Ok((Header {name, value: value_result.0}, consumed + value_result.1))
    }

    fn decode_string(&self, data: &[u8]) -> Result<(String, usize), String> {
        let (length, consumed) = self.decode_int(data, 7)?;
        let end_idx = consumed + length as usize;
        let bytes = data[consumed..end_idx].to_owned();

        if !bytes.is_empty() && bytes[0] & 128 > 0 {
            return Err("Huffman not supported".to_string());
        }

        let value = String::from_utf8(bytes);
        Ok((value.unwrap(), end_idx))
    }

    pub(crate) fn decode_indexed(&self, data: &[u8]) -> Result<(Header, usize), String> {
        let (index, consumed) = self.decode_int(data, 7)?;
        Ok((self.header_table[index.try_into().unwrap()].clone(), consumed))
    }
}

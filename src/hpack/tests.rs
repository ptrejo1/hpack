#[cfg(test)]
mod encoder_tests {
    use crate::hpack::encoder::Encoder;
    use crate::hpack::header::{EncodableHeader, Header};

    #[test]
    fn test_encode_10_with_5_prefix() {
        let encoder = Encoder::new();
        let encoded = encoder.encode_int(10, 5);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 10);
    }

    #[test]
    fn test_encode_1337_with_5_prefix() {
        let encoder = Encoder::new();
        let encoded = encoder.encode_int(1337, 5);
        assert_eq!(encoded.len(), 3);
        assert_eq!(encoded[0], 31);
        assert_eq!(encoded[1], 154);
        assert_eq!(encoded[2], 10);
    }

    #[test]
    fn test_encode_42_with_8_prefix() {
        let encoder = Encoder::new();
        let encoded = encoder.encode_int(42, 8);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 42);
    }

    #[test]
    fn test_encode_indexed() {
        let mut encoder = Encoder::new();
        let headers = [
            Header{name: ":method".to_string(), value: "GET".to_string()}
        ];
        let encoded = encoder.encode(&headers);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 130);
    }

    #[test]
    fn test_encode_sensitive_literal_no_indexing() {
        let mut encoder = Encoder::new();
        let headers = [
            EncodableHeader {
                name: "foo".to_string(),
                value: "bar".to_string(),
                is_sensitive: true,
            }
        ];
        let encoded = encoder.encode_headers(&headers);
        let expected = vec![
            0u8,
            3,
            102, 111, 111,
            3,
            98, 97, 114
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_literal_indexing() {
        let mut encoder = Encoder::new();
        let path = "/sample/path";
        let headers = [
            Header {name: ":path".to_string(), value: path.to_string()}
        ];
        let encoded = encoder.encode(&headers);
        let mut expected: Vec<u8> = vec![68, path.len().try_into().unwrap()];
        expected.append(&mut path.as_bytes().to_vec());

        assert_eq!(encoded, expected);

        // adds to header table
        let idx = encoder.header_table.search_with_name_and_value(":path", path);
        assert_eq!(idx.unwrap(), 62);
    }

    #[test]
    fn test_encode_literal_no_indexing() {
        let mut encoder = Encoder::new();
        let path = "/sample/path";
        let headers = [
            EncodableHeader {
                name: ":path".to_string(),
                value: path.to_string(),
                is_sensitive: true,
            }
        ];
        let encoded = encoder.encode_headers(&headers);
        let mut expected = vec![20, path.len().try_into().unwrap()];
        expected.append(&mut path.as_bytes().to_vec());

        assert_eq!(encoded, expected);

        // doesn't add to header table
        let idx = encoder.header_table.search_with_name_and_value(":path", path);
        assert!(idx.is_none());
    }
}

#[cfg(test)]
mod decoder_tests {
    use crate::hpack::decoder::Decoder;
    use crate::hpack::header::Header;

    #[test]
    fn test_decode_10_with_5_prefix() {
        let decoder = Decoder::new();
        let (decoded, consumed) = decoder.decode_int(&[10], 5).unwrap();
        assert_eq!(decoded, 10);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_decode_1337_with_5_prefix() {
        let decoder = Decoder::new();
        let (decoded, consumed) = decoder.decode_int(&[31, 154, 10], 5).unwrap();
        assert_eq!(decoded, 1337);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_decode_42_with_8_prefix() {
        let decoder = Decoder::new();
        let (decoded, consumed) = decoder.decode_int(&[42], 8).unwrap();
        assert_eq!(decoded, 42);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_decode_empty() {
        let decoder = Decoder::new();
        let result = decoder.decode_int(&[], 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_header_table_size() {
        let decoder = Decoder::new();
        assert_eq!(decoder.header_table.max_size, 4096);
    }

    #[test]
    fn test_decode_indexed() {
        let mut decoder = Decoder::new();
        let encoded = vec![130];

        let headers = decoder.decode(&encoded).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, ":method");
        assert_eq!(headers[0].value, "GET");
    }

    #[test]
    fn test_decode_literal_without_indexing() {
        let mut decoder = Decoder::new();
        let path = "/sample/path";
        let mut encoded: Vec<u8> = vec![4, path.len().try_into().unwrap()];
        encoded.append(&mut path.as_bytes().to_vec());

        let headers = decoder.decode(&encoded).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, ":path");
        assert_eq!(headers[0].value, path);
    }

    #[test]
    fn test_decode_unindexed_literal_without_indexing() {
        let mut decoder = Decoder::new();
        let encoded: Vec<u8> = vec![
            16,
            8,  // length of key
            112, 97, 115, 115, 119, 111, 114, 100,  // key: password
            6,  // length of value
            115, 101, 99, 114, 101, 116,  // value: secret
        ];

        let headers = decoder.decode(&encoded).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, "password");
        assert_eq!(headers[0].value, "secret");
    }

    #[test]
    fn test_decode_literal_with_indexing() {
        let mut decoder = Decoder::new();
        let path = "/sample/path";
        let mut encoded: Vec<u8> = vec![68, path.len().try_into().unwrap()];
        encoded.append(&mut path.as_bytes().to_vec());

        let headers = decoder.decode(&encoded).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, ":path");
        assert_eq!(headers[0].value, path);
        assert_eq!(decoder.header_table.search_with_name_and_value(":path", path).unwrap(), 62);
    }

    #[test]
    fn test_decode_headers() {
        let mut first_bytes: Vec<u8> = vec![130, 134, 132, 1, 15];
        first_bytes.append(&mut "www.example.com".as_bytes().to_vec());
        let first_headers: Vec<Header> = vec![
            Header {name: ":method".to_string(), value: "GET".to_string()},
            Header {name: ":scheme".to_string(), value: "http".to_string()},
            Header {name: ":path".to_string(), value: "/".to_string()},
            Header {name: ":authority".to_string(), value: "www.example.com".to_string()},
        ];

        let mut second_bytes: Vec<u8> = vec![130, 134, 132, 1, 15];
        second_bytes.append(&mut "www.example.com".as_bytes().to_vec());
        second_bytes.append(&mut vec![15, 9, 8]);
        second_bytes.append(&mut "no-cache".as_bytes().to_vec());
        let second_headers: Vec<Header> = vec![
            Header {name: ":method".to_string(), value: "GET".to_string()},
            Header {name: ":scheme".to_string(), value: "http".to_string()},
            Header {name: ":path".to_string(), value: "/".to_string()},
            Header {name: ":authority".to_string(), value: "www.example.com".to_string()},
            Header {name: "cache-control".to_string(), value: "no-cache".to_string()},
        ];

        let mut third_bytes: Vec<u8> = vec![130, 135, 133, 1, 15];
        third_bytes.append(&mut "www.example.com".as_bytes().to_vec());
        third_bytes.append(&mut vec![64, 10]);
        third_bytes.append(&mut "custom-key".as_bytes().to_vec());
        third_bytes.append(&mut vec![12]);
        third_bytes.append(&mut "custom-value".as_bytes().to_vec());
        let third_headers: Vec<Header> = vec![
            Header {name: ":method".to_string(), value: "GET".to_string()},
            Header {name: ":scheme".to_string(), value: "http".to_string()},
            Header {name: ":path".to_string(), value: "/index.html".to_string()},
            Header {name: ":authority".to_string(), value: "www.example.com".to_string()},
            Header {name: "custom-key".to_string(), value: "custom-value".to_string()},
        ];

        let mut decoder = Decoder::new();
        let first_decoded = decoder.decode(&first_bytes).unwrap();
        let second_decoded = decoder.decode(&second_bytes).unwrap();
        let third_decoded = decoder.decode(&third_bytes).unwrap();

        fn compare(expected: &[Header], actual: &[Header]) {
            assert_eq!(expected.len(), actual.len());

            for (i, header) in actual.iter().enumerate() {
                assert_eq!(header.name, expected[i].name);
                assert_eq!(header.value, expected[i].value);
            }
        }

        compare(&first_headers, &first_decoded);
        assert_eq!(decoder.header_table.search_with_name_and_value("custom-key", "custom-value").unwrap(), 62);
    }

    #[test]
    fn test_decode_updates_max_header_size() {
        let mut decoder = Decoder::new();
        let encoded: Vec<u8> = vec![62];

        let headers = decoder.decode(&encoded).unwrap();
        assert_eq!(headers.len(), 0);
        assert_eq!(decoder.header_table.max_size, 30);
    }

    #[test]
    fn test_decode_no_huffman_support() {
        let mut decoder = Decoder::new();
        let encoded: Vec<u8> = vec![
            130, 134, 132, 1, 140, 241, 227, 194, 229, 242, 58, 107, 160,
            171, 144, 244, 255
        ];

        let headers = decoder.decode(&encoded);
        assert!(headers.is_err());
    }
}

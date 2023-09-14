#[cfg(test)]
mod encoder_tests {
    use crate::hpack::encoder::Encoder;

    #[test]
    fn test_encode_10_with_5_prefix() {
        let encoder = Encoder;
        let encoded = encoder.encode_int(10, 5);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 10);
    }

    #[test]
    fn test_encode_1337_with_5_prefix() {
        let encoder = Encoder;
        let encoded = encoder.encode_int(1337, 5);
        assert_eq!(encoded.len(), 3);
        assert_eq!(encoded[0], 31);
        assert_eq!(encoded[1], 154);
        assert_eq!(encoded[2], 10);
    }

    #[test]
    fn test_encode_42_with_8_prefix() {
        let encoder = Encoder;
        let encoded = encoder.encode_int(42, 8);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0], 42);
    }

    #[test]
    fn test_encode_literal_no_indexing() {
        let encoder = Encoder;
        let encoded = encoder.encode_literal("foo", "bar");
        assert_eq!(encoded, vec![
            0u8,
            3,
            102, 111, 111,
            3,
            98, 97, 114
        ]);
    }

    #[test]
    fn test_encode_indexed() {
        let encoder = Encoder;
        let encoded = encoder.encode_indexed(130);
        assert_eq!(encoded.len(), 2);
        assert_eq!(encoded[0], 0xff);
    }
}

#[cfg(test)]
mod decoder_tests {
    use crate::hpack::decoder::Decoder;

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
    fn test_decode_literal_without_indexing() {
        let decoder = Decoder::new();
        let encoded = vec![
            0u8,
            3,
            102, 111, 111,
            3,
            98, 97, 114
        ];

        let decoded = decoder.decode_literal(&encoded, 4);
        let (decoded_header, consumed) = decoded.unwrap();
        assert_eq!(decoded_header.name, "foo");
        assert_eq!(decoded_header.value, "bar");
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn test_decode_indexed() {
        let decoder = Decoder::new();
        let encoded = vec![2];
        let (header, consumed) = decoder.decode_indexed(&encoded).unwrap();
        assert_eq!(header.name, ":method");
        assert_eq!(header.value, "GET")
    }
}

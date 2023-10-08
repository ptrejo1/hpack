# HPACK
HPACK ([RFC 7541](https://tools.ietf.org/html/rfc7541)) implementation in Rust.
```rust
// encode
let mut encoder = Encoder::new();
let headers = [
    Header {name: ":method".to_string(), value: "GET".to_string()}
];
let encoded = encoder.encode(&headers);

// decode
let mut decoder = Decoder::new();
let headers = decoder.decode(&encoded).unwrap();
```

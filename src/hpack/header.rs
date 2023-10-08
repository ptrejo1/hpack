#[derive(Debug, Clone)]
pub(crate) struct Header {
    pub(crate) name: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EncodableHeader {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) is_sensitive: bool,
}

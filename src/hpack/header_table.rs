use std::ops::Index;
use lazy_static::lazy_static;
use crate::hpack::header::Header;

lazy_static! {
    static ref STATIC_ENTRIES: [Header; 61] = [
        Header {name: ":authority".to_string(), value: "".to_string()},
        Header {name: ":method".to_string(), value: "GET".to_string()},
        Header {name: ":method".to_string(), value: "POST".to_string()},
        Header {name: ":path".to_string(), value: "/".to_string()},
        Header {name: ":path".to_string(), value: "/index.html".to_string()},
        Header {name: ":scheme".to_string(), value: "http".to_string()},
        Header {name: ":scheme".to_string(), value: "https".to_string()},
        Header {name: ":status".to_string(), value: "200".to_string()},
        Header {name: ":status".to_string(), value: "204".to_string()},
        Header {name: ":status".to_string(), value: "206".to_string()},
        Header {name: ":status".to_string(), value: "304".to_string()},
        Header {name: ":status".to_string(), value: "400".to_string()},
        Header {name: ":status".to_string(), value: "404".to_string()},
        Header {name: ":status".to_string(), value: "500".to_string()},
        Header {name: "accept-charset".to_string(), value: "".to_string()},
        Header {name: "accept-encoding".to_string(), value: "gzip, deflate".to_string()},
        Header {name: "accept-language".to_string(), value: "".to_string()},
        Header {name: "accept-ranges".to_string(), value: "".to_string()},
        Header {name: "accept".to_string(), value: "".to_string()},
        Header {name: "access-control-allow-origin".to_string(), value: "".to_string()},
        Header {name: "age".to_string(), value: "".to_string()},
        Header {name: "allow".to_string(), value: "".to_string()},
        Header {name: "authorization".to_string(), value: "".to_string()},
        Header {name: "cache-control".to_string(), value: "".to_string()},
        Header {name: "content-disposition".to_string(), value: "".to_string()},
        Header {name: "content-encoding".to_string(), value: "".to_string()},
        Header {name: "content-language".to_string(), value: "".to_string()},
        Header {name: "content-length".to_string(), value: "".to_string()},
        Header {name: "content-location".to_string(), value: "".to_string()},
        Header {name: "content-range".to_string(), value: "".to_string()},
        Header {name: "content-type".to_string(), value: "".to_string()},
        Header {name: "cookie".to_string(), value: "".to_string()},
        Header {name: "date".to_string(), value: "".to_string()},
        Header {name: "etag".to_string(), value: "".to_string()},
        Header {name: "expect".to_string(), value: "".to_string()},
        Header {name: "expires".to_string(), value: "".to_string()},
        Header {name: "from".to_string(), value: "".to_string()},
        Header {name: "host".to_string(), value: "".to_string()},
        Header {name: "if-match".to_string(), value: "".to_string()},
        Header {name: "if-modified-since".to_string(), value: "".to_string()},
        Header {name: "if-none-match".to_string(), value: "".to_string()},
        Header {name: "if-range".to_string(), value: "".to_string()},
        Header {name: "if-unmodified-since".to_string(), value: "".to_string()},
        Header {name: "last-modified".to_string(), value: "".to_string()},
        Header {name: "link".to_string(), value: "".to_string()},
        Header {name: "location".to_string(), value: "".to_string()},
        Header {name: "max-forwards".to_string(), value: "".to_string()},
        Header {name: "proxy-authenticate".to_string(), value: "".to_string()},
        Header {name: "proxy-authorization".to_string(), value: "".to_string()},
        Header {name: "range".to_string(), value: "".to_string()},
        Header {name: "referer".to_string(), value: "".to_string()},
        Header {name: "refresh".to_string(), value: "".to_string()},
        Header {name: "retry-after".to_string(), value: "".to_string()},
        Header {name: "server".to_string(), value: "".to_string()},
        Header {name: "set-cookie".to_string(), value: "".to_string()},
        Header {name: "strict-transport-security".to_string(), value: "".to_string()},
        Header {name: "transfer-encoding".to_string(), value: "".to_string()},
        Header {name: "user-agent".to_string(), value: "".to_string()},
        Header {name: "vary".to_string(), value: "".to_string()},
        Header {name: "via".to_string(), value: "".to_string()},
        Header {name: "www-authenticate".to_string(), value: "".to_string()},
    ];
}

pub(crate) struct HeaderTable {
    dynamic_entries: Vec<Header>,
    pub max_size: usize,
}

impl HeaderTable {

    pub(crate) fn new(max_size: usize) -> HeaderTable {
        HeaderTable{
            dynamic_entries: Vec::new(),
            max_size
        }
    }

    pub(crate) fn new_default() -> HeaderTable {
        HeaderTable{
            dynamic_entries: Vec::new(),
            max_size: 4096
        }
    }

    pub(crate) fn search_with_name(&self, name: &str) -> Option<usize> {
        for (i, header) in STATIC_ENTRIES.iter().enumerate() {
            if header.name != name { continue; }
            return Some(i + 1);
        }

        for (i, header) in self.dynamic_entries.iter().enumerate() {
            if header.name != name { continue; }
            return Some(STATIC_ENTRIES.len() + i + 1);
        }

        None
    }

    pub(crate) fn search_with_name_and_value(&self, name: &str, value: &str) -> Option<usize> {
        for (i, header) in STATIC_ENTRIES.iter().enumerate() {
            if header.name != name || header.value != value { continue; }
            return Some(i + 1);
        }

        for (i, header) in self.dynamic_entries.iter().enumerate() {
            if  header.name != name || header.value != value { continue; }
            return Some(STATIC_ENTRIES.len() + i + 1);
        }

        None
    }

    pub(crate) fn add(&mut self, name: &str, value: &str) {
        self.dynamic_entries.push(Header{name: name.to_string(), value: value.to_string()});
    }

    pub(crate) fn set_max_size(&mut self, size: usize) {
        if size == 0 {
            self.dynamic_entries.clear()
        } else if self.dynamic_entries.len() > size {
            self.dynamic_entries = self.dynamic_entries[0..size].to_owned();
        }

        self.max_size = size;
    }
}

impl Index<usize> for HeaderTable {
    type Output = Header;

    fn index(&self, index: usize) -> &Self::Output {
        if index <= STATIC_ENTRIES.len() {
            return &STATIC_ENTRIES[index - 1];
        }

        &self.dynamic_entries[index - STATIC_ENTRIES.len() - 1]
    }
}

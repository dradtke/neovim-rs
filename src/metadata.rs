use mpack::Value;

pub struct Metadata {
    buffer_id: i64,
    window_id: i64,
    tabpage_id: i64,
}

pub enum InvalidMetadata {
    NotAMap,
    NoTypeInformation,
}

impl Metadata {
    pub fn new(val: Value) -> Result<Metadata> {
        let metadata = match val {
            Value::Map(map) => map,
            _ => return Err(InvalidMetadata::NotAMap),
        };
        let types = match metadata.get("types") {
            Some(types) => types,
            None => return Err(InvalidMetadata::NoTypeInformation),
        };

        let buffer_id = types.get("Buffer").unwrap().get("id").unwrap().int();
        let window_id = types.get("Window").unwrap().get("id").unwrap().int();
        let tabpage_id = types.get("Tabpage").unwrap().get("id").unwrap().int();

        Metadata {
            buffer_id = buffer_id,
            window_id = window_id,
            tabpage_id = tabpage_id,
        }
    }
}

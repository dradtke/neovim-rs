use mpack::{MapGetError, Value};

pub struct Metadata {
    pub buffer_id: i64,
    pub window_id: i64,
    pub tabpage_id: i64,
}

pub enum GetMetadataError {
    NotAMap,
    NoTypeInformation,
    Missing(&'static str),
    Invalid(&'static str),
}

impl Metadata {
    /// Attempt to read metadata information from the provided value.
    ///
    /// This method expects the value to represent this type of data structure:
    ///
    /// ```
    /// {
    ///     "types": {
    ///         "Buffer":  { "id": <int> },
    ///         "Window":  { "id": <int> },
    ///         "Tabpage": { "id": <int> }
    ///     }
    /// }
    ///
    /// It then pulls out the id values and stores them in the returned `Metadata` struct
    /// so that buffer, window, and tabpage values received from Neovim can be parsed
    /// appropriately.
    ///
    /// # Errors
    ///
    /// 1. `NotAMap`: the passed-in value doesn't represent a map.
    /// 2. `NoTypeInformation`: the "types" object couldn't be found.
    /// 3. `Missing(name)`: the id value indicated by `name` wasn't found.
    /// 4. `Invalid(name)`: the id value indicated by `name` was found, but couldn't be parsed as an int.
    /// ```
    pub fn new(metadata: Value) -> Result<Metadata, GetMetadataError> {
        let types = match metadata.get("types") {
            Ok(types) => types,
            Err(MapGetError::NotAMap) => return Err(GetMetadataError::NotAMap),
            Err(MapGetError::NotFound(..)) => return Err(GetMetadataError::NoTypeInformation),
        };

        fn get_id(types: &Value, path: Vec<&'static str>, name: &'static str) -> Result<i64, GetMetadataError> {
            match types.deep_get(path) {
                Ok(id) => match id.int() {
                    Some(x) => Ok(x),
                    None => Err(GetMetadataError::Invalid(name)),
                },
                Err(..) => Err(GetMetadataError::Missing(name)),
            }
        }

        Ok(Metadata {
            buffer_id: try!(get_id(types, vec!["Buffer", "id"], "Buffer.id")),
            window_id: try!(get_id(types, vec!["Window", "id"], "Window.id")),
            tabpage_id: try!(get_id(types, vec!["Tabpage", "id"], "Tabpage.id")),
        })
    }
}

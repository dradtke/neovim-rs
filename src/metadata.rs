use mpack::{self, Value, ValueMap};
use std::convert;
use std::error;
use std::fmt;

pub struct Metadata {
    pub buffer_id: i64,
    pub window_id: i64,
    pub tabpage_id: i64,
}

#[derive(Debug)]
pub enum GetMetadataError {
    NotAMap,
    NoTypeInformation,
    Invalid(String),
    Missing(String),
    ReadError(mpack::ReadError),
}

impl fmt::Display for GetMetadataError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self)
    }
}

impl error::Error for GetMetadataError {
    fn description(&self) -> &str {
        match *self {
            GetMetadataError::NotAMap => "not a map",
            GetMetadataError::NoTypeInformation => "no type information",
            GetMetadataError::Invalid(_) => "invalid id",
            GetMetadataError::Missing(_) => "missing id",
            GetMetadataError::ReadError(_) => "read error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            GetMetadataError::ReadError(ref e) => Some(e as &error::Error),
            _ => None,
        }
    }
}

impl convert::From<mpack::ReadError> for GetMetadataError {
    fn from(err: mpack::ReadError) -> GetMetadataError {
        GetMetadataError::ReadError(err)
    }
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
        let metadata = match metadata.map() {
            Ok(m) => m,
            Err(_) => return Err(GetMetadataError::NotAMap),
        };

        let types = match metadata.get("types") {
            Some(t) => t.clone().map().unwrap(),
            None => return Err(GetMetadataError::NoTypeInformation),
        };

        fn get_id(types: &ValueMap, name: &'static str) -> Result<i64, GetMetadataError> {
            let ob = match types.get(name) {
                Some(v) => match v.clone().map() {
                    Ok(ob) => ob,
                    Err(_) => return Err(GetMetadataError::Missing(format!("{}.id", name))),
                },
                None => return Err(GetMetadataError::Missing(format!("{}.id", name))),
            };

            match ob.get("id") {
                Some(id) => Ok(id.clone().int().unwrap()),
                None => return Err(GetMetadataError::Invalid(format!("{}.id", name))),
            }
        }

        Ok(Metadata {
            buffer_id: try!(get_id(&types, "Buffer")),
            window_id: try!(get_id(&types, "Window")),
            tabpage_id: try!(get_id(&types, "Tabpage")),
        })
    }
}

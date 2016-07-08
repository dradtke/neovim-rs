#![allow(dead_code, unused_variables)]

extern crate mpack;

use std::env;
use std::fmt;
use std::io;
use std::process::Command;

pub use self::metadata::Metadata;
pub use self::session::Session;

mod metadata;
mod session;

/// A function as parsed from `get_api_info()`.
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, String)>,
    pub return_type: String,
    pub async: bool,
    pub can_fail: bool,
}

impl fmt::Display for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{name}({params}) -> {return_type} {can_fail}{async}",
            return_type=self.return_type,
            async=if self.async { "async" } else { "" },
            name=self.name,
            params=self.parameters.iter().map(|p| format!("{} {}", p.0, p.1)).collect::<Vec<String>>().join(", "),
            can_fail=if self.can_fail { "[can fail]" } else { "" },
        )
    }
}

/// The result of `get_api_info()`.
pub struct ApiInfo {
    pub functions: Vec<Function>,
}

/// Get API information from Vim by running `nvim --api-info` and parsing the output.
pub fn get_api_info() -> Result<ApiInfo, mpack::ReadError> {
    let cmd = env::var("NVIM_BIN").unwrap_or(String::from("nvim"));
    let output = try!(Command::new(cmd).arg("--api-info").output());
    if !output.status.success() {
        return Err(mpack::ReadError::Io(match output.status.code() {
            Some(code) => io::Error::from_raw_os_error(code),
            None => io::Error::new(io::ErrorKind::Other, "killed by signal"),
        }))
    }

    let mut r = mpack::Reader::new(&output.stdout[..]);
    let dict = try!(r.read_value()).map().unwrap();
    let dict_functions = dict.get_array("functions").unwrap();

    let mut functions = Vec::with_capacity(dict_functions.len());

    for f in dict_functions {
        let f = f.map().unwrap();
        let name = f.get_string("name").unwrap();
        let parameters = f.get_array("parameters").unwrap().into_iter().map(|p| {
            let p = p.array().unwrap();
            (p[0].clone().string().unwrap(), p[1].clone().string().unwrap())
        }).collect();
        let return_type = f.get_string("return_type").unwrap();
        let async = f.get_bool("async").unwrap();
        let can_fail = f.get_bool("can_fail").unwrap_or(false);
        functions.push(Function{
            name: name,
            parameters: parameters,
            return_type: return_type,
            async: async,
            can_fail: can_fail,
        });
    }

    Ok(ApiInfo{
        functions: functions,
    })
}

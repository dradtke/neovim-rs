//! Example program that spawns a child Neovim instance and uses the Msgpack-RPC
//! protocol directly to request the current version number.

extern crate mpack;
extern crate neovim;

fn main() {
    // Open up a Neovim session by spawning a new instance of it.
    let mut s = neovim::Session::new_child(&[]).unwrap();

    let vim_version = s.call_sync(String::from("vim_get_vvar"), vec![mpack::Value::String(String::from("version"))]).unwrap();
    match vim_version {
        Ok(x) => println!("Neovim Version: {}", x.uint().unwrap()),
        Err(err) => println!("Unexpected error: {:?}", err),
    }
}

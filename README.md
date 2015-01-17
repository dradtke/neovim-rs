# neovim
Support for writing Neovim plugins in Rust.

This crate doesn't actually do anything yet except open up a session that can communicate over TCP, Unix socket, stdio, or a child process, but in the near future it will provide full support for communicating with Neovim.

There's a lot that needs to stabilize before this will become possible, including Neovim's plugin API, Rust itself, and the `msgpack` crate which will be used for serialization.

Development will attempt to mimick that of the [Python client](https://github.com/neovim/python-client).

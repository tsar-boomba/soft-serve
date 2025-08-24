# sfs - Soft Serve

A very simple file server, named after my favorite kind of ice cream :icecream:

## Features

- HTTP/1 and HTTP/2
- FTP
- TFTP
- As simple as it gets

## Installation

With [`cargo-binstall`]([http://binstall](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file))

```bash
cargo binstall soft-serve
```

or `cargo install`

```bash
cargo install soft-serve --locked
```

## Usage

Serves the current directory over HTTP

```bash
sfs
```

Serves the `dist` directory over HTTP

```bash
sfs dist
```

Serves `files` over FTP

```bash
sfs ftp files
```

Serves `files` over TFTP

```bash
sfs ftp files --trivial
```

or

```bash
sfs ftp files -t
```

### Options

#### `--port` or `-p`

Set the port the server lsitens on. Defaults to `5001` for HTTP and `5002` for FTP and TFTP.

#### `--ip` or `-i`

Set the IP address of the server. Defaults to `127.0.0.1`

#### `--no-index-convenience` (HTTP only)

Turns off the convenience functionality of `/` being treated as `/index.html`.

#### `--trivial` or `-t` (FTP only)

Serve files over TFTP instead of FTP.

### As Library

```toml
soft-serve = { version = "0.0.9", no-default-features = true, features = ["http", "ftp"] }
```

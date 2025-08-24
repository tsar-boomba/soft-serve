#!/bin/sh

set -e
cargo build --all-features
cargo release --tag-name 'v{{version}}' -v $@

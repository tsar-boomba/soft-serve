#!/bin/sh

cargo release --tag-name 'v{{version}}' -v $@

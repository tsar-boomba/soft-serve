#!/bin/sh

cargo release --allow-branch main --tag-name 'v{{version}}' -v $@

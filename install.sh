#!/bin/bash

cd "$(dirname "$0")"

cargo install --path .

echo "vcs installed successfully."

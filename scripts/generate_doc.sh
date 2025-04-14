#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1

pushd ..

pushd code

cargo +nightly rustdoc -- --output-format json -Z unstable-options

popd # code

cargo run --bin doc-generator -- ./target/doc/code.json ./target/doc/index.d.ts

cp ./target/doc/index.d.ts ./tests/src/

popd

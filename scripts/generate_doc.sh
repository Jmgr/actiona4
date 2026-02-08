#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1

pushd ..

pushd actiona-ng

cargo +nightly rustdoc -- --output-format json -Z unstable-options

popd # actiona-ng

cargo run --bin doc-generator -- ./target/doc/actiona_ng.json ./target/doc/index.d.ts

cp ./target/doc/index.d.ts ./tests/src/
cp ./target/doc/index.d.ts ./actiona-ng-cli/assets/

popd

#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1

pushd ..

pushd core

cargo +nightly rustdoc -- --output-format json -Z unstable-options

popd # core

cargo run --bin doc-generator -- ./target/doc/actiona_core.json ./target/doc/index.d.ts
cargo run --bin doc-generator -- --no-globals ./target/doc/actiona_core.json ./target/doc/index.noglobals.d.ts

cp ./target/doc/index.d.ts ./tests/src/
cp ./target/doc/index.d.ts ./run/assets/
cp ./target/doc/index.noglobals.d.ts ./tests/src/
cp ./target/doc/index.noglobals.d.ts ./run/assets/

popd

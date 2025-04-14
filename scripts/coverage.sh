#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1

pushd ..

cargo tarpaulin --out Lcov

popd

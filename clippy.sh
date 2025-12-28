#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x
git add -u
git commit -m. || true
cargo +nightly clippy --tests --all-targets --all-features --fix -Z unstable-options -- -A clippy::uninit_assumed_init

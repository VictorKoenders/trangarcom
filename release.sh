#/bin/bash
set -e
cargo +nightly build --release
docker build -t trangarcom .
docker save trangarcom --output trangarcom.tar


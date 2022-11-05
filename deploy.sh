#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pizw2
readonly TARGET_PATH=/home/pi/bin/iot-central
readonly SOURCE_PATH=./target/release/iot-central

docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:bullseye cargo build --release --no-default-features --features rpi
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} sudo systemctl restart iot-central.service

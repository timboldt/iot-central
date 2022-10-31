#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pizw2
readonly TARGET_PATH=/home/pi/iot-central
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/iot-central

cargo build --release --target=${TARGET_ARCH}
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}
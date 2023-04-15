#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pizw2
readonly TARGET_PATH=/home/pi/bin/

env GOOS=linux GOARCH=arm64 go build -o build/iot-central-finance cmd/finance/main.go
env GOOS=linux GOARCH=arm64 go build -o build/iot-central-sensors cmd/sensors/main.go
env GOOS=linux GOARCH=arm64 go build -o build/iot-central-weather cmd/weather/main.go

rsync build/iot-central-finance ${TARGET_HOST}:${TARGET_PATH}
rsync build/iot-central-sensors ${TARGET_HOST}:${TARGET_PATH}
rsync build/iot-central-weather ${TARGET_HOST}:${TARGET_PATH}

ssh -t ${TARGET_HOST} sudo systemctl restart iot-central-finance.service
#ssh -t ${TARGET_HOST} sudo systemctl restart iot-central-sensors.service
ssh -t ${TARGET_HOST} sudo systemctl restart iot-central-weather.service


package main

import (
	"os"
	"strconv"
	"sync"

	"github.com/timboldt/iot-central/adafruitio"
	"github.com/timboldt/iot-central/finance"
	"github.com/timboldt/iot-central/weather"
)

func parseFloatOrDefault(s string) float64 {
	f, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return 0
	}
	return f
}

func main() {
	var wg sync.WaitGroup
	doneChan := make(chan bool)
	aioChan := make(chan adafruitio.Metric)

	// NOTE: Don't add the Adafruit IO Sender to the wait group yet.
	go adafruitio.Sender(adafruitio.Params{
		Username: os.Getenv("IO_USERNAME"),
		Key:      os.Getenv("IO_KEY"),
		AIOChan:  aioChan,
		WG:       &wg,
	})

	wg.Add(1)
	go finance.Fetcher(finance.Params{
		APIKey:   os.Getenv("FINHUB_API_KEY"),
		AIOChan:  aioChan,
		DoneChan: doneChan,
		WG:       &wg,
	})

	wg.Add(1)
	go weather.Fetcher(weather.Params{
		APIKey:    os.Getenv("OPEN_WEATHER_KEY"),
		Latitude:  parseFloatOrDefault(os.Getenv("OPEN_WEATHER_LAT")),
		Longitude: parseFloatOrDefault(os.Getenv("OPEN_WEATHER_LON")),
		AIOChan:   aioChan,
		DoneChan:  doneChan,
		WG:        &wg,
	})

	// Tell everyone except Adafruit IO Sender to shut down.
	close(doneChan)
	wg.Wait()

	// Now tell the Adafruit IO Sender to shut down.
	wg.Add(1)
	close(aioChan)
	wg.Wait()
}

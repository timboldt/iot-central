package weather

import (
	"fmt"
	"log"
	"sync"
	"time"

	owm "github.com/briandowns/openweathermap"
	"github.com/timboldt/iot-central/adafruitio"
)

type Params struct {
	APIKey    string
	Latitude  float64
	Longitude float64
	AIOChan   chan adafruitio.Metric
	DoneChan  chan bool
	WG        *sync.WaitGroup
}

func Fetcher(params Params) {
	ticker := time.NewTicker(10 * time.Minute)

	log.Println("OpenWeather fetcher starting...")
	processWeather(&params)
	for {
		select {
		case <-params.DoneChan:
			log.Println("OpenWeather fetcher shutting down...")
			ticker.Stop()
			params.WG.Done()
			return
		case <-ticker.C:
			processWeather(&params)
		}
	}
}

func processWeather(params *Params) {
	w, err := owm.NewOneCall("F", "EN", params.APIKey, []string{owm.ExcludeDaily, owm.ExcludeMinutely})
	if err != nil {
		log.Printf("Error setting up OpenWeather OneCall config: %v", err)
		return
	}

	err = w.OneCallByCoordinates(
		&owm.Coordinates{
			Latitude:  params.Latitude,
			Longitude: params.Longitude,
		},
	)
	if err != nil {
		log.Printf("Error getting OpenWeather OneCall: %v", err)
		return
	}

	temperature := w.Current.Temp
	humidity := w.Current.Humidity
	// Convert hPa to inHg.
	pressure := float32(w.Current.Pressure) / 33.863886666667
	log.Printf("Weather: %.1f'F %d%% %.2finHg\n", temperature, humidity, pressure)
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.temp",
		Value: fmt.Sprintf("%f", temperature),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.humidity",
		Value: fmt.Sprintf("%d", humidity),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.pressure",
		Value: fmt.Sprintf("%f", pressure),
	}
}

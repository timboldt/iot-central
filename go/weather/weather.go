package weather

import (
	"fmt"
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
	timer := time.NewTimer(10 * time.Minute)

	fmt.Println("OpenWeather fetcher starting...")
	processWeather(&params)
	for {
		select {
		case <-params.DoneChan:
			fmt.Println("OpenWeather fetcher shutting down...")
			params.WG.Done()
			return
		case <-timer.C:
			processWeather(&params)
		}
	}
}

func processWeather(params *Params) {
	w, err := owm.NewOneCall("C", "EN", params.APIKey, []string{owm.ExcludeDaily, owm.ExcludeMinutely})
	if err != nil {
		fmt.Printf("Error setting up OpenWeather OneCall config: %v", err)
		return
	}

	err = w.OneCallByCoordinates(
		&owm.Coordinates{
			Latitude:  params.Latitude,
			Longitude: params.Longitude,
		},
	)
	if err != nil {
		fmt.Printf("Error getting OpenWeather OneCall: %v", err)
		return
	}

	//fmt.Printf("Weather: %v\n", w)
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.temp",
		Value: fmt.Sprintf("%f", w.Current.Temp),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.humidity",
		Value: fmt.Sprintf("%d", w.Current.Humidity),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "weather.pressure",
		Value: fmt.Sprintf("%d", w.Current.Pressure),
	}
}

package main

import (
	"fmt"
	"os"

	"github.com/timboldt/iot-central/finance"
)

// func parseFloatOrDefault(s string) float64 {
// 	f, err := strconv.ParseFloat(s, 64)
// 	if err != nil {
// 		return 0
// 	}
// 	return f
// }

func main() {
	// w, err := weather.Get(weather.Params{
	// 	APIKey:    os.Getenv("OPEN_WEATHER_KEY"),
	// 	Latitude:  parseFloatOrDefault(os.Getenv("OPEN_WEATHER_LAT")),
	// 	Longitude: parseFloatOrDefault(os.Getenv("OPEN_WEATHER_LON")),
	// })
	// if err != nil {
	// 	fmt.Printf("Failed to get weather: %v\n", err)
	// } else {
	// 	fmt.Println(w)
	// }

	quote, err := finance.Get(finance.Params{
		APIKey: os.Getenv("FINHUB_API_KEY"),
	})
	if err != nil {
		fmt.Printf("Failed to get finance info: %v\n", err)
	} else {
		fmt.Printf("%+v\n", quote)
	}
}

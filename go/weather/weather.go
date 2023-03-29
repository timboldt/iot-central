package weather

import (
	owm "github.com/briandowns/openweathermap"
)

type Params struct {
	APIKey    string
	Latitude  float64
	Longitude float64
}

func Get(params Params) (*owm.OneCallData, error) {
	w, err := owm.NewOneCall("C", "EN", params.APIKey, []string{owm.ExcludeDaily, owm.ExcludeMinutely})
	if err != nil {
		return nil, err
	}

	err = w.OneCallByCoordinates(
		&owm.Coordinates{
			Latitude:  params.Latitude,
			Longitude: params.Longitude,
		},
	)
	if err != nil {
		return nil, err
	}

	return w, nil
}

package adafruitio

import (
	"fmt"

	aio "github.com/adafruit/io-client-go/v2"
)

type Params struct {
	Username string
	Key      string
}

func Send(params Params) {
	client := aio.NewClient(params.Username, params.Key)
	feeds, _, err := client.Feed.All()
	if err != nil {
		fmt.Printf("Failed to send: %v\n", err)
	} else {
		for _, f := range feeds {
			fmt.Println(f.Name)
		}
	}
}

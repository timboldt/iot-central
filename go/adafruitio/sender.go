package adafruitio

import (
	"fmt"
	"sync"

	aio "github.com/adafruit/io-client-go/v2"
)

type Metric struct {
	Feed  string
	Value string
}

type Params struct {
	Username string
	Key      string
	AIOChan  chan Metric
	WG       *sync.WaitGroup
}

func Sender(params Params) {
	fmt.Println("Adafruit IO sender starting...")
	client := aio.NewClient(params.Username, params.Key)
	for {
		m := <-params.AIOChan
		client.SetFeed(&aio.Feed{Key: m.Feed})
		if m.Feed == "" {
			fmt.Println("Adafruit IO sender shutting down...")
			params.WG.Done()
			return
		}
		if _, _, err := client.Data.Create(&aio.Data{Value: m.Value}); err != nil {
			fmt.Printf("Error sending %+v: %v", m, err)
		}
	}
}

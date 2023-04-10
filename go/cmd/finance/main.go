package main

import (
	"os"
	"os/signal"
	"sync"

	"github.com/timboldt/iot-central/adafruitio"
	"github.com/timboldt/iot-central/finance"
)

func main() {
	var wg sync.WaitGroup
	doneChan := make(chan bool)
	aioChan := make(chan adafruitio.Metric)

	// NOTE: We don't add the Adafruit IO Sender to the wait group yet.
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

	// Block until signal.
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt)
	<-sig

	// Tell everyone except Adafruit IO Sender to shut down.
	close(doneChan)
	wg.Wait()

	// Now tell the Adafruit IO Sender to shut down.
	wg.Add(1)
	close(aioChan)
	wg.Wait()
}

package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"sync"

	"github.com/timboldt/iot-central/adafruitio"
	"github.com/timboldt/iot-central/sensors"
	"periph.io/x/conn/v3/i2c/i2creg"
	"periph.io/x/host/v3"
)

func main() {
	// Load all the drivers:
	if _, err := host.Init(); err != nil {
		log.Fatal(err)
	}

	// Open a handle to the first available I²C bus:
	bus, err := i2creg.Open("/dev/i2c-1")
	if err != nil {
		log.Fatal(err)
	}
	defer bus.Close()

	var wg sync.WaitGroup
	doneChan := make(chan bool)
	aioChan := make(chan adafruitio.Metric)

	// NOTE: We don't add the Adafruit IO Sender to the wait group yet.
	// go adafruitio.Sender(adafruitio.Params{
	// 	Username: os.Getenv("IO_USERNAME"),
	// 	Key:      os.Getenv("IO_KEY"),
	// 	AIOChan:  aioChan,
	// 	WG:       &wg,
	// })

	wg.Add(1)
	go sensors.BME280Fetcher(sensors.BME280Params{
		Bus:      bus,
		AIOChan:  aioChan,
		DoneChan: doneChan,
		WG:       &wg,
	})

	for v := range aioChan {
		fmt.Printf("AIO: %+v\n", v)
	}

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

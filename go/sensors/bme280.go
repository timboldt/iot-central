package sensors

import (
	"fmt"
	"log"
	"math"
	"sync"
	"time"

	"github.com/timboldt/iot-central/adafruitio"
	"periph.io/x/conn/v3/i2c"
	"periph.io/x/conn/v3/physic"
	"periph.io/x/devices/v3/bmxx80"
)

type BME280Params struct {
	Bus      i2c.Bus
	AIOChan  chan adafruitio.Metric
	DoneChan chan bool
	WG       *sync.WaitGroup
}

func BME280Fetcher(params BME280Params) {
	ticker := time.NewTicker(1 * time.Minute)

	log.Println("BME280 fetcher starting...")
	processBME280(&params)
	for {
		select {
		case <-params.DoneChan:
			log.Println("BME280 fetcher shutting down...")
			ticker.Stop()
			params.WG.Done()
			return
		case <-ticker.C:
			processBME280(&params)
		}
	}
}

func processBME280(params *BME280Params) {
	// Open a handle to a bme280/bmp280 connected on the I²C bus using default
	// settings:
	dev, err := bmxx80.NewI2C(params.Bus, 0x77, &bmxx80.DefaultOpts)
	if err != nil {
		log.Printf("Failed to connect to BME280: %v\n", err)
		return
	}
	defer dev.Halt()

	// Read temperature from the sensor:
	var env physic.Env
	if err = dev.Sense(&env); err != nil {
		log.Printf("Failed to read from BME280: %v\n", err)
		return
	}
	log.Printf("BME280: %8s %10s %9s\n", env.Temperature, env.Pressure, env.Humidity)

	celsius := env.Temperature.Celsius()
	relHum := float64(env.Humidity) / 1e5
	hPa := float64(env.Pressure) / 1e11
	absHum := relativeHumidityToAbsolute(relHum, celsius)
	seaLevelPressure := hPaToinHg(rawPressureToSeaLevel(hPa, celsius))

	// Raw values.
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr-bme280.temperature",
		Value: fmt.Sprintf("%f", celsius),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr-bme280.humidity",
		Value: fmt.Sprintf("%f", relHum),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr-bme280.pressure",
		Value: fmt.Sprintf("%f", hPa),
	}

	// Computed values.
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr.temperature",
		Value: fmt.Sprintf("%f", env.Temperature.Fahrenheit()),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr.humidity",
		Value: fmt.Sprintf("%f", relHum),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr.abs-humidity",
		Value: fmt.Sprintf("%f", absHum),
	}
	params.AIOChan <- adafruitio.Metric{
		Feed:  "mbr.pressure",
		Value: fmt.Sprintf("%f", seaLevelPressure),
	}
}

func celsiusToKelvin(celsius float64) float64 {
	return celsius + 273.15
}

func rawPressureToSeaLevel(rawHPa float64, celsius float64) float64 {
	// Local altitude in meters.
	const ALTITUDE = 100.0
	return rawHPa * math.Pow(1.0-0.0065*ALTITUDE/(0.0065+celsiusToKelvin(celsius)), -5.257)
}

func hPaToinHg(hPa float64) float64 {
	return hPa / 33.863_888
}

// https://sensirion.com/media/documents/984E0DD5/61644B8B/Sensirion_Gas_Sensors_Datasheet_SGP30.pdf
// relative_humidity should be a percentage value between 0 and 100.
// Output is in grams per cubic meter.
func relativeHumidityToAbsolute(relHumidity float64, celsius float64) float64 {
	saturatingPressure := 6.112 * math.Exp(17.62*celsius/(243.12+celsius))
	pressure := saturatingPressure * relHumidity / 100.0
	return 216.7 * pressure / celsiusToKelvin(celsius)
}

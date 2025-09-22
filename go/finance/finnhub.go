package finance

import (
	"context"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	finnhub "github.com/Finnhub-Stock-API/finnhub-go"
	"github.com/timboldt/iot-central/adafruitio"
)

type Params struct {
	APIKey   string
	AIOChan  chan adafruitio.Metric
	DoneChan chan bool
	WG       *sync.WaitGroup
}

func Fetcher(params Params) {
	symbols := []string{
		//"DIA",
		//"BITFINEX:USTUSD",
		"COINBASE:BTC-USD",
		"KRAKEN:USDTZUSD",
		"QQQ",
		"IONQ",
		"TSLA",
	}

	cfg := finnhub.NewConfiguration()
	cfg.AddDefaultHeader("X-Finnhub-Token", params.APIKey)
	ticker := time.NewTicker(10 * time.Minute)

	log.Println("Finnhub fetcher starting...")
	processQuotes(cfg, params.AIOChan, symbols)
	for {
		select {
		case <-params.DoneChan:
			log.Println("Finnhub fetcher shutting down...")
			ticker.Stop()
			params.WG.Done()
			return
		case <-ticker.C:
			processQuotes(cfg, params.AIOChan, symbols)
		}
	}
}

func processQuotes(cfg *finnhub.Configuration, ch chan adafruitio.Metric, symbols []string) {
	client := finnhub.NewAPIClient(cfg).DefaultApi
	for _, symbol := range symbols {
		quote, _, err := client.Quote(context.Background(), symbol)
		if err != nil {
			log.Printf("Error getting quote for %q: %v", symbol, err)
		} else if quote.C == 0.0 {
			log.Printf("Ignoring zero-valued quote for %q", symbol)
		} else {
			log.Printf("Quote for %q: %v\n", symbol, quote.C)
			ch <- adafruitio.Metric{
				Feed:  "finance." + strings.ReplaceAll(strings.ToLower(symbol), ":", "-"),
				Value: fmt.Sprintf("%f", quote.C),
			}
		}
	}
}

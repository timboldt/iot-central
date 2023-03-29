package finance

import (
	"context"

	finnhub "github.com/Finnhub-Stock-API/finnhub-go"
)

type Params struct {
	APIKey string
}

func Get(params Params) (finnhub.Quote, error) {
	cfg := finnhub.NewConfiguration()
	cfg.AddDefaultHeader("X-Finnhub-Token", params.APIKey)
	finnhubClient := finnhub.NewAPIClient(cfg).DefaultApi
	quote, _, err := finnhubClient.Quote(context.Background(), "AAPL")
	return quote, err
}

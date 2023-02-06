import asyncio

from pyowm import OWM


class Fetcher:
    def __init__(self, sender: object, open_weather_key: str, lat: float, lon: float) -> None:
        self._UPDATE_PERIOD = 10.0 * 60.0
        self._sender = sender
        self._api = OWM(open_weather_key)
        self.lat = lat
        self.lon = lon

    async def run(self):
        print("Weather Fetcher started")
        mgr = self._api.weather_manager()
        while True:
            print("Weather Fetcher requesting data")
            res = mgr.one_call(lat=self.lat, lon=self.lon)
            print("Weather Fetcher got data")
            print("Weather:", res.current.temperature("celsius"),
                  res.current.humidity, res.current.barometric_pressure("hPa"))
            try:
                await self._sender.send("weather.temp", res.current.temperature("celsius")["temp"])
                await self._sender.send("weather.humidity", res.current.humidity)
                await self._sender.send("weather.pressure", res.current.barometric_pressure("hPa")["press"])
                await asyncio.sleep(self._UPDATE_PERIOD)
            except asyncio.exceptions.CancelledError:
                print("Weather Fetcher stopped")
                return

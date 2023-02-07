"""The weather module queries Open Weather and queues the weather telemetry for publishing."""
import asyncio

import pyowm
from pyowm.commons.exceptions import PyOWMError
from pyowm.weatherapi25.one_call import OneCall


class Fetcher:
    """The Weather Fetcher queries Open Weather and queues the weather telemetry for publishing."""

    def __init__(self, sender: object, open_weather_key: str, lat: float, lon: float) -> None:
        self.sender = sender
        self.lat = lat
        self.lon = lon
        self._update_period = 10.0 * 60.0
        self._api = pyowm.OWM(open_weather_key)
        self._mgr = self._api.weather_manager()

    async def run(self) -> None:
        """run() periodically queries Open Weather and queues the weather
        telemetry for publishing."""

        print("Weather Fetcher started")
        try:
            while True:
                res = self._fetch()
                if res is not None:
                    await self._publish(res)
                await asyncio.sleep(self._update_period)
        except asyncio.exceptions.CancelledError:
            print("Weather Fetcher stopped")
            return

    def _fetch(self) -> OneCall:
        print("Weather Fetcher requesting data...")
        try:
            res = self._mgr.one_call(lat=self.lat, lon=self.lon)
        except PyOWMError as err:
            print(f"Error fetching weather data: {err}")
            return None
        print("Weather is:", res.current.temperature("celsius"),
              res.current.humidity, res.current.barometric_pressure("hPa"))
        return res

    async def _publish(self, res: OneCall):
        try:
            await self.sender.send(
                "weather.temp",
                res.current.temperature("celsius")["temp"])
            await self.sender.send(
                "weather.humidity",
                res.current.humidity)
            await self.sender.send(
                "weather.pressure",
                res.current.barometric_pressure("hPa")["press"])
        except asyncio.exceptions.CancelledError:
            print("Weather Fetcher stopped")
            return

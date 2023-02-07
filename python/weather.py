import asyncio
import pyowm
import pyowm.commons.exceptions as owmexceptions

class Fetcher:
    def __init__(self, sender: object, open_weather_key: str, lat: float, lon: float) -> None:
        self.UPDATE_PERIOD = 10.0 * 60.0
        self._sender = sender
        self._api = pyowm.OWM(open_weather_key)
        self._mgr = self._api.weather_manager()
        self.lat = lat
        self.lon = lon

    async def run(self):
        print("Weather Fetcher started")
        try:
            while True:
                res = self._fetch()
                if res is not None:
                    await self._send()
                await asyncio.sleep(self.UPDATE_PERIOD)
        except asyncio.exceptions.CancelledError:
            print("Weather Fetcher stopped")
            return
       
    def _fetch(self):
        print("Weather Fetcher requesting data...")
        try:
            res = self._mgr.one_call(lat=self.lat, lon=self.lon)
        except owmexceptions.PyOWMError as err:
            print("Error fetching weather data: {}".format(err))
            return None
        print("Weather is:", res.current.temperature("celsius"),
            res.current.humidity, res.current.barometric_pressure("hPa"))

    async def _send(self, res: object):
        try:
            await self._sender.send("weather.temp", res.current.temperature("celsius")["temp"])
            await self._sender.send("weather.humidity", res.current.humidity)
            await self._sender.send("weather.pressure", res.current.barometric_pressure("hPa")["press"])
        except asyncio.exceptions.CancelledError:
            print("Weather Fetcher stopped")
            return

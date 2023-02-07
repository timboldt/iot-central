"""Uploads sensor telemetry and web API data to Adafruit IO"""
import asyncio
import os

import adafruit
import weather


async def main():
    """Uploads sensor telemetry and web API data to Adafruit IO"""
    sender = adafruit.Sender(os.getenv("IO_USERNAME"),
                             os.getenv("IO_KEY"))

    weather_fetcher = weather.Fetcher(
        sender,
        os.getenv("OPEN_WEATHER_KEY"),
        float(os.getenv("OPEN_WEATHER_LAT")),
        float(os.getenv("OPEN_WEATHER_LON")))

    send_task = asyncio.create_task(sender.run())
    weather_task = asyncio.create_task(weather_fetcher.run())
    await asyncio.sleep(10.0)

    # Stop the producers.
    weather_task.cancel()

    # Drain and then stop the consumer.
    await sender.wait_for_drain()
    send_task.cancel()

    asyncio.gather(
        send_task,
        weather_task
    )

asyncio.run(main())

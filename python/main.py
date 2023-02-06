import asyncio
import os
import random
import time

import adafruit


async def main():
    sender = adafruit.Sender("https://io.adafruit.com/api/v2",
                             os.getenv("IO_USERNAME"),
                             os.getenv("IO_KEY"))
    send_task = asyncio.create_task(sender.run())

    # Generate random timings and put them into the queue.
    total_sleep_time = 0
    for _ in range(20):
        sleep_for = random.uniform(0.05, 1.0)
        total_sleep_time += sleep_for
        await sender.send("feed1", sleep_for)

    await sender.wait_for_drain()
    send_task.cancel()
    await send_task

    print('====')
    print(f'total expected sleep time: {total_sleep_time:.2f} seconds')


asyncio.run(main())

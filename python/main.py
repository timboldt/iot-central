import asyncio
import random
import time

import adafruit

async def main():
    sender = adafruit.Sender()
    queue = sender.queue()
    send_task = asyncio.create_task(sender.run())

    # Generate random timings and put them into the queue.
    total_sleep_time = 0
    for _ in range(20):
        sleep_for = random.uniform(0.05, 1.0)
        total_sleep_time += sleep_for
        await queue.put(sleep_for)

    started_at = time.monotonic()
    await queue.join()
    total_slept_for = time.monotonic() - started_at

    send_task.cancel()
    await send_task

    print('====')
    print(f'3 workers slept in parallel for {total_slept_for:.2f} seconds')
    print(f'total expected sleep time: {total_sleep_time:.2f} seconds')


asyncio.run(main())

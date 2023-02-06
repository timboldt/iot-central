import asyncio


class Sender:
    def __init__(self) -> None:
        QUEUE_SIZE = 8
        self._queue = asyncio.Queue(QUEUE_SIZE)

    def queue(self) -> asyncio.Queue:
        return self._queue

    async def run(self) -> None:
        while True:
            try:
                msg = await self._queue.get()
            except asyncio.exceptions.CancelledError:
                return
            # TODO(timboldt): actually send the message
            print("Received message: {}".format(msg))
            self._queue.task_done()

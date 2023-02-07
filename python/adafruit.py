import Adafruit_IO
import asyncio


class Message:
    def __init__(self, feed: str, value: str) -> None:
        self.feed = feed
        self.value = value


class Sender:
    def __init__(self, io_user: str, io_key: str) -> None:
        QUEUE_SIZE = 8
        self._queue = asyncio.Queue(QUEUE_SIZE)
        self._io_user = io_user
        self._io_key = io_key
        self._aio = Adafruit_IO.Client(self._io_user, self._io_key)

    async def send(self, feed: str, value: str) -> None:
        await self._queue.put(Message(feed, value))

    async def wait_for_drain(self) -> None:
        await self._queue.join()

    async def run(self) -> None:
        try:
            print("Adafruit IO Sender started")
            while True:
                msg = await self._queue.get()
                self._send(msg)
                self._queue.task_done()
        except asyncio.exceptions.CancelledError:
            print("Adafruit IO Sender stopped")
            return

    def _send(self, msg: Message) -> None:
        try:
            self._aio.send_data(msg.feed, msg.value)
            print("Adafruit IO sender sent: {}/{}".format(msg.feed, msg.value))
        except Adafruit_IO.errors.RequestError as err:
            print("Adafruit IO Sender request error: {}".format(err))
        except Adafruit_IO.errors.AdafruitIOError as err:
            print("Adafruit IO Sender general error: {}".format(err))

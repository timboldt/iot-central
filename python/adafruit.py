"""Uploads telemetry to Adafruit IO"""
import asyncio

import Adafruit_IO

QUEUE_SIZE = 8


class Message:
    """A Message contains a feed and value pair, to be uploaded to Adafruit IO."""
    def __init__(self, feed: str, value: str) -> None:
        self.feed = feed
        self.value = value


class Sender:
    """A Sender gets Messages from a queue and sends them to Adafruit IO."""
    def __init__(self, io_user: str, io_key: str) -> None:
        self._queue = asyncio.Queue(QUEUE_SIZE)
        self._io_user = io_user
        self._io_key = io_key
        self._aio = Adafruit_IO.Client(self._io_user, self._io_key)

    async def send(self, feed: str, value: str) -> None:
        """send() queues Messages onto the queue for subsequent publishing."""
        await self._queue.put(Message(feed, value))

    async def wait_for_drain(self) -> None:
        """wait_for_drain() waits until all messages in the queue have been published."""
        await self._queue.join()

    async def run(self) -> None:
        """run() pulls messages from the queue and publishes them to Adafruit IO."""
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
            print(f"Adafruit IO sender sent: {msg.feed}/{msg.value}")
        except Adafruit_IO.errors.RequestError as err:
            print(f"Adafruit IO Sender request error: {err}")
        except Adafruit_IO.errors.AdafruitIOError as err:
            print(f"Adafruit IO Sender general error: {err}")

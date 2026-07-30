import asyncio


async def test_worker_drains_eventually():
    await asyncio.sleep(2)
    assert queue_is_empty()

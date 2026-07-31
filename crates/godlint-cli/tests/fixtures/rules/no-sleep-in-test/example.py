import time


def test_worker_drains():
    start_worker()
    time.sleep(2)
    assert queue_is_empty()

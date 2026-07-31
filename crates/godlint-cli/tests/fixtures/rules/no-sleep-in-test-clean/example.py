import time


def wait_for_shutdown():
    time.sleep(5)


def test_worker_drains():
    start_worker()
    assert eventually(queue_is_empty)

import random


def test_shuffle_is_stable():
    random.seed(1)
    items = random.sample(pool, 10)
    assert sorted(items) == sorted(shuffle(items))

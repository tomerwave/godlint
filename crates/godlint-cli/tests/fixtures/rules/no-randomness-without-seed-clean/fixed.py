def test_shuffle_is_stable():
    items = [3, 1, 2]
    assert sorted(items) == sorted(shuffle(items))

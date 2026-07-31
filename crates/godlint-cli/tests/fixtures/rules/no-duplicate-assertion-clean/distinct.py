def test_total_is_charged():
    total = charge(order)
    assert total == 100
    assert order.state == "charged"

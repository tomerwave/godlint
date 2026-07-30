def test_refund_is_processed():
    assert process_refund(order).status == "refunded"


def test_refund_rejects_a_closed_order():
    with pytest.raises(ClosedOrder):
        process_refund(closed)

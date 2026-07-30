#[test]
fn refund_is_processed() {
    assert_eq!(process_refund(order).status, Status::Refunded);
}

#[test]
#[should_panic(expected = "closed")]
fn refund_rejects_a_closed_order() {
    process_refund(closed);
}

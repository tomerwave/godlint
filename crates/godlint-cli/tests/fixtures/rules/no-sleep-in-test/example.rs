#[test]
fn worker_drains() {
    start_worker();
    std::thread::sleep(DELAY);
    assert!(queue_is_empty());
}

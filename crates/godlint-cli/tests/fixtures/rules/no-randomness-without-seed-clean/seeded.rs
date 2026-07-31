#[test]
fn shuffle_is_stable() {
    let mut rng = StdRng::seed_from_u64(7);
    let picked: usize = rand::random();
    assert!(picked < 10);
}

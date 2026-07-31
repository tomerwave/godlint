#[test]
fn prices_a_random_basket() {
    let size: usize = rand::random();
    assert!(price(basket(size)) > 0);
}

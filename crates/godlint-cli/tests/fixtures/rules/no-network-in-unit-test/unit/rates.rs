#[test]
fn reads_the_rates() {
    let response = reqwest::blocking::get("https://api.example.com/rates");
    assert!(response.is_ok());
}

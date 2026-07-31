def test_rates_against_the_real_service():
    response = requests.get("https://api.example.com/rates")
    assert response.status_code == 200

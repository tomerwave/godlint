def test_rates():
    response = requests.get("https://api.example.com/rates")
    assert response.status_code == 200

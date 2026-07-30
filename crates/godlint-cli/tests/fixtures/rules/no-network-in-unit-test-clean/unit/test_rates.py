def test_rates():
    rates = Rates(client=FakeClient({"EUR": 1.1}))
    assert rates.of("EUR") == 1.1

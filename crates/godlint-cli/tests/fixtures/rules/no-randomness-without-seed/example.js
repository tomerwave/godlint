it("prices a random basket", () => {
  const size = Math.floor(Math.random() * 10);
  expect(price(basket(size))).toBeGreaterThan(0);
});

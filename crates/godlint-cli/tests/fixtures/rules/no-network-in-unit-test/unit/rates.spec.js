it("reads the rates", async () => {
  const response = await fetch("https://api.example.com/rates");
  expect(response.ok).toBe(true);
});

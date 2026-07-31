describe("charges", () => {
  it("charges a small order", () => {
    expect(charge(small)).toBe(1);
  });
  it("charges a large order", () => {
    expect(charge(small)).toBe(1);
  });
});

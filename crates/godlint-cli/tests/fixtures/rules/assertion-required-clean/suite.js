describe("refunds", () => {
  it("processes a refund", () => {
    expect(processRefund(order).status).toBe("refunded");
  });
});

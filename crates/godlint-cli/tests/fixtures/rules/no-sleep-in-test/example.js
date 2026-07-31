it("drains the queue", async () => {
  startWorker();
  await page.waitForTimeout(2000);
  expect(await queueIsEmpty()).toBe(true);
});

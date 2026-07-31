it("drains the queue", async () => {
  startWorker();
  await expect.poll(() => queueIsEmpty()).toBe(true);
});

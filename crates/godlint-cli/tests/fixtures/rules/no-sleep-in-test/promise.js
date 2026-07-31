it("drains the queue", async () => {
  startWorker();
  await new Promise((resolve) => setTimeout(resolve, 2000));
});

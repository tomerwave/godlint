it("drains the queue", async () => {
  startWorker();
  await new Promise((resolve, reject) => {
    queue.on("drained", resolve);
    setTimeout(() => reject(new Error("never drained")), 5000);
  });
});

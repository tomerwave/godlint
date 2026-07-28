function accepted(value: number): number {
  if (value === 1) {
    return 10;
  } else if (value === 2) {
    return 20;
  } else if (value === 3) {
    return 30;
  }

  return 0;
}

function reported(value: number) {
  if (value === 1) {
    for (const item of items) {
      work(item);
    }
  }
}

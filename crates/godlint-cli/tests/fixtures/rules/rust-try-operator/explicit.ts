function example(): number | null {
  const first = one();
  if (first === null) {
    return null;
  }

  const second = two();
  if (second === null) {
    return null;
  }

  const third = three();
  if (third === null) {
    return null;
  }

  return first + second + third;
}

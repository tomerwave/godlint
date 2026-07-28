/** Sums the positive values. */
export function total(values: number[]): number {
  return values.filter((value) => value > 0).reduce((sum, value) => sum + value, 0);
}

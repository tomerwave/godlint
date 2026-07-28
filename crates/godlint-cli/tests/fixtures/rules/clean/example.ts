// A file that satisfies every configured rule.
export function total(values: number[]): number {
  return values.filter((value) => value > 0).reduce((sum, value) => sum + value, 0);
}

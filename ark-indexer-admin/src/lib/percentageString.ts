export function percentageString(total: number, part: number, digits = 2) {
  const percent = (part / total) * 100;
  return percent < 1 ? percent.toFixed(digits) : percent.toFixed(0);
}

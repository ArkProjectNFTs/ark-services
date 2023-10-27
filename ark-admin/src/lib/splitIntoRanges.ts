export function splitIntoRanges(total: number, count: number) {
  const rangeSize = Math.floor(total / count);
  const ranges = [];

  for (let i = 0; i < count; i++) {
    const start = i * rangeSize;
    const end = (i !== count - 1 ? (i + 1) * rangeSize : total) - 1;

    ranges.push([start, end]);
  }

  return ranges;
}

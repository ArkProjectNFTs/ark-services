export function splitIntoRanges(number: number, count: number) {
  const rangeSize = Math.floor(number / count);
  const ranges = [];

  for (let i = 0; i < count; i++) {
    const start = i * rangeSize;
    const end = i !== count - 1 ? start + rangeSize : number;

    ranges.push([start, end]);
  }

  return ranges;
}

export function containsNumbersInRange(
  arr: number[],
  start: number,
  end: number,
) {
  const numSet = new Set(arr);

  for (let i = start; i <= end; i++) {
    if (numSet.has(i)) {
      return true;
    }
  }

  return false;
}

export function numbersInRange(arr: number[], start: number, end: number) {
  const numSet = new Set(arr);
  const result: number[] = [];

  for (let i = start; i <= end; i++) {
    if (numSet.has(i)) {
      result.push(i);
    }
  }

  return result;
}

export function numbersNotInRange(arr: number[], start: number, end: number) {
  const numSet = new Set(arr);
  const result = [];

  for (let i = start; i <= end; i++) {
    if (!numSet.has(i)) {
      result.push(i);
    }
  }

  return result;
}

export function splitIntoRanges(total: number, count: number) {
  const rangeSize = Math.floor(total / count);
  const ranges: [number, number][] = [];

  for (let i = 0; i < count; i++) {
    const start = i * rangeSize;
    const end = (i !== count - 1 ? (i + 1) * rangeSize : total) - 1;

    ranges.push([start, end]);
  }

  return ranges;
}

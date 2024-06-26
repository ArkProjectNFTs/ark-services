/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
import { type Block, type Network } from "~/types";
import { pool } from "./postgres";
import { type Range } from "./range";

/**
 * Fetches blocks from the database, calculates ranges and returns them.
 *
 * @param {Network} network - Network type parameter.
 * @param {number} latest - Latest block number.
 *
 * @returns {Object} - Returns ranges, rangeSize, and count.
 */
export async function fetchBlocks(network: Network, latest: number) {
  const existingBlocks = await fetchAllBlocks();
  const count = latest - existingBlocks.length;
  const rangeCount = 120;
  const rangeSize = Math.ceil(latest / rangeCount);

  const ranges: Range[] = createEmptyRanges(latest, rangeCount, rangeSize);
  populateRangesWithBlocks(ranges, existingBlocks, rangeSize, latest);

  return { ranges, rangeSize, count };
}

export async function fetchIndexers() {
  const res = await pool.query(
    `SELECT indexer_identifier, indexer_status, last_updated_timestamp, created_timestamp, indexer_version, indexation_progress_percentage, current_block_number, is_force_mode_enabled, start_block_number, end_block_number 
    FROM public.indexer
    ORDER BY created_timestamp DESC`,
    [],
  );

  return res.rows;
}

export async function fetchLatestBlocks(): Promise<Block[]> {
  const res = await pool.query(
    "SELECT block_number, block_timestamp, block_status FROM block WHERE block_status = 'Terminated' ORDER BY block_number DESC LIMIT 10",
    [],
  );

  return res.rows;
}

/**
 * Fetch all indexed blocks
 *
 * @param {Network} network - Network type parameter.
 *
 * @returns {Promise<Record<string, AttributeValue>[]>} - Returns all fetched items.
 */
async function fetchAllBlocks(): Promise<string[]> {
  const res = await pool.query(
    "SELECT block_number FROM block WHERE block_status = 'Terminated' ORDER BY block_number ASC",
    [],
  );

  return res.rows.map((row: any) => row.block_number);
}

/**
 * Creates a list of empty ranges based on provided parameters.
 *
 * @param {number} latest - Latest block number.
 * @param {number} rangeCount - Total count of ranges.
 * @param {number} rangeSize - Size of each range.
 *
 * @returns {Range[]} - Returns an array of empty ranges.
 */
function createEmptyRanges(
  latest: number,
  rangeCount: number,
  rangeSize: number,
): Range[] {
  return Array.from({ length: rangeCount }, (_, i) => {
    const start = i * rangeSize;
    const end = i !== rangeCount - 1 ? (i + 1) * rangeSize - 1 : latest;
    return { start, end, blocks: [] };
  });
}

/**
 * Populates the ranges with blocks from the list of all items.
 *
 * @param {Range[]} ranges - Array of ranges.
 * @param {Record<string, AttributeValue>[]} allItems - All fetched items.
 * @param {number} rangeSize - Size of each range.
 * @param {number} latest - Latest block number.
 */
function populateRangesWithBlocks(
  ranges: Range[],
  existingBlocks: string[],
  rangeSize: number,
  latest: number,
) {
  let nextExpectedBlock = 0;

  for (const existingBlock of existingBlocks) {
    const blockNumber = parseInt(existingBlock, 10);
    const block = blockNumber;

    while (nextExpectedBlock < block) {
      const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
      ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
      nextExpectedBlock++;
    }

    nextExpectedBlock = block + 1;
  }

  while (nextExpectedBlock <= latest) {
    const rangeIndex = Math.floor(nextExpectedBlock / rangeSize);
    ranges[rangeIndex]?.blocks.push(nextExpectedBlock);
    nextExpectedBlock++;
  }
}

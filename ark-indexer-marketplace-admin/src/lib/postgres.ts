import pg from "pg";

const { Pool } = pg;

export const pool = new Pool();

export async function insertIndexer(
  taskId: string,
  indexerVersion: string,
  progress: number,
  currentBlockNumber: number,
  isForceMode: boolean,
  from: number,
  to: number,
) {
  await pool.query(
    `INSERT INTO public.indexer
(indexer_identifier, indexer_status, last_updated_timestamp, created_timestamp, indexer_version, indexation_progress_percentage, current_block_number, is_force_mode_enabled, start_block_number, end_block_number)
VALUES($1, 'requested', EXTRACT(epoch FROM now())::bigint, EXTRACT(epoch FROM now())::bigint, $2, $3, $4, $5, $6, $7);`,
    [
      taskId,
      indexerVersion,
      progress,
      currentBlockNumber,
      isForceMode,
      from,
      to,
    ],
  );
}

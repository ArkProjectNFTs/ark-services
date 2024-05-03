CREATE TABLE indexer (
     task_id SERIAL PRIMARY KEY,
     status TEXT NOT NULL,
     last_update BIGINT NOT NULL,
     version TEXT NOT NULL,
     indexation_progress REAL,
     current_block_number BIGINT
);

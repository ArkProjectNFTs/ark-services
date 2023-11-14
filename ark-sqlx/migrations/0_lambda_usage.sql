-- SQL migration for lambda usage.
--
CREATE TABLE ark_lambda_usage (
       request_id TEXT NOT NULL,
       api_key TEXT NOT NULL,
       timestamp BIGINT NOT NULL,
       capacity FLOAT8 NOT NULL,
       execution_time_in_ms BIGINT NOT NULL,
       response_status_code INT NOT NULL,
       request_method TEXT NOT NULL,
       request_path TEXT NOT NULL,
       request_params TEXT NOT NULL,
       ip VARCHAR(64) NOT NULL,
       stage_name TEXT NOT NULL,

       PRIMARY KEY (request_id)
);

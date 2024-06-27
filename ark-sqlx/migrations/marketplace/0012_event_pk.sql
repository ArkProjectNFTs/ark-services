ALTER TABLE token_event DROP CONSTRAINT token_event_pkey;
ALTER TABLE token_event DROP COLUMN token_event_id;
ALTER TABLE token_event ADD COLUMN id SERIAL PRIMARY KEY;

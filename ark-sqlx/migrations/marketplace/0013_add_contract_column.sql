ALTER TABLE contract
ADD COLUMN is_refreshing BOOLEAN DEFAULT FALSE,
ADD COLUMN last_refreshed TIMESTAMP WITH TIME ZONE;
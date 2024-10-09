ALTER TABLE contract ADD COLUMN total_volume_2 numeric;
ALTER TABLE contract DROP COLUMN total_volume;
ALTER TABLE contract RENAME COLUMN total_volume_2 TO total_volume;

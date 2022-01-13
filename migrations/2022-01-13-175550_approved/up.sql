ALTER TABLE review ADD COLUMN approved bool;
UPDATE review SET approved = true WHERE state = 'approved';
UPDATE review SET approved = false WHERE NOT state = 'approved';
ALTER TABLE review ALTER COLUMN approved SET NOT NULL;

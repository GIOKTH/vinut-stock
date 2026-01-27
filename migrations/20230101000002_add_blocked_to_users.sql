-- Add is_blocked column to users table
ALTER TABLE users ADD COLUMN is_blocked BOOLEAN DEFAULT FALSE;

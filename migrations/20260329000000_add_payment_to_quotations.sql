-- Add payment_amount and payment_currency to quotations table
ALTER TABLE quotations ADD COLUMN payment_amount DECIMAL(19, 4);
ALTER TABLE quotations ADD COLUMN payment_currency VARCHAR(10);

-- Update existing records to match total if appropriate, or leave null
UPDATE quotations SET payment_amount = total_amount, payment_currency = currency_code WHERE payment_amount IS NULL;

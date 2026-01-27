-- Fix numeric overflow in exchange rates and ensure currency columns
-- Up Migration

-- 1. Alter quotations table
ALTER TABLE quotations 
    ALTER COLUMN exchange_rate TYPE DECIMAL(15, 6),
    ALTER COLUMN tax_rate TYPE DECIMAL(5, 2), -- Explicitly set tax_rate precision if needed, or leave as is if sufficient
    ALTER COLUMN discount_amount TYPE DECIMAL(15, 2);

-- 2. Alter sales table
ALTER TABLE sales
    ALTER COLUMN exchange_rate TYPE DECIMAL(15, 6);

-- 3. Alter exchange_rates table (if not already sufficient, though init.sql said 15,6)
-- Just to be safe and consistent
ALTER TABLE exchange_rates
    ALTER COLUMN rate_to_base TYPE DECIMAL(15, 6);

-- Add status to purchases and new_price options to purchase_items
-- Up Migration

-- Alter Purchases table to support status
ALTER TABLE purchases ADD COLUMN status VARCHAR(50) DEFAULT 'PENDING';

-- Alter Purchase Items table to store future pricing details
ALTER TABLE purchase_items ADD COLUMN new_sale_price DECIMAL(15, 2);
ALTER TABLE purchase_items ADD COLUMN new_commission_price DECIMAL(15, 2);
ALTER TABLE purchase_items ADD COLUMN new_promotion_price DECIMAL(15, 2);
ALTER TABLE purchase_items ADD COLUMN landed_cost_base DECIMAL(15, 2);

-- Increase precision for all financial columns to avoid numeric field overflow
-- Up Migration

-- 0. Drop dependent views first
DROP VIEW IF EXISTS dashboard_stats;
DROP VIEW IF EXISTS product_performance;

-- Fix Quotations
ALTER TABLE quotations 
    ALTER COLUMN total_amount TYPE DECIMAL(15, 2),
    ALTER COLUMN tax_rate TYPE DECIMAL(15, 2),
    ALTER COLUMN discount_amount TYPE DECIMAL(15, 2),
    ALTER COLUMN exchange_rate TYPE DECIMAL(15, 6);

-- Fix Quotation Items
ALTER TABLE quotation_items
    ALTER COLUMN unit_price TYPE DECIMAL(15, 2);

-- Check if subtotal exists and fix it, otherwise add it if needed by the code
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='quotation_items' AND column_name='subtotal') THEN
        ALTER TABLE quotation_items ALTER COLUMN subtotal TYPE DECIMAL(15, 2);
    ELSE
        ALTER TABLE quotation_items ADD COLUMN subtotal DECIMAL(15, 2) DEFAULT 0.00;
        UPDATE quotation_items SET subtotal = quantity * unit_price;
        ALTER TABLE quotation_items ALTER COLUMN subtotal SET NOT NULL;
    END IF;
END $$;

-- Fix Sales
ALTER TABLE sales
    ALTER COLUMN total_amount TYPE DECIMAL(15, 2),
    ALTER COLUMN discount_amount TYPE DECIMAL(15, 2),
    ALTER COLUMN exchange_rate TYPE DECIMAL(15, 6);

-- Fix Sale Items
ALTER TABLE sale_items
    ALTER COLUMN unit_price TYPE DECIMAL(15, 2),
    ALTER COLUMN subtotal TYPE DECIMAL(15, 2);

-- Fix Products
ALTER TABLE products
    ALTER COLUMN sale_price TYPE DECIMAL(15, 2),
    ALTER COLUMN commission_price TYPE DECIMAL(15, 2),
    ALTER COLUMN promotion_price TYPE DECIMAL(15, 2);

-- Fix Exchange Rates
ALTER TABLE exchange_rates
    ALTER COLUMN rate_to_base TYPE DECIMAL(15, 6);

-- 4. Recreate Views
-- Dashboard Stats View (Today's Summary)
CREATE OR REPLACE VIEW dashboard_stats AS
WITH daily_sales AS (
    SELECT 
        COALESCE(SUM(s.total_amount), 0) as total_sales_today,
        COALESCE(SUM(si.quantity * (si.unit_price - p.cost_price)), 0) as total_profit_today
    FROM sales s
    LEFT JOIN sale_items si ON s.id = si.sale_id
    LEFT JOIN products p ON si.product_id = p.id
    WHERE s.created_at >= CURRENT_DATE
),
best_seller AS (
    SELECT p.name
    FROM sale_items si
    JOIN products p ON si.product_id = p.id
    JOIN sales s ON si.sale_id = s.id
    WHERE s.created_at >= CURRENT_DATE
    GROUP BY p.name
    ORDER BY SUM(si.quantity) DESC
    LIMIT 1
),
low_stock AS (
    SELECT COUNT(*) as low_stock_count
    FROM products
    WHERE quantity < low_stock_threshold
)
SELECT 
    ds.total_sales_today,
    ds.total_profit_today,
    COALESCE(bs.name, 'N/A') as best_selling_product,
    ls.low_stock_count
 FROM daily_sales ds, low_stock ls
 LEFT JOIN best_seller bs ON true;

 -- Product Performance View (Historical)
 CREATE OR REPLACE VIEW product_performance AS
 SELECT 
    p.name as product_name,
    SUM(si.quantity) as total_sold,
    SUM(si.subtotal) as total_revenue,
    SUM(si.quantity * (si.unit_price - p.cost_price)) as total_profit
 FROM products p
 LEFT JOIN sale_items si ON p.id = si.product_id
 GROUP BY p.id, p.name;

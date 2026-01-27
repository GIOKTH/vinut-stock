-- Add cost_price to products
ALTER TABLE products ADD COLUMN cost_price DECIMAL(15, 2) DEFAULT 0.00;

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

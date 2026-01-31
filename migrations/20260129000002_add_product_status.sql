-- Add is_active status to products
ALTER TABLE products ADD COLUMN is_active BOOLEAN DEFAULT TRUE;

-- Update product_performance view to include current status
DROP VIEW IF EXISTS product_performance;
CREATE VIEW product_performance AS
SELECT 
    p.name as product_name,
    p.quantity as current_stock,
    p.low_stock_threshold,
    p.is_active,
    (p.quantity <= p.low_stock_threshold) as is_low_stock,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity ELSE 0 END), 0) as total_sold,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.subtotal ELSE 0 END), 0) as total_revenue,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity * (si.unit_price - (p.cost_price * COALESCE(s.exchange_rate, 1))) ELSE 0 END), 0) as total_profit
FROM products p
LEFT JOIN sale_items si ON p.id = si.product_id
LEFT JOIN sales s ON si.sale_id = s.id
GROUP BY p.id, p.name, p.quantity, p.low_stock_threshold, p.is_active;

-- Update product_performance view to include stock information and alerts
DROP VIEW IF EXISTS product_performance;
CREATE VIEW product_performance AS
SELECT 
    p.name as product_name,
    p.quantity as current_stock,
    p.low_stock_threshold,
    (p.quantity <= p.low_stock_threshold) as is_low_stock,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity ELSE 0 END), 0) as total_sold,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.subtotal ELSE 0 END), 0) as total_revenue,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity * (si.unit_price - (p.cost_price * COALESCE(s.exchange_rate, 1))) ELSE 0 END), 0) as total_profit
FROM products p
LEFT JOIN sale_items si ON p.id = si.product_id
LEFT JOIN sales s ON si.sale_id = s.id
GROUP BY p.id, p.name, p.quantity, p.low_stock_threshold;

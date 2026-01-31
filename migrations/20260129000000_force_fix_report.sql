-- Refined product_performance view to fix nulls, bug in join, and status filtering
DROP VIEW IF EXISTS product_performance;
CREATE VIEW product_performance AS
SELECT 
    p.name as product_name,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity ELSE 0 END), 0) as total_sold,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.subtotal ELSE 0 END), 0) as total_revenue,
    COALESCE(SUM(CASE WHEN s.status = 'COMPLETED' THEN si.quantity * (si.unit_price - (p.cost_price * COALESCE(s.exchange_rate, 1))) ELSE 0 END), 0) as total_profit
FROM products p
LEFT JOIN sale_items si ON p.id = si.product_id
LEFT JOIN sales s ON si.sale_id = s.id
GROUP BY p.id, p.name;

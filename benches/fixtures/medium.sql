-- A representative analytics query with joins, a CTE, and grouping.
WITH recent AS (
  SELECT id, created_at, customer_id FROM orders
  WHERE created_at >= '2026-01-01'
)
SELECT
  u.id                          AS user_id,
  u.name                        AS user_name,
  COUNT(*)                      AS order_count,
  SUM(r.total)                  AS revenue,
  CAST(AVG(r.total) AS INTEGER) AS avg_order
FROM users u
LEFT JOIN recent r ON u.id = r.customer_id
WHERE u.active = 1 AND u.score > 0.5
GROUP BY u.id
HAVING COUNT(*) > 0
ORDER BY revenue DESC, user_name ASC
LIMIT 50 OFFSET 0;

SELECT id FROM t1 UNION ALL SELECT id FROM t2;

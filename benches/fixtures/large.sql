-- A larger corpus of realistic SELECT statements, intentionally exercising
-- the shared-prefix expression backtracking that motivates memoization:
-- function_call vs qualified_column_ref vs column_ref_bare all start with
-- an identifier.

-- Q1: nested CTE chain with multi-column aggregation.
WITH daily_orders AS (
  SELECT
    customer_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(total) AS revenue
  FROM orders
  WHERE created_at >= '2026-01-01' AND status != 'cancelled'
  GROUP BY customer_id, DATE(created_at)
),
customer_summary AS (
  SELECT
    customer_id,
    SUM(n) AS orders_total,
    SUM(revenue) AS revenue_total,
    AVG(revenue) AS revenue_avg,
    MAX(day) AS last_day
  FROM daily_orders
  GROUP BY customer_id
)
SELECT
  c.id AS customer_id,
  c.name AS customer_name,
  c.tier AS tier,
  cs.orders_total,
  cs.revenue_total,
  CAST(cs.revenue_avg AS INTEGER) AS revenue_avg_int,
  cs.last_day,
  CASE
    WHEN cs.revenue_total >= 10000 THEN 'gold'
    WHEN cs.revenue_total >=  1000 THEN 'silver'
    WHEN cs.revenue_total >=   100 THEN 'bronze'
    ELSE 'basic'
  END AS segment
FROM customers c
LEFT JOIN customer_summary cs ON c.id = cs.customer_id
WHERE c.active = 1
  AND c.created_at <= '2026-04-01'
  AND (c.tier IN ('gold', 'silver', 'bronze') OR c.tier IS NULL)
  AND EXISTS (SELECT 1 FROM orders o WHERE o.customer_id = c.id)
ORDER BY cs.revenue_total DESC, c.name ASC
LIMIT 100 OFFSET 0;

-- Q2: multi-join with GROUP BY and HAVING, exercises aliased_expr and
-- qualified_column_ref paths heavily.
SELECT
  p.id AS product_id,
  p.sku,
  p.name,
  cat.name AS category,
  SUM(oi.quantity)           AS units_sold,
  SUM(oi.quantity * oi.price) AS revenue,
  COUNT(DISTINCT o.customer_id) AS distinct_buyers
FROM products p
INNER JOIN categories cat ON p.category_id = cat.id
INNER JOIN order_items oi ON oi.product_id = p.id
INNER JOIN orders o ON o.id = oi.order_id
WHERE o.status = 'shipped'
  AND o.created_at BETWEEN '2026-01-01' AND '2026-04-01'
  AND p.discontinued = 0
GROUP BY p.id, p.sku, p.name, cat.name
HAVING SUM(oi.quantity) > 10
ORDER BY revenue DESC
LIMIT 200;

-- Q3: compound SELECT via UNION ALL with scalar subqueries.
SELECT 'a' AS src, id, name FROM users_archive WHERE active = 1
UNION ALL
SELECT 'b' AS src, id, name FROM users_live    WHERE active = 1
UNION ALL
SELECT 'c' AS src, id, name FROM users_pending WHERE active IS NOT NULL;

-- Q4: IN / LIKE / BETWEEN to exercise the cmp_tail alternatives.
SELECT id, name, email
FROM users
WHERE id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
  AND email LIKE '%@example.com'
  AND name NOT LIKE '_test%'
  AND score BETWEEN 0.25 AND 0.95
  AND created_at GLOB '2026-*'
ORDER BY id;

-- Q5: nested scalar subquery in the result list (shared-prefix primary
-- alternative paren_primary vs function_call vs column_ref hot path).
SELECT
  u.id,
  u.name,
  (SELECT COUNT(*) FROM orders o WHERE o.customer_id = u.id)    AS order_count,
  (SELECT MAX(created_at) FROM orders o WHERE o.customer_id = u.id) AS last_order
FROM users u
WHERE u.active = 1
ORDER BY order_count DESC
LIMIT 25;

-- Q6: bind parameters of every supported shape.
SELECT id FROM t WHERE a = ? AND b = ?1 AND c = :name AND d = @name AND e = $name;

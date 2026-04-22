-- Large SQLite fixture exercising the full-dialect surface: DDL with
-- constraints and generated columns, DML with UPSERT and RETURNING,
-- triggers with multi-statement bodies, window functions, transactions,
-- PRAGMA, and the administrative statements. Designed for both the
-- memoization bench (shared-prefix expression backtracking) and as a
-- visual smoke test for the highlighter.

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = 'wal';

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS customers (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL COLLATE NOCASE,
  email      TEXT UNIQUE,
  tier       TEXT DEFAULT 'standard' CHECK (tier IN ('standard', 'gold', 'vip')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  CONSTRAINT customers_email_lower CHECK (email = LOWER(email))
) STRICT;

CREATE TABLE orders (
  id          INTEGER PRIMARY KEY,
  customer_id INTEGER NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
  created_at  DATETIME NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  total       NUMERIC(10, 2) NOT NULL,
  status      TEXT NOT NULL DEFAULT 'pending',
  total_cents INTEGER GENERATED ALWAYS AS (CAST(total * 100 AS INTEGER)) STORED,
  FOREIGN KEY (customer_id) REFERENCES customers(id)
    DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX IF NOT EXISTS customers_email_idx
  ON customers (email) WHERE email IS NOT NULL;

CREATE INDEX orders_by_day_idx
  ON orders (DATE(created_at), customer_id) WHERE status != 'cancelled';

CREATE VIEW customer_revenue (customer_id, revenue_total, last_order) AS
  SELECT customer_id, SUM(total), MAX(created_at)
  FROM orders
  WHERE status = 'completed'
  GROUP BY customer_id;

CREATE TRIGGER customers_touch_updated_at
  AFTER UPDATE ON customers
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE customers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER orders_audit AFTER INSERT ON orders FOR EACH ROW
BEGIN
  INSERT INTO audit_log (op, entity, entity_id, payload)
    VALUES ('insert', 'order', NEW.id, NEW.total);
  UPDATE stats SET n_orders = n_orders + 1 WHERE key = 'global';
END;

COMMIT;

-- Day-to-day DML with UPSERT, RETURNING, INSERT OR X prefixes, and
-- DELETE with a CTE-qualified row-set.

INSERT INTO customers (name, email, tier) VALUES
  ('Ada Lovelace',    'ada@example.com',    'vip'),
  ('Grace Hopper',    'grace@example.com',  'gold'),
  ('Alan Turing',     'alan@example.com',   'standard')
ON CONFLICT (email) DO UPDATE
  SET name = excluded.name, tier = excluded.tier
  WHERE customers.tier != 'vip'
RETURNING id, name, tier;

INSERT OR REPLACE INTO stats (key, n_orders) VALUES ('global', 0);

REPLACE INTO feature_flags (key, enabled) VALUES ('beta_ui', 1);

UPDATE customers
   SET tier = 'gold'
  FROM orders AS o
 WHERE customers.id = o.customer_id AND o.total > 1000
RETURNING customers.id, customers.tier;

UPDATE OR IGNORE stats SET n_orders = n_orders - 1 WHERE key = 'global';

WITH stale AS (
  SELECT id FROM customers WHERE updated_at < '2020-01-01'
)
DELETE FROM customers
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

-- Analytical SELECT: recursive CTE, compound SELECT, window functions,
-- FILTER clause, named windows, CASE expressions, CAST, EXISTS.

WITH RECURSIVE days(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days WHERE d < DATE('2026-01-31')
),
daily_orders AS (
  SELECT
    customer_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(total) AS revenue,
    SUM(total) FILTER (WHERE status = 'completed') AS revenue_completed
  FROM orders
  WHERE created_at >= '2026-01-01' AND status != 'cancelled'
  GROUP BY customer_id, DATE(created_at)
)
SELECT
  c.id AS customer_id,
  c.name AS customer_name,
  c.tier AS tier,
  do.day,
  do.n,
  do.revenue,
  SUM(do.revenue) OVER w_trail AS revenue_7d,
  RANK()          OVER w_rank  AS daily_rank,
  CASE
    WHEN do.revenue > 10000 THEN 'platinum'
    WHEN do.revenue > 1000  THEN 'gold'
    ELSE c.tier
  END AS effective_tier,
  CAST(do.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM orders o2 WHERE o2.customer_id = c.id AND o2.total > 5000) AS is_big_spender
FROM daily_orders AS do
JOIN customers  AS c ON c.id = do.customer_id
LEFT JOIN churn AS k ON k.customer_id = c.id
WINDOW
  w_trail AS (PARTITION BY do.customer_id ORDER BY do.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank  AS (PARTITION BY do.day ORDER BY do.revenue DESC)
ORDER BY do.day, do.customer_id
LIMIT 500 OFFSET 0;

SELECT id FROM orders WHERE id = 1
UNION ALL SELECT id FROM orders WHERE id = 2
INTERSECT SELECT id FROM orders WHERE id > 0
EXCEPT    SELECT id FROM orders WHERE status = 'cancelled';

-- Schema changes, virtual table, admin statements.

ALTER TABLE customers ADD COLUMN notes TEXT;
ALTER TABLE customers RENAME COLUMN tier TO membership_tier;
ALTER TABLE orders DROP COLUMN total_cents;

CREATE VIRTUAL TABLE fts_customers USING fts5(name, email, content='customers');

ATTACH DATABASE 'backup.db' AS backup;
DETACH DATABASE backup;

REINDEX customers_email_idx;
ANALYZE main.orders;
VACUUM INTO 'snapshot.db';

EXPLAIN QUERY PLAN SELECT * FROM orders WHERE customer_id = 42;

SAVEPOINT post_migration;
ROLLBACK TO SAVEPOINT post_migration;
RELEASE SAVEPOINT post_migration;

DROP TRIGGER IF EXISTS orders_audit;
DROP VIEW   IF EXISTS customer_revenue;
DROP INDEX  IF EXISTS orders_by_day_idx;
DROP TABLE  IF EXISTS stale_imports;

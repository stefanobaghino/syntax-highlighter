-- xlarge SQLite fixture. Scaled-up shape of large.sql for the
-- memo-threshold sweep; exercises DDL/DML variety at volume without
-- relying on the recovery operator (covered separately).

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = 'wal';
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -20000;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS customers (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT customers_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS customers_code_idx
  ON customers (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS customers_status_idx
  ON customers (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS customers_touch_updated
  AFTER UPDATE ON customers
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE customers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS orders (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES customers(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT orders_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS orders_code_idx
  ON orders (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS orders_status_idx
  ON orders (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS orders_touch_updated
  AFTER UPDATE ON orders
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE orders SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS products (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES orders(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT products_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS products_code_idx
  ON products (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS products_status_idx
  ON products (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS products_touch_updated
  AFTER UPDATE ON products
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE products SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS shipments (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES products(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT shipments_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS shipments_code_idx
  ON shipments (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS shipments_status_idx
  ON shipments (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS shipments_touch_updated
  AFTER UPDATE ON shipments
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE shipments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS returns (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES shipments(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT returns_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS returns_code_idx
  ON returns (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS returns_status_idx
  ON returns (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS returns_touch_updated
  AFTER UPDATE ON returns
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE returns SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS inventory (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES returns(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT inventory_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS inventory_code_idx
  ON inventory (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS inventory_status_idx
  ON inventory (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS inventory_touch_updated
  AFTER UPDATE ON inventory
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE inventory SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS warehouses (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES inventory(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT warehouses_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS warehouses_code_idx
  ON warehouses (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS warehouses_status_idx
  ON warehouses (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS warehouses_touch_updated
  AFTER UPDATE ON warehouses
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE warehouses SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS invoices (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES warehouses(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT invoices_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS invoices_code_idx
  ON invoices (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS invoices_status_idx
  ON invoices (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS invoices_touch_updated
  AFTER UPDATE ON invoices
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE invoices SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS payments (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES invoices(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT payments_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS payments_code_idx
  ON payments (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS payments_status_idx
  ON payments (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS payments_touch_updated
  AFTER UPDATE ON payments
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE payments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS line_items (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES payments(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT line_items_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS line_items_code_idx
  ON line_items (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS line_items_status_idx
  ON line_items (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS line_items_touch_updated
  AFTER UPDATE ON line_items
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE line_items SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS employees (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES line_items(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT employees_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS employees_code_idx
  ON employees (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS employees_status_idx
  ON employees (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS employees_touch_updated
  AFTER UPDATE ON employees
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE employees SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS departments (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES employees(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT departments_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS departments_code_idx
  ON departments (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS departments_status_idx
  ON departments (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS departments_touch_updated
  AFTER UPDATE ON departments
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE departments SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS suppliers (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES departments(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT suppliers_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS suppliers_code_idx
  ON suppliers (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS suppliers_status_idx
  ON suppliers (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS suppliers_touch_updated
  AFTER UPDATE ON suppliers
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE suppliers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS categories (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES suppliers(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT categories_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS categories_code_idx
  ON categories (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS categories_status_idx
  ON categories (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS categories_touch_updated
  AFTER UPDATE ON categories
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE categories SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS reviews (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES categories(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT reviews_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS reviews_code_idx
  ON reviews (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS reviews_status_idx
  ON reviews (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS reviews_touch_updated
  AFTER UPDATE ON reviews
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE reviews SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS coupons (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES reviews(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT coupons_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS coupons_code_idx
  ON coupons (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS coupons_status_idx
  ON coupons (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS coupons_touch_updated
  AFTER UPDATE ON coupons
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE coupons SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS carts (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES coupons(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT carts_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS carts_code_idx
  ON carts (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS carts_status_idx
  ON carts (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS carts_touch_updated
  AFTER UPDATE ON carts
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE carts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS sessions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES carts(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT sessions_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS sessions_code_idx
  ON sessions (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS sessions_status_idx
  ON sessions (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS sessions_touch_updated
  AFTER UPDATE ON sessions
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS events (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES sessions(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT events_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS events_code_idx
  ON events (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS events_status_idx
  ON events (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS events_touch_updated
  AFTER UPDATE ON events
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE events SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS audit_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES events(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT audit_log_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS audit_log_code_idx
  ON audit_log (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS audit_log_status_idx
  ON audit_log (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS audit_log_touch_updated
  AFTER UPDATE ON audit_log
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE audit_log SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS stats (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES audit_log(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT stats_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS stats_code_idx
  ON stats (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS stats_status_idx
  ON stats (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS stats_touch_updated
  AFTER UPDATE ON stats
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE stats SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS feature_flags (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  code       TEXT UNIQUE NOT NULL COLLATE NOCASE,
  label      TEXT NOT NULL,
  amount     NUMERIC(12, 4) DEFAULT 0.0,
  status     TEXT CHECK (status IN ('open','closed','pending','archived')),
  created_at DATETIME DEFAULT (CURRENT_TIMESTAMP),
  updated_at DATETIME,
  parent_id  INTEGER REFERENCES stats(id) ON DELETE CASCADE,
  meta_json  TEXT,
  active     INTEGER GENERATED ALWAYS AS (status != 'archived') STORED,
  CONSTRAINT feature_flags_amount_nn CHECK (amount >= 0)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS feature_flags_code_idx
  ON feature_flags (code) WHERE code IS NOT NULL;

CREATE INDEX IF NOT EXISTS feature_flags_status_idx
  ON feature_flags (status, DATE(created_at)) WHERE status != 'archived';

CREATE TRIGGER IF NOT EXISTS feature_flags_touch_updated
  AFTER UPDATE ON feature_flags
  FOR EACH ROW
  WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE feature_flags SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

COMMIT;

INSERT INTO customers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-0-0', 'label 0-0', 1.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-0-1', 'label 0-1', 2.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-0-2', 'label 0-2', 3.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-0-3', 'label 0-3', 5.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE customers.status != 'archived'
RETURNING id, code, status;

UPDATE customers
   SET status = 'closed'
  FROM orders AS o
 WHERE customers.id = o.parent_id AND o.amount > 100
RETURNING customers.id, customers.status;

WITH stale AS (
  SELECT id FROM customers WHERE updated_at < '2020-01-01'
)
DELETE FROM customers
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO orders (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-1-0', 'label 1-0', 2.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-1-1', 'label 1-1', 5.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-1-2', 'label 1-2', 7.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-1-3', 'label 1-3', 10.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE orders.status != 'archived'
RETURNING id, code, status;

INSERT INTO products (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-2-0', 'label 2-0', 3.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-2-1', 'label 2-1', 7.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-2-2', 'label 2-2', 11.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-2-3', 'label 2-3', 15.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE products.status != 'archived'
RETURNING id, code, status;

INSERT INTO shipments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-3-0', 'label 3-0', 5.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-3-1', 'label 3-1', 10.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-3-2', 'label 3-2', 15.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-3-3', 'label 3-3', 20.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE shipments.status != 'archived'
RETURNING id, code, status;

INSERT INTO returns (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-4-0', 'label 4-0', 6.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-4-1', 'label 4-1', 12.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-4-2', 'label 4-2', 18.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-4-3', 'label 4-3', 25.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE returns.status != 'archived'
RETURNING id, code, status;

INSERT INTO inventory (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-5-0', 'label 5-0', 7.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-5-1', 'label 5-1', 15.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-5-2', 'label 5-2', 22.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-5-3', 'label 5-3', 30.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE inventory.status != 'archived'
RETURNING id, code, status;

UPDATE inventory
   SET status = 'closed'
  FROM orders AS o
 WHERE inventory.id = o.parent_id AND o.amount > 100
RETURNING inventory.id, inventory.status;

INSERT INTO warehouses (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-6-0', 'label 6-0', 8.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-6-1', 'label 6-1', 17.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-6-2', 'label 6-2', 26.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-6-3', 'label 6-3', 35.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE warehouses.status != 'archived'
RETURNING id, code, status;

INSERT INTO invoices (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-7-0', 'label 7-0', 10.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-7-1', 'label 7-1', 20.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-7-2', 'label 7-2', 30.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-7-3', 'label 7-3', 40.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE invoices.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM invoices WHERE updated_at < '2020-01-01'
)
DELETE FROM invoices
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO payments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-8-0', 'label 8-0', 11.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-8-1', 'label 8-1', 22.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-8-2', 'label 8-2', 33.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-8-3', 'label 8-3', 45.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE payments.status != 'archived'
RETURNING id, code, status;

INSERT INTO line_items (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-9-0', 'label 9-0', 12.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-9-1', 'label 9-1', 25.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-9-2', 'label 9-2', 37.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-9-3', 'label 9-3', 50.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE line_items.status != 'archived'
RETURNING id, code, status;

INSERT INTO employees (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-10-0', 'label 10-0', 13.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-10-1', 'label 10-1', 27.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-10-2', 'label 10-2', 41.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-10-3', 'label 10-3', 55.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE employees.status != 'archived'
RETURNING id, code, status;

UPDATE employees
   SET status = 'closed'
  FROM orders AS o
 WHERE employees.id = o.parent_id AND o.amount > 100
RETURNING employees.id, employees.status;

INSERT INTO departments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-11-0', 'label 11-0', 15.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-11-1', 'label 11-1', 30.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-11-2', 'label 11-2', 45.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-11-3', 'label 11-3', 60.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE departments.status != 'archived'
RETURNING id, code, status;

INSERT INTO suppliers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-12-0', 'label 12-0', 16.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-12-1', 'label 12-1', 32.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-12-2', 'label 12-2', 48.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-12-3', 'label 12-3', 65.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE suppliers.status != 'archived'
RETURNING id, code, status;

INSERT INTO categories (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-13-0', 'label 13-0', 17.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-13-1', 'label 13-1', 35.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-13-2', 'label 13-2', 52.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-13-3', 'label 13-3', 70.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE categories.status != 'archived'
RETURNING id, code, status;

INSERT INTO reviews (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-14-0', 'label 14-0', 18.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-14-1', 'label 14-1', 37.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-14-2', 'label 14-2', 56.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-14-3', 'label 14-3', 75.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE reviews.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM reviews WHERE updated_at < '2020-01-01'
)
DELETE FROM reviews
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO coupons (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-15-0', 'label 15-0', 20.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-15-1', 'label 15-1', 40.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-15-2', 'label 15-2', 60.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-15-3', 'label 15-3', 80.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE coupons.status != 'archived'
RETURNING id, code, status;

UPDATE coupons
   SET status = 'closed'
  FROM orders AS o
 WHERE coupons.id = o.parent_id AND o.amount > 100
RETURNING coupons.id, coupons.status;

INSERT INTO carts (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-16-0', 'label 16-0', 21.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-16-1', 'label 16-1', 42.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-16-2', 'label 16-2', 63.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-16-3', 'label 16-3', 85.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE carts.status != 'archived'
RETURNING id, code, status;

INSERT INTO sessions (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-17-0', 'label 17-0', 22.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-17-1', 'label 17-1', 45.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-17-2', 'label 17-2', 67.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-17-3', 'label 17-3', 90.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE sessions.status != 'archived'
RETURNING id, code, status;

INSERT INTO events (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-18-0', 'label 18-0', 23.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-18-1', 'label 18-1', 47.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-18-2', 'label 18-2', 71.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-18-3', 'label 18-3', 95.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE events.status != 'archived'
RETURNING id, code, status;

INSERT INTO audit_log (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-19-0', 'label 19-0', 25.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-19-1', 'label 19-1', 50.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-19-2', 'label 19-2', 75.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-19-3', 'label 19-3', 100.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE audit_log.status != 'archived'
RETURNING id, code, status;

INSERT INTO stats (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-20-0', 'label 20-0', 26.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-20-1', 'label 20-1', 52.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-20-2', 'label 20-2', 78.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-20-3', 'label 20-3', 105.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE stats.status != 'archived'
RETURNING id, code, status;

UPDATE stats
   SET status = 'closed'
  FROM orders AS o
 WHERE stats.id = o.parent_id AND o.amount > 100
RETURNING stats.id, stats.status;

INSERT INTO feature_flags (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-21-0', 'label 21-0', 27.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-21-1', 'label 21-1', 55.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-21-2', 'label 21-2', 82.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-21-3', 'label 21-3', 110.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE feature_flags.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM feature_flags WHERE updated_at < '2020-01-01'
)
DELETE FROM feature_flags
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO customers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-22-0', 'label 22-0', 28.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-22-1', 'label 22-1', 57.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-22-2', 'label 22-2', 86.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-22-3', 'label 22-3', 115.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE customers.status != 'archived'
RETURNING id, code, status;

INSERT INTO orders (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-23-0', 'label 23-0', 30.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-23-1', 'label 23-1', 60.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-23-2', 'label 23-2', 90.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-23-3', 'label 23-3', 120.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE orders.status != 'archived'
RETURNING id, code, status;

INSERT INTO products (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-24-0', 'label 24-0', 31.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-24-1', 'label 24-1', 62.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-24-2', 'label 24-2', 93.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-24-3', 'label 24-3', 125.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE products.status != 'archived'
RETURNING id, code, status;

INSERT INTO shipments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-25-0', 'label 25-0', 32.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-25-1', 'label 25-1', 65.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-25-2', 'label 25-2', 97.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-25-3', 'label 25-3', 130.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE shipments.status != 'archived'
RETURNING id, code, status;

UPDATE shipments
   SET status = 'closed'
  FROM orders AS o
 WHERE shipments.id = o.parent_id AND o.amount > 100
RETURNING shipments.id, shipments.status;

INSERT INTO returns (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-26-0', 'label 26-0', 33.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-26-1', 'label 26-1', 67.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-26-2', 'label 26-2', 101.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-26-3', 'label 26-3', 135.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE returns.status != 'archived'
RETURNING id, code, status;

INSERT INTO inventory (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-27-0', 'label 27-0', 35.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-27-1', 'label 27-1', 70.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-27-2', 'label 27-2', 105.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-27-3', 'label 27-3', 140.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE inventory.status != 'archived'
RETURNING id, code, status;

INSERT INTO warehouses (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-28-0', 'label 28-0', 36.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-28-1', 'label 28-1', 72.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-28-2', 'label 28-2', 108.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-28-3', 'label 28-3', 145.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE warehouses.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM warehouses WHERE updated_at < '2020-01-01'
)
DELETE FROM warehouses
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO invoices (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-29-0', 'label 29-0', 37.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-29-1', 'label 29-1', 75.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-29-2', 'label 29-2', 112.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-29-3', 'label 29-3', 150.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE invoices.status != 'archived'
RETURNING id, code, status;

INSERT INTO payments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-30-0', 'label 30-0', 38.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-30-1', 'label 30-1', 77.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-30-2', 'label 30-2', 116.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-30-3', 'label 30-3', 155.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE payments.status != 'archived'
RETURNING id, code, status;

UPDATE payments
   SET status = 'closed'
  FROM orders AS o
 WHERE payments.id = o.parent_id AND o.amount > 100
RETURNING payments.id, payments.status;

INSERT INTO line_items (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-31-0', 'label 31-0', 40.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-31-1', 'label 31-1', 80.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-31-2', 'label 31-2', 120.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-31-3', 'label 31-3', 160.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE line_items.status != 'archived'
RETURNING id, code, status;

INSERT INTO employees (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-32-0', 'label 32-0', 41.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-32-1', 'label 32-1', 82.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-32-2', 'label 32-2', 123.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-32-3', 'label 32-3', 165.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE employees.status != 'archived'
RETURNING id, code, status;

INSERT INTO departments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-33-0', 'label 33-0', 42.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-33-1', 'label 33-1', 85.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-33-2', 'label 33-2', 127.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-33-3', 'label 33-3', 170.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE departments.status != 'archived'
RETURNING id, code, status;

INSERT INTO suppliers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-34-0', 'label 34-0', 43.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-34-1', 'label 34-1', 87.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-34-2', 'label 34-2', 131.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-34-3', 'label 34-3', 175.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE suppliers.status != 'archived'
RETURNING id, code, status;

INSERT INTO categories (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-35-0', 'label 35-0', 45.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-35-1', 'label 35-1', 90.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-35-2', 'label 35-2', 135.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-35-3', 'label 35-3', 180.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE categories.status != 'archived'
RETURNING id, code, status;

UPDATE categories
   SET status = 'closed'
  FROM orders AS o
 WHERE categories.id = o.parent_id AND o.amount > 100
RETURNING categories.id, categories.status;

WITH stale AS (
  SELECT id FROM categories WHERE updated_at < '2020-01-01'
)
DELETE FROM categories
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO reviews (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-36-0', 'label 36-0', 46.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-36-1', 'label 36-1', 92.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-36-2', 'label 36-2', 138.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-36-3', 'label 36-3', 185.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE reviews.status != 'archived'
RETURNING id, code, status;

INSERT INTO coupons (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-37-0', 'label 37-0', 47.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-37-1', 'label 37-1', 95.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-37-2', 'label 37-2', 142.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-37-3', 'label 37-3', 190.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE coupons.status != 'archived'
RETURNING id, code, status;

INSERT INTO carts (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-38-0', 'label 38-0', 48.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-38-1', 'label 38-1', 97.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-38-2', 'label 38-2', 146.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-38-3', 'label 38-3', 195.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE carts.status != 'archived'
RETURNING id, code, status;

INSERT INTO sessions (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-39-0', 'label 39-0', 50.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-39-1', 'label 39-1', 100.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-39-2', 'label 39-2', 150.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-39-3', 'label 39-3', 200.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE sessions.status != 'archived'
RETURNING id, code, status;

INSERT INTO events (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-40-0', 'label 40-0', 51.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-40-1', 'label 40-1', 102.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-40-2', 'label 40-2', 153.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-40-3', 'label 40-3', 205.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE events.status != 'archived'
RETURNING id, code, status;

UPDATE events
   SET status = 'closed'
  FROM orders AS o
 WHERE events.id = o.parent_id AND o.amount > 100
RETURNING events.id, events.status;

INSERT INTO audit_log (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-41-0', 'label 41-0', 52.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-41-1', 'label 41-1', 105.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-41-2', 'label 41-2', 157.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-41-3', 'label 41-3', 210.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE audit_log.status != 'archived'
RETURNING id, code, status;

INSERT INTO stats (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-42-0', 'label 42-0', 53.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-42-1', 'label 42-1', 107.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-42-2', 'label 42-2', 161.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-42-3', 'label 42-3', 215.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE stats.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM stats WHERE updated_at < '2020-01-01'
)
DELETE FROM stats
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO feature_flags (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-43-0', 'label 43-0', 55.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-43-1', 'label 43-1', 110.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-43-2', 'label 43-2', 165.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-43-3', 'label 43-3', 220.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE feature_flags.status != 'archived'
RETURNING id, code, status;

INSERT INTO customers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-44-0', 'label 44-0', 56.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-44-1', 'label 44-1', 112.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-44-2', 'label 44-2', 168.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-44-3', 'label 44-3', 225.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE customers.status != 'archived'
RETURNING id, code, status;

INSERT INTO orders (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-45-0', 'label 45-0', 57.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-45-1', 'label 45-1', 115.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-45-2', 'label 45-2', 172.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-45-3', 'label 45-3', 230.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE orders.status != 'archived'
RETURNING id, code, status;

UPDATE orders
   SET status = 'closed'
  FROM orders AS o
 WHERE orders.id = o.parent_id AND o.amount > 100
RETURNING orders.id, orders.status;

INSERT INTO products (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-46-0', 'label 46-0', 58.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-46-1', 'label 46-1', 117.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-46-2', 'label 46-2', 176.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-46-3', 'label 46-3', 235.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE products.status != 'archived'
RETURNING id, code, status;

INSERT INTO shipments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-47-0', 'label 47-0', 60.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-47-1', 'label 47-1', 120.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-47-2', 'label 47-2', 180.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-47-3', 'label 47-3', 240.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE shipments.status != 'archived'
RETURNING id, code, status;

INSERT INTO returns (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-48-0', 'label 48-0', 61.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-48-1', 'label 48-1', 122.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-48-2', 'label 48-2', 183.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-48-3', 'label 48-3', 245.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE returns.status != 'archived'
RETURNING id, code, status;

INSERT INTO inventory (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-49-0', 'label 49-0', 62.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-49-1', 'label 49-1', 125.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-49-2', 'label 49-2', 187.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-49-3', 'label 49-3', 250.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE inventory.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM inventory WHERE updated_at < '2020-01-01'
)
DELETE FROM inventory
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO warehouses (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-50-0', 'label 50-0', 63.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-50-1', 'label 50-1', 127.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-50-2', 'label 50-2', 191.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-50-3', 'label 50-3', 255.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE warehouses.status != 'archived'
RETURNING id, code, status;

UPDATE warehouses
   SET status = 'closed'
  FROM orders AS o
 WHERE warehouses.id = o.parent_id AND o.amount > 100
RETURNING warehouses.id, warehouses.status;

INSERT INTO invoices (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-51-0', 'label 51-0', 65.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-51-1', 'label 51-1', 130.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-51-2', 'label 51-2', 195.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-51-3', 'label 51-3', 260.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE invoices.status != 'archived'
RETURNING id, code, status;

INSERT INTO payments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-52-0', 'label 52-0', 66.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-52-1', 'label 52-1', 132.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-52-2', 'label 52-2', 198.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-52-3', 'label 52-3', 265.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE payments.status != 'archived'
RETURNING id, code, status;

INSERT INTO line_items (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-53-0', 'label 53-0', 67.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-53-1', 'label 53-1', 135.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-53-2', 'label 53-2', 202.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-53-3', 'label 53-3', 270.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE line_items.status != 'archived'
RETURNING id, code, status;

INSERT INTO employees (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-54-0', 'label 54-0', 68.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-54-1', 'label 54-1', 137.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-54-2', 'label 54-2', 206.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-54-3', 'label 54-3', 275.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE employees.status != 'archived'
RETURNING id, code, status;

INSERT INTO departments (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-55-0', 'label 55-0', 70.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-55-1', 'label 55-1', 140.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-55-2', 'label 55-2', 210.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-55-3', 'label 55-3', 280.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE departments.status != 'archived'
RETURNING id, code, status;

UPDATE departments
   SET status = 'closed'
  FROM orders AS o
 WHERE departments.id = o.parent_id AND o.amount > 100
RETURNING departments.id, departments.status;

INSERT INTO suppliers (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-56-0', 'label 56-0', 71.25, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-56-1', 'label 56-1', 142.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-56-2', 'label 56-2', 213.75, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-56-3', 'label 56-3', 285.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE suppliers.status != 'archived'
RETURNING id, code, status;

WITH stale AS (
  SELECT id FROM suppliers WHERE updated_at < '2020-01-01'
)
DELETE FROM suppliers
 WHERE id IN (SELECT id FROM stale)
 RETURNING *;

INSERT INTO categories (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-57-0', 'label 57-0', 72.5, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-57-1', 'label 57-1', 145.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-57-2', 'label 57-2', 217.5, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-57-3', 'label 57-3', 290.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE categories.status != 'archived'
RETURNING id, code, status;

INSERT INTO reviews (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-58-0', 'label 58-0', 73.75, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-58-1', 'label 58-1', 147.5, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-58-2', 'label 58-2', 221.25, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-58-3', 'label 58-3', 295.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE reviews.status != 'archived'
RETURNING id, code, status;

INSERT INTO coupons (code, label, amount, status, created_at, updated_at, parent_id, meta_json) VALUES
  ('code-59-0', 'label 59-0', 75.0, 'open', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-59-1', 'label 59-1', 150.0, 'closed', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-59-2', 'label 59-2', 225.0, 'pending', CURRENT_TIMESTAMP, NULL, NULL, '{}'),
  ('code-59-3', 'label 59-3', 300.0, 'archived', CURRENT_TIMESTAMP, NULL, NULL, '{}')
ON CONFLICT (code) DO UPDATE
  SET label = excluded.label, amount = excluded.amount
  WHERE coupons.status != 'archived'
RETURNING id, code, status;

WITH RECURSIVE days_0(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_0 WHERE d < DATE('2026-02-28')
),
daily_0 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM customers
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS customers_id,
  a.label AS customers_label,
  a.status AS customers_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_0 AS revenue_7d,
  RANK()         OVER w_rank_0  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM orders o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_0 AS d
JOIN customers AS a ON a.id = d.parent_id
LEFT JOIN products AS k ON k.parent_id = a.id
WINDOW
  w_trail_0 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_0  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 100 OFFSET 0;

SELECT id FROM customers WHERE id = 0
UNION ALL SELECT id FROM orders WHERE id = 1
INTERSECT SELECT id FROM products WHERE id > 0
EXCEPT    SELECT id FROM customers WHERE status = 'archived';

WITH RECURSIVE days_1(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_1 WHERE d < DATE('2026-02-28')
),
daily_1 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM orders
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS orders_id,
  a.label AS orders_label,
  a.status AS orders_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_1 AS revenue_7d,
  RANK()         OVER w_rank_1  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM products o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_1 AS d
JOIN orders AS a ON a.id = d.parent_id
LEFT JOIN shipments AS k ON k.parent_id = a.id
WINDOW
  w_trail_1 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_1  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 101 OFFSET 10;

SELECT id FROM orders WHERE id = 1
UNION ALL SELECT id FROM products WHERE id = 2
INTERSECT SELECT id FROM shipments WHERE id > 0
EXCEPT    SELECT id FROM orders WHERE status = 'archived';

WITH RECURSIVE days_2(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_2 WHERE d < DATE('2026-02-28')
),
daily_2 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM products
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS products_id,
  a.label AS products_label,
  a.status AS products_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_2 AS revenue_7d,
  RANK()         OVER w_rank_2  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM shipments o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_2 AS d
JOIN products AS a ON a.id = d.parent_id
LEFT JOIN returns AS k ON k.parent_id = a.id
WINDOW
  w_trail_2 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_2  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 102 OFFSET 20;

SELECT id FROM products WHERE id = 2
UNION ALL SELECT id FROM shipments WHERE id = 3
INTERSECT SELECT id FROM returns WHERE id > 0
EXCEPT    SELECT id FROM products WHERE status = 'archived';

WITH RECURSIVE days_3(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_3 WHERE d < DATE('2026-02-28')
),
daily_3 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM shipments
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS shipments_id,
  a.label AS shipments_label,
  a.status AS shipments_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_3 AS revenue_7d,
  RANK()         OVER w_rank_3  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM returns o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_3 AS d
JOIN shipments AS a ON a.id = d.parent_id
LEFT JOIN inventory AS k ON k.parent_id = a.id
WINDOW
  w_trail_3 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_3  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 103 OFFSET 30;

SELECT id FROM shipments WHERE id = 3
UNION ALL SELECT id FROM returns WHERE id = 4
INTERSECT SELECT id FROM inventory WHERE id > 0
EXCEPT    SELECT id FROM shipments WHERE status = 'archived';

WITH RECURSIVE days_4(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_4 WHERE d < DATE('2026-02-28')
),
daily_4 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM returns
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS returns_id,
  a.label AS returns_label,
  a.status AS returns_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_4 AS revenue_7d,
  RANK()         OVER w_rank_4  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM inventory o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_4 AS d
JOIN returns AS a ON a.id = d.parent_id
LEFT JOIN warehouses AS k ON k.parent_id = a.id
WINDOW
  w_trail_4 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_4  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 104 OFFSET 40;

SELECT id FROM returns WHERE id = 4
UNION ALL SELECT id FROM inventory WHERE id = 5
INTERSECT SELECT id FROM warehouses WHERE id > 0
EXCEPT    SELECT id FROM returns WHERE status = 'archived';

WITH RECURSIVE days_5(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_5 WHERE d < DATE('2026-02-28')
),
daily_5 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM inventory
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS inventory_id,
  a.label AS inventory_label,
  a.status AS inventory_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_5 AS revenue_7d,
  RANK()         OVER w_rank_5  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM warehouses o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_5 AS d
JOIN inventory AS a ON a.id = d.parent_id
LEFT JOIN invoices AS k ON k.parent_id = a.id
WINDOW
  w_trail_5 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_5  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 105 OFFSET 50;

SELECT id FROM inventory WHERE id = 5
UNION ALL SELECT id FROM warehouses WHERE id = 6
INTERSECT SELECT id FROM invoices WHERE id > 0
EXCEPT    SELECT id FROM inventory WHERE status = 'archived';

WITH RECURSIVE days_6(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_6 WHERE d < DATE('2026-02-28')
),
daily_6 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM warehouses
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS warehouses_id,
  a.label AS warehouses_label,
  a.status AS warehouses_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_6 AS revenue_7d,
  RANK()         OVER w_rank_6  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM invoices o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_6 AS d
JOIN warehouses AS a ON a.id = d.parent_id
LEFT JOIN payments AS k ON k.parent_id = a.id
WINDOW
  w_trail_6 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_6  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 106 OFFSET 60;

SELECT id FROM warehouses WHERE id = 6
UNION ALL SELECT id FROM invoices WHERE id = 7
INTERSECT SELECT id FROM payments WHERE id > 0
EXCEPT    SELECT id FROM warehouses WHERE status = 'archived';

WITH RECURSIVE days_7(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_7 WHERE d < DATE('2026-02-28')
),
daily_7 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM invoices
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS invoices_id,
  a.label AS invoices_label,
  a.status AS invoices_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_7 AS revenue_7d,
  RANK()         OVER w_rank_7  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM payments o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_7 AS d
JOIN invoices AS a ON a.id = d.parent_id
LEFT JOIN line_items AS k ON k.parent_id = a.id
WINDOW
  w_trail_7 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_7  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 107 OFFSET 70;

SELECT id FROM invoices WHERE id = 7
UNION ALL SELECT id FROM payments WHERE id = 8
INTERSECT SELECT id FROM line_items WHERE id > 0
EXCEPT    SELECT id FROM invoices WHERE status = 'archived';

WITH RECURSIVE days_8(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_8 WHERE d < DATE('2026-02-28')
),
daily_8 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM payments
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS payments_id,
  a.label AS payments_label,
  a.status AS payments_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_8 AS revenue_7d,
  RANK()         OVER w_rank_8  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM line_items o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_8 AS d
JOIN payments AS a ON a.id = d.parent_id
LEFT JOIN employees AS k ON k.parent_id = a.id
WINDOW
  w_trail_8 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_8  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 108 OFFSET 80;

SELECT id FROM payments WHERE id = 8
UNION ALL SELECT id FROM line_items WHERE id = 9
INTERSECT SELECT id FROM employees WHERE id > 0
EXCEPT    SELECT id FROM payments WHERE status = 'archived';

WITH RECURSIVE days_9(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_9 WHERE d < DATE('2026-02-28')
),
daily_9 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM line_items
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS line_items_id,
  a.label AS line_items_label,
  a.status AS line_items_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_9 AS revenue_7d,
  RANK()         OVER w_rank_9  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM employees o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_9 AS d
JOIN line_items AS a ON a.id = d.parent_id
LEFT JOIN departments AS k ON k.parent_id = a.id
WINDOW
  w_trail_9 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_9  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 109 OFFSET 90;

SELECT id FROM line_items WHERE id = 9
UNION ALL SELECT id FROM employees WHERE id = 10
INTERSECT SELECT id FROM departments WHERE id > 0
EXCEPT    SELECT id FROM line_items WHERE status = 'archived';

WITH RECURSIVE days_10(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_10 WHERE d < DATE('2026-02-28')
),
daily_10 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM employees
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS employees_id,
  a.label AS employees_label,
  a.status AS employees_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_10 AS revenue_7d,
  RANK()         OVER w_rank_10  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM departments o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_10 AS d
JOIN employees AS a ON a.id = d.parent_id
LEFT JOIN suppliers AS k ON k.parent_id = a.id
WINDOW
  w_trail_10 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_10  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 110 OFFSET 100;

SELECT id FROM employees WHERE id = 10
UNION ALL SELECT id FROM departments WHERE id = 11
INTERSECT SELECT id FROM suppliers WHERE id > 0
EXCEPT    SELECT id FROM employees WHERE status = 'archived';

WITH RECURSIVE days_11(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_11 WHERE d < DATE('2026-02-28')
),
daily_11 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM departments
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS departments_id,
  a.label AS departments_label,
  a.status AS departments_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_11 AS revenue_7d,
  RANK()         OVER w_rank_11  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM suppliers o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_11 AS d
JOIN departments AS a ON a.id = d.parent_id
LEFT JOIN categories AS k ON k.parent_id = a.id
WINDOW
  w_trail_11 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_11  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 111 OFFSET 110;

SELECT id FROM departments WHERE id = 11
UNION ALL SELECT id FROM suppliers WHERE id = 12
INTERSECT SELECT id FROM categories WHERE id > 0
EXCEPT    SELECT id FROM departments WHERE status = 'archived';

WITH RECURSIVE days_12(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_12 WHERE d < DATE('2026-02-28')
),
daily_12 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM suppliers
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS suppliers_id,
  a.label AS suppliers_label,
  a.status AS suppliers_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_12 AS revenue_7d,
  RANK()         OVER w_rank_12  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM categories o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_12 AS d
JOIN suppliers AS a ON a.id = d.parent_id
LEFT JOIN reviews AS k ON k.parent_id = a.id
WINDOW
  w_trail_12 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_12  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 112 OFFSET 120;

SELECT id FROM suppliers WHERE id = 12
UNION ALL SELECT id FROM categories WHERE id = 13
INTERSECT SELECT id FROM reviews WHERE id > 0
EXCEPT    SELECT id FROM suppliers WHERE status = 'archived';

WITH RECURSIVE days_13(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_13 WHERE d < DATE('2026-02-28')
),
daily_13 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM categories
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS categories_id,
  a.label AS categories_label,
  a.status AS categories_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_13 AS revenue_7d,
  RANK()         OVER w_rank_13  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM reviews o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_13 AS d
JOIN categories AS a ON a.id = d.parent_id
LEFT JOIN coupons AS k ON k.parent_id = a.id
WINDOW
  w_trail_13 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_13  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 113 OFFSET 130;

SELECT id FROM categories WHERE id = 13
UNION ALL SELECT id FROM reviews WHERE id = 14
INTERSECT SELECT id FROM coupons WHERE id > 0
EXCEPT    SELECT id FROM categories WHERE status = 'archived';

WITH RECURSIVE days_14(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_14 WHERE d < DATE('2026-02-28')
),
daily_14 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM reviews
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS reviews_id,
  a.label AS reviews_label,
  a.status AS reviews_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_14 AS revenue_7d,
  RANK()         OVER w_rank_14  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM coupons o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_14 AS d
JOIN reviews AS a ON a.id = d.parent_id
LEFT JOIN carts AS k ON k.parent_id = a.id
WINDOW
  w_trail_14 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_14  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 114 OFFSET 140;

SELECT id FROM reviews WHERE id = 14
UNION ALL SELECT id FROM coupons WHERE id = 15
INTERSECT SELECT id FROM carts WHERE id > 0
EXCEPT    SELECT id FROM reviews WHERE status = 'archived';

WITH RECURSIVE days_15(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_15 WHERE d < DATE('2026-02-28')
),
daily_15 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM coupons
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS coupons_id,
  a.label AS coupons_label,
  a.status AS coupons_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_15 AS revenue_7d,
  RANK()         OVER w_rank_15  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM carts o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_15 AS d
JOIN coupons AS a ON a.id = d.parent_id
LEFT JOIN sessions AS k ON k.parent_id = a.id
WINDOW
  w_trail_15 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_15  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 115 OFFSET 150;

SELECT id FROM coupons WHERE id = 15
UNION ALL SELECT id FROM carts WHERE id = 16
INTERSECT SELECT id FROM sessions WHERE id > 0
EXCEPT    SELECT id FROM coupons WHERE status = 'archived';

WITH RECURSIVE days_16(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_16 WHERE d < DATE('2026-02-28')
),
daily_16 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM carts
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS carts_id,
  a.label AS carts_label,
  a.status AS carts_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_16 AS revenue_7d,
  RANK()         OVER w_rank_16  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM sessions o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_16 AS d
JOIN carts AS a ON a.id = d.parent_id
LEFT JOIN events AS k ON k.parent_id = a.id
WINDOW
  w_trail_16 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_16  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 116 OFFSET 160;

SELECT id FROM carts WHERE id = 16
UNION ALL SELECT id FROM sessions WHERE id = 17
INTERSECT SELECT id FROM events WHERE id > 0
EXCEPT    SELECT id FROM carts WHERE status = 'archived';

WITH RECURSIVE days_17(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_17 WHERE d < DATE('2026-02-28')
),
daily_17 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM sessions
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS sessions_id,
  a.label AS sessions_label,
  a.status AS sessions_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_17 AS revenue_7d,
  RANK()         OVER w_rank_17  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM events o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_17 AS d
JOIN sessions AS a ON a.id = d.parent_id
LEFT JOIN audit_log AS k ON k.parent_id = a.id
WINDOW
  w_trail_17 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_17  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 117 OFFSET 170;

SELECT id FROM sessions WHERE id = 17
UNION ALL SELECT id FROM events WHERE id = 18
INTERSECT SELECT id FROM audit_log WHERE id > 0
EXCEPT    SELECT id FROM sessions WHERE status = 'archived';

WITH RECURSIVE days_18(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_18 WHERE d < DATE('2026-02-28')
),
daily_18 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM events
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS events_id,
  a.label AS events_label,
  a.status AS events_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_18 AS revenue_7d,
  RANK()         OVER w_rank_18  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM audit_log o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_18 AS d
JOIN events AS a ON a.id = d.parent_id
LEFT JOIN stats AS k ON k.parent_id = a.id
WINDOW
  w_trail_18 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_18  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 118 OFFSET 180;

SELECT id FROM events WHERE id = 18
UNION ALL SELECT id FROM audit_log WHERE id = 19
INTERSECT SELECT id FROM stats WHERE id > 0
EXCEPT    SELECT id FROM events WHERE status = 'archived';

WITH RECURSIVE days_19(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_19 WHERE d < DATE('2026-02-28')
),
daily_19 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM audit_log
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS audit_log_id,
  a.label AS audit_log_label,
  a.status AS audit_log_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_19 AS revenue_7d,
  RANK()         OVER w_rank_19  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM stats o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_19 AS d
JOIN audit_log AS a ON a.id = d.parent_id
LEFT JOIN feature_flags AS k ON k.parent_id = a.id
WINDOW
  w_trail_19 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_19  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 119 OFFSET 190;

SELECT id FROM audit_log WHERE id = 19
UNION ALL SELECT id FROM stats WHERE id = 20
INTERSECT SELECT id FROM feature_flags WHERE id > 0
EXCEPT    SELECT id FROM audit_log WHERE status = 'archived';

WITH RECURSIVE days_20(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_20 WHERE d < DATE('2026-02-28')
),
daily_20 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM stats
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS stats_id,
  a.label AS stats_label,
  a.status AS stats_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_20 AS revenue_7d,
  RANK()         OVER w_rank_20  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM feature_flags o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_20 AS d
JOIN stats AS a ON a.id = d.parent_id
LEFT JOIN customers AS k ON k.parent_id = a.id
WINDOW
  w_trail_20 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_20  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 120 OFFSET 200;

SELECT id FROM stats WHERE id = 20
UNION ALL SELECT id FROM feature_flags WHERE id = 21
INTERSECT SELECT id FROM customers WHERE id > 0
EXCEPT    SELECT id FROM stats WHERE status = 'archived';

WITH RECURSIVE days_21(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_21 WHERE d < DATE('2026-02-28')
),
daily_21 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM feature_flags
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS feature_flags_id,
  a.label AS feature_flags_label,
  a.status AS feature_flags_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_21 AS revenue_7d,
  RANK()         OVER w_rank_21  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM customers o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_21 AS d
JOIN feature_flags AS a ON a.id = d.parent_id
LEFT JOIN orders AS k ON k.parent_id = a.id
WINDOW
  w_trail_21 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_21  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 121 OFFSET 210;

SELECT id FROM feature_flags WHERE id = 21
UNION ALL SELECT id FROM customers WHERE id = 22
INTERSECT SELECT id FROM orders WHERE id > 0
EXCEPT    SELECT id FROM feature_flags WHERE status = 'archived';

WITH RECURSIVE days_22(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_22 WHERE d < DATE('2026-02-28')
),
daily_22 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM customers
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS customers_id,
  a.label AS customers_label,
  a.status AS customers_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_22 AS revenue_7d,
  RANK()         OVER w_rank_22  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM orders o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_22 AS d
JOIN customers AS a ON a.id = d.parent_id
LEFT JOIN products AS k ON k.parent_id = a.id
WINDOW
  w_trail_22 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_22  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 122 OFFSET 220;

SELECT id FROM customers WHERE id = 22
UNION ALL SELECT id FROM orders WHERE id = 23
INTERSECT SELECT id FROM products WHERE id > 0
EXCEPT    SELECT id FROM customers WHERE status = 'archived';

WITH RECURSIVE days_23(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_23 WHERE d < DATE('2026-02-28')
),
daily_23 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM orders
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS orders_id,
  a.label AS orders_label,
  a.status AS orders_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_23 AS revenue_7d,
  RANK()         OVER w_rank_23  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM products o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_23 AS d
JOIN orders AS a ON a.id = d.parent_id
LEFT JOIN shipments AS k ON k.parent_id = a.id
WINDOW
  w_trail_23 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_23  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 123 OFFSET 230;

SELECT id FROM orders WHERE id = 23
UNION ALL SELECT id FROM products WHERE id = 24
INTERSECT SELECT id FROM shipments WHERE id > 0
EXCEPT    SELECT id FROM orders WHERE status = 'archived';

WITH RECURSIVE days_24(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_24 WHERE d < DATE('2026-02-28')
),
daily_24 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM products
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS products_id,
  a.label AS products_label,
  a.status AS products_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_24 AS revenue_7d,
  RANK()         OVER w_rank_24  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM shipments o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_24 AS d
JOIN products AS a ON a.id = d.parent_id
LEFT JOIN returns AS k ON k.parent_id = a.id
WINDOW
  w_trail_24 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_24  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 124 OFFSET 240;

SELECT id FROM products WHERE id = 24
UNION ALL SELECT id FROM shipments WHERE id = 25
INTERSECT SELECT id FROM returns WHERE id > 0
EXCEPT    SELECT id FROM products WHERE status = 'archived';

WITH RECURSIVE days_25(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_25 WHERE d < DATE('2026-02-28')
),
daily_25 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM shipments
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS shipments_id,
  a.label AS shipments_label,
  a.status AS shipments_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_25 AS revenue_7d,
  RANK()         OVER w_rank_25  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM returns o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_25 AS d
JOIN shipments AS a ON a.id = d.parent_id
LEFT JOIN inventory AS k ON k.parent_id = a.id
WINDOW
  w_trail_25 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_25  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 125 OFFSET 250;

SELECT id FROM shipments WHERE id = 25
UNION ALL SELECT id FROM returns WHERE id = 26
INTERSECT SELECT id FROM inventory WHERE id > 0
EXCEPT    SELECT id FROM shipments WHERE status = 'archived';

WITH RECURSIVE days_26(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_26 WHERE d < DATE('2026-02-28')
),
daily_26 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM returns
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS returns_id,
  a.label AS returns_label,
  a.status AS returns_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_26 AS revenue_7d,
  RANK()         OVER w_rank_26  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM inventory o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_26 AS d
JOIN returns AS a ON a.id = d.parent_id
LEFT JOIN warehouses AS k ON k.parent_id = a.id
WINDOW
  w_trail_26 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_26  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 126 OFFSET 260;

SELECT id FROM returns WHERE id = 26
UNION ALL SELECT id FROM inventory WHERE id = 27
INTERSECT SELECT id FROM warehouses WHERE id > 0
EXCEPT    SELECT id FROM returns WHERE status = 'archived';

WITH RECURSIVE days_27(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_27 WHERE d < DATE('2026-02-28')
),
daily_27 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM inventory
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS inventory_id,
  a.label AS inventory_label,
  a.status AS inventory_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_27 AS revenue_7d,
  RANK()         OVER w_rank_27  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM warehouses o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_27 AS d
JOIN inventory AS a ON a.id = d.parent_id
LEFT JOIN invoices AS k ON k.parent_id = a.id
WINDOW
  w_trail_27 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_27  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 127 OFFSET 270;

SELECT id FROM inventory WHERE id = 27
UNION ALL SELECT id FROM warehouses WHERE id = 28
INTERSECT SELECT id FROM invoices WHERE id > 0
EXCEPT    SELECT id FROM inventory WHERE status = 'archived';

WITH RECURSIVE days_28(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_28 WHERE d < DATE('2026-02-28')
),
daily_28 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM warehouses
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS warehouses_id,
  a.label AS warehouses_label,
  a.status AS warehouses_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_28 AS revenue_7d,
  RANK()         OVER w_rank_28  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM invoices o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_28 AS d
JOIN warehouses AS a ON a.id = d.parent_id
LEFT JOIN payments AS k ON k.parent_id = a.id
WINDOW
  w_trail_28 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_28  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 128 OFFSET 280;

SELECT id FROM warehouses WHERE id = 28
UNION ALL SELECT id FROM invoices WHERE id = 29
INTERSECT SELECT id FROM payments WHERE id > 0
EXCEPT    SELECT id FROM warehouses WHERE status = 'archived';

WITH RECURSIVE days_29(d) AS (
  SELECT DATE('2026-01-01')
  UNION ALL
  SELECT DATE(d, '+1 day') FROM days_29 WHERE d < DATE('2026-02-28')
),
daily_29 AS (
  SELECT
    parent_id,
    DATE(created_at) AS day,
    COUNT(*) AS n,
    SUM(amount) AS revenue,
    SUM(amount) FILTER (WHERE status = 'closed') AS revenue_closed
  FROM invoices
  WHERE created_at >= '2026-01-01' AND status != 'archived'
  GROUP BY parent_id, DATE(created_at)
)
SELECT
  a.id AS invoices_id,
  a.label AS invoices_label,
  a.status AS invoices_status,
  d.day,
  d.n,
  d.revenue,
  SUM(d.revenue) OVER w_trail_29 AS revenue_7d,
  RANK()         OVER w_rank_29  AS daily_rank,
  CASE
    WHEN d.revenue > 10000 THEN 'platinum'
    WHEN d.revenue > 1000  THEN 'gold'
    ELSE a.status
  END AS effective_tier,
  CAST(d.revenue AS INTEGER) AS revenue_int,
  EXISTS (SELECT 1 FROM payments o2 WHERE o2.parent_id = a.id AND o2.amount > 5000) AS is_big
FROM daily_29 AS d
JOIN invoices AS a ON a.id = d.parent_id
LEFT JOIN line_items AS k ON k.parent_id = a.id
WINDOW
  w_trail_29 AS (PARTITION BY d.parent_id ORDER BY d.day
              ROWS BETWEEN 6 PRECEDING AND CURRENT ROW),
  w_rank_29  AS (PARTITION BY d.day ORDER BY d.revenue DESC)
ORDER BY d.day, d.parent_id
LIMIT 129 OFFSET 290;

SELECT id FROM invoices WHERE id = 29
UNION ALL SELECT id FROM payments WHERE id = 30
INTERSECT SELECT id FROM line_items WHERE id > 0
EXCEPT    SELECT id FROM invoices WHERE status = 'archived';

ALTER TABLE customers ADD COLUMN notes TEXT;
ALTER TABLE orders ADD COLUMN notes TEXT;
ALTER TABLE products ADD COLUMN notes TEXT;
ALTER TABLE shipments ADD COLUMN notes TEXT;
ALTER TABLE returns ADD COLUMN notes TEXT;
ALTER TABLE inventory ADD COLUMN notes TEXT;
ALTER TABLE warehouses ADD COLUMN notes TEXT;
ALTER TABLE invoices ADD COLUMN notes TEXT;
ALTER TABLE payments ADD COLUMN notes TEXT;
ALTER TABLE line_items ADD COLUMN notes TEXT;
ALTER TABLE customers RENAME COLUMN label TO display_label;
ALTER TABLE orders RENAME COLUMN label TO display_label;
ALTER TABLE products RENAME COLUMN label TO display_label;
ALTER TABLE shipments RENAME COLUMN label TO display_label;
ALTER TABLE returns RENAME COLUMN label TO display_label;

CREATE VIRTUAL TABLE IF NOT EXISTS fts_customers USING fts5(name, email, content='customers');
CREATE VIRTUAL TABLE IF NOT EXISTS fts_products  USING fts5(code, label);

ATTACH DATABASE 'backup.db' AS backup;
DETACH DATABASE backup;

REINDEX customers_code_idx;
REINDEX orders_code_idx;
REINDEX products_code_idx;
REINDEX shipments_code_idx;
REINDEX returns_code_idx;
REINDEX inventory_code_idx;
REINDEX warehouses_code_idx;
REINDEX invoices_code_idx;
REINDEX payments_code_idx;
REINDEX line_items_code_idx;
REINDEX employees_code_idx;
REINDEX departments_code_idx;
REINDEX suppliers_code_idx;
REINDEX categories_code_idx;
REINDEX reviews_code_idx;
REINDEX coupons_code_idx;
REINDEX carts_code_idx;
REINDEX sessions_code_idx;
REINDEX events_code_idx;
REINDEX audit_log_code_idx;
REINDEX stats_code_idx;
REINDEX feature_flags_code_idx;

ANALYZE main.customers;
ANALYZE main.orders;
ANALYZE main.products;
ANALYZE main.shipments;
ANALYZE main.returns;
ANALYZE main.inventory;
ANALYZE main.warehouses;
ANALYZE main.invoices;
ANALYZE main.payments;
ANALYZE main.line_items;
ANALYZE main.employees;
ANALYZE main.departments;
ANALYZE main.suppliers;
ANALYZE main.categories;
ANALYZE main.reviews;
ANALYZE main.coupons;
ANALYZE main.carts;
ANALYZE main.sessions;
ANALYZE main.events;
ANALYZE main.audit_log;
ANALYZE main.stats;
ANALYZE main.feature_flags;

VACUUM INTO 'snapshot.db';

EXPLAIN QUERY PLAN SELECT * FROM customers WHERE parent_id = 42;
EXPLAIN QUERY PLAN SELECT * FROM orders WHERE parent_id = 42;
EXPLAIN QUERY PLAN SELECT * FROM products WHERE parent_id = 42;
EXPLAIN QUERY PLAN SELECT * FROM shipments WHERE parent_id = 42;
EXPLAIN QUERY PLAN SELECT * FROM returns WHERE parent_id = 42;

SAVEPOINT post_migration;
ROLLBACK TO SAVEPOINT post_migration;
RELEASE SAVEPOINT post_migration;

DROP INDEX IF EXISTS customers_status_idx;
DROP INDEX IF EXISTS orders_status_idx;
DROP INDEX IF EXISTS products_status_idx;
DROP INDEX IF EXISTS shipments_status_idx;
DROP INDEX IF EXISTS returns_status_idx;
DROP INDEX IF EXISTS inventory_status_idx;
DROP INDEX IF EXISTS warehouses_status_idx;
DROP INDEX IF EXISTS invoices_status_idx;
DROP TRIGGER IF EXISTS customers_touch_updated;
DROP TRIGGER IF EXISTS orders_touch_updated;
DROP TRIGGER IF EXISTS products_touch_updated;
DROP TRIGGER IF EXISTS shipments_touch_updated;

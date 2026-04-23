// Generated JavaScript bench fixture. Not hand-maintained —
// regenerate via benches/fixtures/gen_js.py if shapes need tweaking.
// The module is intentionally self-contained (no imports) so the
// grammar sees the full expression/statement surface.

const DEFAULTS = Object.freeze({ size: 16, verbose: false, tag: "default" });

function classify(n) {
  switch (true) {
    case n < 0: return "negative";
    case n === 0: return "zero";
    case n < 10: return "single";
    case n < 100: return "double";
    case n < 1000: return "triple";
    default: return "big";
  }
}

async function loadSamples(source) {
  try {
    const data = await source.read();
    return data.filter(x => x != null).map(x => Number(x));
  } catch (e) {
    console.error(`load failed: ${e.message}`);
    return [];
  } finally {
    source.close?.();
  }
}

const utils = {
  clamp(n, lo, hi) { return n < lo ? lo : n > hi ? hi : n; },
  tagged(strings, ...values) {
    return strings.reduce((acc, s, i) => acc + s + (values[i] ?? ""), "");
  },
  [Symbol.iterator]() { return [1, 2, 3][Symbol.iterator](); },
};

class Base0 {}
class Base1 {}
class Base2 {}
class Base3 {}
class Base4 {}

class Counter0 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter0(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter1 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter1(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter2 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter2(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter3 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter3(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter4 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter4(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter5 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter5(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter6 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter6(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter7 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter7(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter8 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter8(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter9 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter9(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter10 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter10(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter11 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter11(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter12 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter12(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter13 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter13(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter14 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter14(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter15 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter15(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter16 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter16(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter17 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter17(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter18 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter18(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter19 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter19(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter20 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter20(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter21 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter21(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter22 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter22(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter23 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter23(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter24 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter24(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter25 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter25(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter26 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter26(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter27 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter27(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter28 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter28(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter29 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter29(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter30 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter30(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter31 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter31(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter32 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter32(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter33 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter33(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter34 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter34(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter35 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter35(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter36 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter36(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter37 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter37(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter38 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter38(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter39 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter39(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter40 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter40(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter41 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter41(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter42 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter42(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter43 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter43(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter44 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter44(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter45 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter45(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter46 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter46(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter47 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter47(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter48 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter48(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter49 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter49(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter50 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter50(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter51 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter51(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter52 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter52(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter53 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter53(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter54 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter54(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter55 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter55(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter56 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter56(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter57 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter57(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter58 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter58(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter59 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter59(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter60 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter60(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter61 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter61(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter62 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter62(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter63 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter63(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter64 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter64(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter65 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter65(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter66 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter66(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter67 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter67(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter68 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter68(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter69 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter69(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter70 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter70(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter71 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter71(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter72 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter72(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter73 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter73(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter74 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter74(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter75 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter75(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter76 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter76(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter77 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter77(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter78 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter78(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter79 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter79(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter80 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter80(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter81 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter81(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter82 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter82(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter83 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter83(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter84 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter84(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter85 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter85(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter86 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter86(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter87 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter87(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter88 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter88(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter89 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter89(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter90 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter90(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter91 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter91(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter92 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter92(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter93 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter93(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter94 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter94(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter95 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter95(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter96 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter96(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter97 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter97(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter98 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter98(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter99 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter99(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter100 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter100(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter101 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter101(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter102 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter102(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter103 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter103(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter104 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter104(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter105 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter105(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter106 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter106(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter107 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter107(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter108 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter108(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter109 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter109(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter110 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter110(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter111 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter111(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter112 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter112(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter113 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter113(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter114 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter114(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter115 extends Base0 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter115(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter116 extends Base1 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter116(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter117 extends Base2 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter117(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter118 extends Base3 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter118(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}

class Counter119 extends Base4 {
  constructor(name, options = {}) {
    super();
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
    this._cache = null;
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    this._cache = null;
    return this;
  }

  get size() {
    return this.values.length;
  }

  static make(name, ...rest) {
    return new Counter119(name, Object.assign({}, ...rest));
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n, tag: `${this.name}:${k}` };
    }
  }

  async flush(sink) {
    const rows = [...this.entries()];
    for (const row of rows) {
      await sink?.write?.(row);
    }
    return rows.length;
  }

  summary(limit = 3) {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, limit)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    const { size } = this;
    return `${this.name} (n=${size}, top=[${top}])`;
  }
}


async function main() {
  const c = new Counter0("root", { size: 32, verbose: true });
  const samples = [1, 2, 2, 3, 10, 10, 10, -1, "a", "a", "b"];
  for (const x of samples) c.push(x);
  const first = samples[0];
  const rest = samples.slice(1);
  const [head, ...tail] = rest;
  const { name, options: { size = 0, verbose } = {} } = c;
  const label = utils.tagged`counter ${name} size=${size} verbose=${verbose}`;
  console.log(label);
  console.log(c.summary());
  for (const n of samples.map(Number).filter(Boolean)) {
    console.log(`${n} -> ${classify(n)}`);
  }
  const loaded = await loadSamples({
    async read() { return samples; },
    close() { /* noop */ },
  });
  return loaded.reduce((a, b) => a + b, 0);
}

main().catch(e => console.error(e?.stack ?? e));

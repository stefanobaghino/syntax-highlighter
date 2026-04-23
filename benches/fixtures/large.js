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

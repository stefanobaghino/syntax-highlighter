// Medium-sized JavaScript bench fixture: exercises classes, arrow
// fns, destructuring, template literals, async/await, and spread.
// Self-contained so the grammar sees the full surface without an
// import cluster obscuring it.

const DEFAULTS = Object.freeze({ size: 16, verbose: false });

class Counter {
  constructor(name, options = {}) {
    this.name = name;
    this.options = { ...DEFAULTS, ...options };
    this.values = [];
    this.histogram = new Map();
  }

  push(v) {
    const key = typeof v === "string" ? v : String(v);
    this.histogram.set(key, (this.histogram.get(key) ?? 0) + 1);
    this.values.push(v);
    return this;
  }

  get size() {
    return this.values.length;
  }

  *entries() {
    for (const [k, n] of this.histogram) {
      yield { key: k, count: n };
    }
  }

  summary() {
    const top = [...this.histogram.entries()]
      .sort(([, a], [, b]) => b - a)
      .slice(0, 3)
      .map(([k, n]) => `${k}=${n}`)
      .join(", ");
    return `${this.name} (n=${this.size}, top=[${top}])`;
  }
}

function classify(n) {
  switch (true) {
    case n < 0:
      return "negative";
    case n === 0:
      return "zero";
    case n < 10:
      return "single";
    case n < 100:
      return "double";
    default:
      return "large";
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

const sink = {
  log(msg) {
    console.log(msg);
  },
  warn(msg) {
    console.warn(`WARN: ${msg}`);
  },
};

const data = [1, 2, 2, 3, 10, 10, 10, -1];
const c = new Counter("ages");
for (const x of data) c.push(x);
sink.log(c.summary());
for (const n of data) sink.log(`${n} -> ${classify(n)}`);

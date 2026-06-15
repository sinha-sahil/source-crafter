# Temple-DSL Integration

How temple-dsl powers the engine internally, and why YAML authors never see it.

---

## What Temple-DSL Is

Temple is a Rust data-shaping language. It is:

- **Non-Turing-complete** — no loops, no recursion, no arbitrary code
- **Exact decimal arithmetic** — no floating-point surprises (`0.1 + 0.2 == 0.3`)
- **Pre-compiled** — templates compile to CBOR blobs once, execute in microseconds
- **Deterministic** — same input always produces same output

Temple handles: math expressions, `when` conditionals, collection methods (map, filter, fold, sum, any, all, min, max, sort), string operations, and boolean logic.

Source: [github.com/sinha-sahil/temple-dsl](https://github.com/sinha-sahil/temple-dsl)

---

## Temple Is Internal

YAML authors write declarative syntax. The engine translates it to temple internally.

```
YAML author writes:              Engine internally does:
─────────────────                ───────────────────────
compute:                         Compiles to temple blob:
  total: "$a + $b * 0.18"       →  { add($a, mul($b, 0.18)) }

  tier:                          Compiles to temple blob:
    when:                        →  { when($spend >= 1000, "gold",
      - "$spend >= 1000": gold        when($spend >= 250, "silver",
      - "$spend >= 250": silver       "standard")) }
      - else: standard

  names: "$items.map(name)"     →  { map($items, |item| item.name) }
```

The YAML author never writes temple syntax, never imports temple, and never knows it exists. Temple is an implementation detail of the engine, like how a database query planner is internal to a database.

---

## The Blob Model

### Compile Once, Execute Many

```
Deploy time (once):
  YAML compute expressions → temple compiler → CBOR blob (bytes)
  Component HTML templates → temple compiler → CBOR blob (bytes)
  Blobs stored in-memory or on disk

Request time (every request):
  Load blob (no parsing — raw bytes) → execute with current data → result
```

### Why Blobs

| Without blobs | With blobs |
|---------------|------------|
| Parse YAML expression string | Load raw bytes (zero parse) |
| Build AST | AST already in CBOR |
| Evaluate AST | Evaluate CBOR directly |
| ~50-200µs per expression | ~0.5-5.5µs per expression |

That's 10-100x faster. For a page with 50 computed values, this saves 2.5-10ms per request.

### Blob Cache

```rust
// In temple-bridge crate
struct BlobCache {
    cache: LruCache<BlobKey, Vec<u8>>,
}

impl BlobCache {
    fn get_or_compile(&mut self, expression: &str) -> &[u8] {
        // Returns cached blob or compiles, caches, and returns
    }
}
```

LRU eviction. Most expressions repeat across requests — cache hit rate > 99% in steady state.

---

## What Temple Compiles

### 1. Compute Section Expressions

```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  discount: "$compute.subtotal * 0.1"
  tax: "($compute.subtotal - $compute.discount) * 0.18"
  total: "$compute.subtotal - $compute.discount + $compute.tax"
```

Each expression compiles to one blob. The engine evaluates them in dependency order (DAG — subtotal before discount before tax before total).

### 2. Condition Expressions

```yaml
condition:
  and:
    - gt: ["$cart.total", 0]
    - not: "$user.banned"
```

Compiles to a boolean-returning blob.

### 3. Component HTML Templates

```yaml
# components.yaml
components:
  product-card:
    template: |
      <div class="card">
        <img src="{{ input.image }}" />
        <h3>{{ input.name }}</h3>
        <span class="price">{{ input.price }}</span>
      </div>
```

The `{{ input.* }}` placeholders compile to temple expressions inside the HTML template blob. At render time, the blob executes with component props as input, producing the final HTML string.

### 4. $Reference Resolution

Every `$reference` in YAML is a temple expression:

| YAML | Temple expression |
|------|-------------------|
| `$product.name` | `input.product.name` |
| `$cart.items.sum(price)` | `sum(input.cart.items, \|i\| i.price)` |
| `$compute.total` | (resolved from compute DAG) |
| `"Hello, $user.name!"` | `concat("Hello, ", input.user.name, "!")` |

---

## Temple-Bridge Crate

The `temple-bridge` crate sits between `engine-core` and `temple-dsl`:

```
engine-core                temple-bridge              temple-dsl
──────────                 ─────────────              ──────────
Parse YAML → PageTree  →  Translate $refs  →         Compile to blob
                           to temple exprs

Render request:
  Resolve providers     →  Execute blobs    →         Blob execution
  Build view tree       ←  Return results   ←         Return values
```

### Responsibilities

| engine-core | temple-bridge | temple-dsl |
|-------------|---------------|------------|
| YAML parsing | $ref → temple translation | Compile expressions |
| View tree structure | Blob caching (LRU) | Execute CBOR blobs |
| Type ID assignment | Dependency ordering (DAG) | Exact decimal math |
| Serialization (HTML/FB/JSON) | Component template rendering | Collection methods |
| Handler dispatch | | When guards |

---

## Temple Is Optional

Temple-dsl is a dependency of engine-core. But consumers choose how much to use:

### Full temple (default)

Use `compute:` section, `$references` with collection methods, conditional `when` blocks. The engine compiles and executes temple behind the scenes.

```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  tier:
    when:
      - "$user.spend >= 1000": "gold"
      - else: "standard"
```

### No temple (raw handlers only)

Skip `compute:` entirely. Use YAML for layout + built-in actions only. Write all logic in Rust/TS handler functions.

```yaml
# Pure layout, no compute
regions:
  main:
    components:
      - type: text
        props:
          content: "Hello"

interactions:
  - on: btn.click
    handler: myCustomLogic
    args:
      id: "123"
```

```rust
// All logic in handler
server.register_raw_handler("myCustomLogic", |ctx| async {
    let result = do_complex_stuff(ctx.args).await;
    ctx.refresh("main");
    Ok(result)
});
```

Both work. A project can mix both — temple for simple derived values, raw handlers for complex business logic.

---

## Temple WASM (Optional, Web Editor)

For web-based YAML editors, temple-dsl compiles to WASM to provide:

- **Intellisense** — autocomplete for `$references` and collection methods
- **Validation** — check expressions before deploy
- **Format-on-save** — normalize YAML expressions

```typescript
import { validate, suggest } from '@orrery/temple-wasm'

const result = validate('$cart.items.sum(price * quantity)')
// { valid: true }

const completions = suggest('$cart.items.', schema)
// ['sum', 'filter', 'map', 'any', 'all', 'min', 'max', 'sort', 'len']
```

This is optional. The server validates everything anyway — WASM just gives faster feedback in the editor.

---

## Performance Impact

| Without temple blobs | With temple blobs |
|---------------------|-------------------|
| Parse expression every request | Parse once at deploy |
| ~50-200µs per expression | ~0.5-5.5µs per expression |
| 50 expressions = 2.5-10ms | 50 expressions = 0.025-0.275ms |
| CPU scales linearly with requests | CPU nearly flat after warmup |

Temple's blob model is what makes server-side rendering fast enough for real-time interactions. Without it, re-rendering on every data change would be too slow.

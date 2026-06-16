# Libraries & Crate Structure

All packages needed to run Orrery across platforms.

---

## Library Overview

| # | Package | Language | Required | Purpose |
|---|---------|----------|----------|---------|
| 1 | `@orrery/server` | Rust + NAPI-RS | Yes (backend) | YAML parse, temple render, component → HTML rewrite, view tree build, serialize |
| 2 | `temple-dsl` | Rust (dependency) | Yes (internal) | Expression engine, blob compile/render |
| 3 | `@orrery/web` | TypeScript | Yes (web) | innerHTML, events, virtual scroll, built-in actions |
| 4 | `@orrery/ios` | Swift | Yes (iOS) | FlatBuffers/JSON decode, UIKit rendering, built-in actions |
| 5 | `@orrery/android` | Kotlin | Yes (Android) | FlatBuffers/JSON decode, Compose rendering, built-in actions |
| 6 | `temple-dsl` (WASM) | Rust -> WASM | Optional | Web editor intellisense, format-on-save |

## What the Developer Installs

**Web app:**
```
npm install @orrery/web
```
1 package. Includes built-in actions and event handling.

**Backend (Node.js):**
```
npm install @orrery/server
```
1 package. Rust binary via NAPI-RS. Includes temple-dsl internally.

**Backend (pure Rust):**
```toml
[dependencies]
orrery-server = "1.0"
```

**iOS:**
```swift
// Swift Package Manager
.package(url: "https://github.com/coderrdodo/orrery-ios", from: "1.0.0")
```
1 package. Includes renderer + built-in actions.

**Android:**
```kotlin
// build.gradle
implementation("com.orrery:android:1.0.0")
```
1 package. Includes renderer + built-in actions.

## Crate Structure

```
source-crafter/
├── crates/
│   ├── engine-core/          # Core engine: YAML parsing, view tree, serialization
│   │   ├── Cargo.toml        # depends on temple-dsl, serde, serde_yaml, flatbuffers
│   │   └── src/
│   │       ├── lib.rs         # Public API: parse, render, serialize
│   │       ├── parser/        # YAML → PageTree
│   │       ├── resolver/      # $reference resolution, condition evaluation
│   │       ├── tree/          # View tree construction, integer type ID assignment
│   │       ├── serializer/    # HTML, FlatBuffers, JSON output
│   │       └── generated/     # Types from type-crafter (spec.yaml)
│   │
│   ├── temple-bridge/        # Bridge between engine-core and temple-dsl
│   │   ├── Cargo.toml        # depends on engine-core, temple-dsl
│   │   └── src/
│   │       ├── lib.rs         # Convert $references → temple expressions
│   │       ├── compiler.rs    # Compile YAML compute/conditions to temple blobs
│   │       └── cache.rs       # Blob cache (in-memory, LRU)
│   │
│   └── handlers/             # Handler registration and execution
│       ├── Cargo.toml        # depends on engine-core, temple-bridge
│       └── src/
│           ├── lib.rs         # Handler registry, action execution
│           ├── registry.rs    # register_handler(name, config)
│           ├── actions.rs     # Built-in actions: fetch, refresh, navigate, notify
│           └── extensions.rs  # Platform extension dispatch
│
├── packages/
│   ├── server/               # @orrery/server (Node.js NAPI-RS bindings)
│   │   ├── package.json
│   │   └── src/
│   │       └── lib.rs        # napi-rs exports: createServer, render, registerHandler
│   │
│   ├── web/                  # @orrery/web (TypeScript client)
│   │   ├── package.json
│   │   └── src/
│   │       ├── index.ts       # mount(), public API
│   │       ├── renderer.ts    # innerHTML, event binding
│   │       ├── scroll.ts      # Virtual scroll implementation
│   │       ├── actions.ts     # Built-in action handlers (show/hide/toggle/navigate/notify)
│   │       ├── diff.ts        # Apply server diffs to existing DOM
│   │       └── cache.ts       # Local cache (ServiceWorker / localStorage)
│
├── types/
│   └── spec.yaml             # Single source of truth for types (type-crafter)
│
└── docs/                     # Documentation (this folder)
```

## Why Platform Adapters Are Separate Packages

**Engine-core** is pure Rust. It knows nothing about UIKit, Compose, DOM, or any platform.

**Platform adapters** (`@orrery/web`, `@orrery/ios`, `@orrery/android`) are written in platform-native languages (TypeScript, Swift, Kotlin). They can't live inside a Rust crate.

Separation also means:
- iOS adapter ships as a Swift Package — no Rust toolchain needed on the iOS developer's machine (they get a pre-built binary)
- Android adapter ships as a Gradle dependency — same
- Web adapter ships as an npm package — same
- Engine-core can be updated independently of adapters
- Adapters can be versioned per platform release cycle

## Temple-DSL as Optional

Temple-dsl is a dependency of `engine-core`. But the consumer can choose whether to use temple features:

**Using temple (default):**
```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  discount: "$compute.subtotal * 0.1"
```
Server compiles to temple blobs and evaluates automatically.

**Without temple (raw handler functions):**
```rust
server.register_handler("addToCart", |ctx| {
    let product_id = ctx.args.get("product_id");
    let result = my_api::add_to_cart(product_id).await;
    ctx.refresh("cart");
    ctx.notify("Added!");
    result
});
```
Developer writes Rust/TS handler functions directly. No temple involved. The YAML still works for layout and built-in actions — just the compute/transform layer is bypassed.

Both approaches coexist. A project can use temple for some handlers and raw functions for others.

# Orrery Architecture

YAML-driven cross-platform UI engine. Pre-rendered output, thin native clients.

---

## Overview

```
Developer writes YAML + references component libraries via `use:` + registers handlers
                    |
                    v
        ┌───────────────────────────────┐
        │       @orrery/server          │
        │  (Rust + embedded QuickJS     │
        │   + esbuild for SSR)          │
        │                               │
        │  COMPILE TIME (once):         │
        │  1. Resolve `use:` refs       │
        │     (components, icons,       │
        │      theme, fonts, layout)    │
        │  2. SSR extract component     │
        │     HTML from frameworks      │
        │  3. Compile to temple blobs   │
        │                               │
        │  PRE-RENDER (on data change): │
        │  4. Parse YAML                │
        │  5. Fetch provider data       │
        │  6. Evaluate compute section  │
        │  7. Evaluate conditions       │
        │  8. Resolve all $references   │
        │  9. Build view tree           │
        │ 10. Render to .html/.css/.js  │
        │     (web) or FlatBuffers      │
        │     (mobile)                  │
        └──────────┬────────────────────┘
                   │
        StorageAdapter (consumer's choice)
        S3 / Redis / R2 / filesystem
                   │
            CDN / Storage
                   │
       ┌───────────┼───────────┐
       v           v           v
  @orrery/web  @orrery/ios  @orrery/android
  (TypeScript)  (Swift)      (Kotlin)
       │           │           │
   Fetch .html  Fetch FB     Fetch FB
   innerHTML    UIKit        Compose
   + events     views        composables
   + virtual    + UIColV     + LazyColumn
     scroll
```

## What the Server Does

The server is **pure Rust** (~25-30MB binary). It embeds QuickJS (~300KB) for JS execution and esbuild (~8MB) for bundling. No Node.js. No npm CLI.

All logic runs at **compile/pre-render time** — not at request time:

- **Resolve `use:` references** — download component libraries, icon sets, themes, fonts, layouts via HTTP from npm registry, URLs, or local paths
- **SSR extract components** — three tiers: HTML templates (direct), Web Components (custom element tags), Framework SSR (esbuild bundles framework + components → QuickJS executes → HTML string)
- **Temple compilation** — compile HTML templates, compute expressions, conditions to CBOR blobs
- **Pre-rendering** — execute blobs with provider data → produce final .html, .css, .js files per page
- **Storage** — store pre-rendered files via consumer-configured StorageAdapter
- **Manifest generation** — generate versioned route manifest (page → CDN URL mapping)
- **Handler execution** — process handler calls when user interacts (the only runtime server responsibility)

### Component Extraction Tiers

| Tier | Input | Extraction | Cost |
|------|-------|-----------|------|
| **1. HTML Templates** | `.html` files, inline templates | None — use directly | 0ms |
| **2. Web Components** | Custom element JS files (`<sui-card>`) | Load JS bundle, template = `<tag-name>` | ~10ms |
| **3. Framework SSR** | `.jsx`, `.svelte`, `.vue` source files | esbuild bundle → QuickJS execute → HTML | ~3-8s first run, ~5ms cached |

Extraction results are cached per library version. Changing YAML does NOT re-extract components — only changing the component library version triggers re-extraction.

The server outputs platform-specific files:

| Platform | Output | Where Stored |
|----------|--------|-------------|
| Web | `.html` + `.css` + `.js` | StorageAdapter → CDN |
| iOS (prod) | FlatBuffers binary | StorageAdapter → CDN |
| iOS (debug) | JSON | StorageAdapter → CDN |
| Android (prod) | FlatBuffers binary | StorageAdapter → CDN |
| Android (debug) | JSON | StorageAdapter → CDN |

## What the Client Does

The client is thin. It only handles rendering and interaction capture:

- **Fetch pre-rendered files** from CDN (no server round-trip for page loads)
- **Web:** `innerHTML` for HTML, CSS loaded via `<link>`
- **Mobile:** FlatBuffers zero-copy decode → native view construction
- **Attach event listeners** — from interactions.js definitions
- **Execute built-in actions** — show, hide, toggle, navigate, notify (no server round-trip)
- **Navigate via manifest** — client knows all page URLs, fetches next page directly from CDN
- **Prefetch linked pages** — `<link rel="prefetch">` for pages reachable from current page
- **Forward handler calls** — send event + args to server, apply returned diff
- **Virtual scroll** — for lists exceeding threshold (200 web, 100 mobile)

The client never evaluates conditions, resolves references, or computes values. That's all done at pre-render time.

## Storage Adapter

The consumer decides where pre-rendered files are stored:

```rust
// Trait the consumer implements (or uses a built-in)
trait StorageAdapter {
    async fn store(&self, key: &str, data: &[u8], content_type: &str) -> Result<String>;
    async fn fetch(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> bool;
}
```

```typescript
// Configuration
const server = createOrreryServer({
  storage: new S3Adapter({ bucket: "my-app", region: "ap-south-1" }),  // production
  // storage: new FileSystemAdapter({ path: "./cache" }),              // development
  // storage: new RedisAdapter({ url: "redis://localhost:6379" }),     // alternative
  pages: "./pages",
  components: "./components"
})
```

Orrery ships `FileSystemAdapter` and `MemoryAdapter`. Consumer brings their own for production.

## Route Manifest

The manifest maps page names to CDN URLs. Stored in same storage, fetched by client on first load:

```json
{
  "version": "v42",
  "pages": {
    "home": "/pages/home/index.html",
    "products": "/pages/products/index.html",
    "cart": "/pages/cart/index.html"
  },
  "assets": {
    "components": "/assets/components.css",
    "theme": "/assets/theme.css",
    "runtime": "/assets/orrery-runtime.js"
  }
}
```

Client stores this in `sessionStorage` + JS memory. Navigation uses manifest for instant page lookup — no server round-trip.

## Integer Type IDs

Components use integer IDs instead of string names for fast lookup:

```
Server assigns:  type_id 0 = "text", 1 = "button", 2 = "input", 3 = "image", ...

Client adapter:  constructors[0] = UILabel    (iOS)
                 constructors[1] = UIButton   (iOS)
                 constructors[2] = UITextField (iOS)
                 constructors[3] = UIImageView (iOS)
```

Array index lookup vs HashMap string lookup. Measurable difference at scale.

## Temple-DSL Integration

Temple is an internal engine detail. YAML authors never see temple syntax.

```
YAML author writes:     quantity: "$event.quantity"
                        subtotal: "$cart.items.sum(price * quantity)"

Server internally:      Converts to temple expression
                        Compiles to CBOR blob (once, at deploy)
                        Renders against provider data (~1 microsecond)

YAML author sees:       Resolved value in the rendered output
```

Temple provides:
- Exact decimal arithmetic (no floating-point errors for pricing)
- Non-Turing-complete evaluation (always terminates)
- Pre-compiled blobs (compile once, render many)
- `when` guards for conditional compute
- Collection operations (sum, filter, map, fold, any, all)

Temple is optional. The consumer can register raw handler functions server-side instead.

## Format Negotiation (Mobile Only)

Mobile adapters negotiate format via `Accept` header:

```
Accept: application/x-flatbuffers    →  FlatBuffers binary (prod)
Accept: application/json             →  JSON (debug)
```

Web always receives `.html` files — no negotiation needed.

Per-request override via query param: `?format=json` for debugging.

## Diff Updates

When data changes (handler triggers a refresh), the server doesn't re-render the entire page:

```
1. Client sends: refresh("cart")
2. Server re-fetches cart provider data
3. Server re-renders only cart-dependent regions (using stored blobs)
4. Server diffs old view tree vs new view tree
5. Server sends only changed nodes
6. Client patches the existing UI
7. Server stores updated pre-rendered files for next visitor
```

## No PR For UI Changes

Pre-rendered files are generated artifacts in storage, not source code:

```
YAML change → engine re-renders → stores new files → CDN serves new version
(seconds, no PR, no deploy, no CI)
```

Only handler registration (backend logic) and new component definitions need code changes.

# Orrery Architecture

YAML-driven cross-platform UI engine. Server-side rendering, thin native clients.

---

## Overview

```
Developer writes YAML + registers handlers + registers components
                    |
                    v
        ┌───────────────────────┐
        │    @orrery/server     │
        │       (Rust)          │
        │                       │
        │  1. Parse YAML        │
        │  2. Load temple blobs │
        │  3. Fetch provider    │
        │     data              │
        │  4. Evaluate compute  │
        │  5. Evaluate          │
        │     conditions        │
        │  6. Resolve all       │
        │     $references       │
        │  7. Build view tree   │
        │  8. Serialize output  │
        └──────────┬────────────┘
                   │
            Network / Cache
                   │
       ┌───────────┼───────────┐
       v           v           v
  @orrery/web  @orrery/ios  @orrery/android
  (TypeScript)  (Swift)      (Kotlin)
       │           │           │
   innerHTML    UIKit       Compose
   + events     views       composables
   + virtual    + UIColV    + LazyColumn
     scroll
```

## What the Server Does

The server is the brain. It handles all logic:

- **YAML parsing** — reads page template, style tokens, data providers, compute section, regions, interactions
- **Temple rendering** — resolves `$references`, evaluates conditions, computes derived values using pre-compiled temple blobs
- **View tree construction** — builds the component tree with integer type IDs and fully resolved props
- **Serialization** — outputs HTML (web), FlatBuffers (mobile prod), or JSON (mobile debug)

The server knows nothing about platforms. It outputs a platform-agnostic description. The format differs:

| Platform | Output format | Why |
|----------|--------------|-----|
| Web | HTML string | Browser parses HTML natively at 152 MB/s |
| iOS (prod) | FlatBuffers | Zero-copy decode, < 1ms for 500KB |
| iOS (debug) | JSON | Human-readable for debugging |
| Android (prod) | FlatBuffers | Zero-copy decode, < 1ms for 500KB |
| Android (debug) | JSON | Human-readable for debugging |

## What the Client Does

The client is thin. It handles only rendering and interaction capture:

- **Receive output** from server (HTML or FlatBuffers/JSON)
- **Map to native views** — type ID array lookup, not string HashMap
- **Attach event listeners** — for interactions defined in the YAML
- **Execute built-in actions** — show, hide, toggle, navigate, notify (no server round-trip)
- **Forward handler calls** — send event + args to server, apply returned diff
- **Virtual scroll** — for lists exceeding threshold (200 web, 100 mobile)

The client never evaluates conditions, resolves references, or computes values. That's all server-side.

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

Temple is optional. The consumer can choose to register raw handler functions server-side instead of using YAML handler declarations.

## Format Negotiation

The client sends an `Accept` header. The server responds accordingly:

```
Accept: text/html                    →  HTML string (web)
Accept: application/x-flatbuffers    →  FlatBuffers binary (mobile prod)
Accept: application/json             →  JSON (mobile debug)
```

Server config can override:

```rust
OrreryServer::new()
    .format(Format::Auto)           // FlatBuffers in prod, JSON in debug
    .format(Format::FlatBuffers)    // always binary
    .format(Format::Json)           // always JSON
```

Per-request override via query param: `?format=json` for debugging.

## Diff Updates

When data changes (handler triggers a refresh), the server doesn't re-send the entire page:

```
1. Client sends: refresh("cart")
2. Server re-fetches cart provider data
3. Server re-renders only the cart-dependent regions
4. Server diffs old view tree vs new view tree
5. Server sends only changed nodes
6. Client patches the existing UI
```

This minimizes network transfer and avoids full re-renders on every interaction.

## Caching

View trees are cached locally on the client:

- First load: server response cached to disk (keyed by YAML filename + data version hash)
- Subsequent loads: render from cache immediately, check server in background (stale-while-revalidate)
- Cache invalidation: server sends version header, client compares

With caching, the network overhead drops to 0ms for repeat visits.

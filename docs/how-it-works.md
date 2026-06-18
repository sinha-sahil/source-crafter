# How It Works

The complete technical flow — from YAML to browser.

---

## The Big Picture

```
YAML + Component Library
        │
        ▼
Orrery Server (compile time)
  1. Resolve `use:` references (components, icons, themes, fonts)
  2. Extract HTML templates from framework components via SSR
  3. Compile templates to temple blobs
  4. Fetch data from providers
  5. Execute blobs with data → produce .html, .css, .js per page
  6. Store everything via StorageAdapter (S3, Redis, filesystem, etc.)
  7. Generate route manifest
        │
        ▼
CDN / Storage
  Pre-rendered .html, .css, .js files sitting ready
        │
        ▼
Browser
  Fetches files from CDN → injects HTML → binds events → done
  No framework runtime. No server round-trip for page loads.
```

---

## The `use:` Section

At the top of any YAML page, you declare what external resources you need:

```yaml
page: home

use:
  components: "@shadcn/ui"
  icons: "lucide"
  theme: "https://cdn.myapp.com/themes/dark.yaml"
  layout: "dashboard"
  fonts:
    - "Inter"
    - "JetBrains Mono"
```

### What Can Be Referenced

| Type | What It Gives You | Examples |
|------|-------------------|----------|
| `components` | UI elements (buttons, cards, dialogs) | `@shadcn/ui`, `@radix/ui`, custom library |
| `icons` | Icon sets | `lucide`, `phosphor`, `heroicons`, custom SVGs |
| `theme` | Colors, spacing, typography, border-radius tokens | `material-dark`, brand theme URL |
| `layout` | Pre-built page structures | `dashboard`, `e-commerce`, `blog` |
| `fonts` | Typography | `Inter`, Google Fonts URL, custom font URL |

### Ways To Reference

```yaml
# npm package name
use:
  components: "@shadcn/ui"

# URL (any hosted source)
use:
  components: "https://unpkg.com/@shadcn/ui"

# GitHub repo
use:
  components: "github:shadcn/ui"

# Local file path
use:
  components: "./my-components.yaml"

# Multiple sources (last wins on conflict)
use:
  components:
    - "@shadcn/ui"
    - "https://cdn.company.com/internal-components"
    - "./overrides"

# Cherry-pick specific components
use:
  components:
    from: "@shadcn/ui"
    pick: [button, card, dialog, input, avatar]

# Inline HTML template (no external source)
use:
  components:
    product-card:
      template: '<div class="card"><h3>{{ input.name }}</h3><span>{{ input.price }}</span></div>'
      style: '.card { border: 1px solid #30363d; padding: 16px; }'
```

---

## How Component Extraction Works (SSR)

SSR stands for Server-Side Rendering. Every UI framework (React, Svelte, Vue) has a built-in function that takes a component and returns the HTML string it would produce — without needing a browser.

| Framework | Function | What It Does |
|-----------|----------|-------------|
| React | `renderToStaticMarkup(component)` | Component in → HTML string out |
| Svelte | `Component.render(props)` | Component in → `{ html: string }` out |
| Vue | `renderToString(app)` | Component in → HTML string out |

The Orrery server is **pure Rust**. It doesn't run Node.js. Instead, it embeds a lightweight JavaScript runtime (QuickJS via the `rquickjs` crate) and uses esbuild to bundle framework code. This happens at compile time only — never at request time.

### Three Tiers of Component Libraries

Not all component libraries need the same extraction approach:

| Tier | What It Ships | Extraction | Examples |
|------|--------------|-----------|----------|
| **Tier 1: HTML Templates** | `.html` files or template strings | Zero extraction needed — use directly | Custom components, Orrery-native libs |
| **Tier 2: Web Components** | Custom element JS files | Load JS bundle, use `<tag-name>` in template | `@juspay/svelte-ui-components` (ships `<sui-card>`, `<sui-button>`) |
| **Tier 3: Raw Framework** | `.jsx`, `.svelte`, `.vue` source files | Full SSR extraction via QuickJS + esbuild | Most npm component libraries |

The server auto-detects which tier a package falls into by examining its `package.json` and file structure.

### How the Rust Server Runs JavaScript

The server embeds two tools:

1. **esbuild** (~8MB binary) — Bundles framework code + component source into a single `.js` file. Resolves all imports. Takes ~50-100ms.
2. **QuickJS** via `rquickjs` crate (~300KB) — Executes the bundled JS. No V8, no Node.js. Sub-millisecond startup.

```
Component library + framework compiler
        │
        ▼
esbuild bundles everything → single bundle.js (no external deps)
        │
        ▼
QuickJS executes bundle.js → calls framework SSR function → HTML string
        │
        ▼
Replace markers → HTML template → temple blob (pure Rust from here)
```

### How npm Packages Are Downloaded (No npm CLI)

The server downloads packages directly via HTTP — no `npm` CLI, no `node_modules`:

```
1. GET https://registry.npmjs.org/@shadcn/ui
   → JSON with version info, tarball URL

2. GET https://registry.npmjs.org/@shadcn/ui/-/ui-1.0.0.tgz
   → Download tarball

3. Extract to .orrery/cache/@shadcn/ui/1.0.0/
   → Read package.json, detect framework, find component files
```

This is plain HTTP. Any language can do it. The npm registry is just a REST API.

### Step-By-Step: React Component (Tier 3)

Given a React button from `@shadcn/ui`:

```jsx
// Inside the @shadcn/ui package
function Button({ text, variant, disabled }) {
  return <button className={cn("btn", variant)} disabled={disabled}>{text}</button>
}
```

**Step 1 — Server generates an entry script:**

```rust
// Rust generates a JS file that imports the framework + component
fn generate_react_entry(components: &[ComponentFile]) -> String {
    format!(r#"
        import React from 'react';
        import {{ renderToStaticMarkup }} from 'react-dom/server';
        import {{ Button }} from '@shadcn/ui';

        export function extractAll() {{
            return {{
                button: {{
                    html: renderToStaticMarkup(
                        React.createElement(Button, {{
                            text: '___ORRERY_PROP_text___',
                            variant: '___ORRERY_PROP_variant___',
                            disabled: '___ORRERY_PROP_disabled___'
                        }})
                    )
                }}
            }};
        }}
    "#)
}
```

**Step 2 — esbuild bundles entry + React + component into ONE file:**

```rust
// esbuild resolves all imports, inlines everything
Command::new("./esbuild")
    .args(["entry.js", "--bundle", "--format=esm", "--platform=neutral",
           "--outfile=.orrery/cache/bundle.js"])
    .status()?;
// Output: bundle.js (~150KB) — React + component, zero external deps
```

**Step 3 — QuickJS executes the bundle:**

```rust
use rquickjs::{Runtime, Context};

let runtime = Runtime::new()?;
let context = Context::full(&runtime)?;
context.with(|ctx| {
    let code = std::fs::read_to_string(".orrery/cache/bundle.js")?;
    ctx.eval::<(), _>(&code)?;
    let json: String = ctx.eval("JSON.stringify(extractAll())")?;
    // json = { "button": { "html": "<button class=\"btn ___ORRERY_PROP_variant___\">..." } }
});
```

Result: `<button class="btn ___ORRERY_PROP_variant___" disabled="___ORRERY_PROP_disabled___">___ORRERY_PROP_text___</button>`

**Step 4 — Replace markers with temple placeholders:**

```rust
let template = html.replace("___ORRERY_PROP_text___", "{{ input.text }}")
                   .replace("___ORRERY_PROP_variant___", "{{ input.variant }}")
                   .replace("___ORRERY_PROP_disabled___", "{{ input.disabled }}");
// Result: '<button class="btn {{ input.variant }}" disabled="{{ input.disabled }}">{{ input.text }}</button>'
```

**Step 5 — Compile to temple blob (pure Rust):**

```rust
let blob = temple_dsl::compile(&template);
// blob = CBOR bytes (compiled template, ready to execute with any props)
```

**Step 6 — Extract CSS:**

The engine also captures styles from the component (CSS modules, styled-components output, Tailwind classes) and collects them into a single CSS file.

**Done.** React is thrown away. It never runs again. It never goes to the browser.

### Step-By-Step: Svelte Component (Tier 3)

Svelte packages often ship raw `.svelte` source files (not compiled JS). The Svelte compiler itself must run to produce SSR-capable code.

Real-world example: `@juspay/svelte-ui-components` v2.68.0 ships 67 raw `.svelte` files using Svelte 5 runes (`$props()`, `$derived()`, `$bindable()`).

```
What the server does:
1. Download @juspay/svelte-ui-components (component source)
2. Download svelte (the compiler package)
3. Generate entry.js that imports svelte/compiler + embeds .svelte sources
4. esbuild bundles everything → bundle.js (~500KB-1MB)
5. QuickJS executes bundle.js:
   a. svelte.compile(source, { generate: 'server' }) → JS code with render()
   b. Execute compiled JS → get render() function
   c. render({ text: "___PROP___" }) → HTML string
6. Replace markers → HTML template → temple blob
```

```rust
// The generated entry script for Svelte
fn generate_svelte_entry(component_sources: &[(&str, String)]) -> String {
    let mut script = String::from("import { compile } from 'svelte/compiler';\n");

    for (name, source) in component_sources {
        let escaped = source.replace('\\', "\\\\").replace('`', "\\`");
        script.push_str(&format!(r#"
            const {name}_compiled = compile(`{source}`, {{ generate: 'server', name: '{name}' }});
            const {name}_module = new Function('return ' + {name}_compiled.js.code)();
            const {name}_result = {name}_module.default.render({{
                text: '___ORRERY_PROP_text___',
                title: '___ORRERY_PROP_title___'
            }});
        "#, name = name, source = escaped));
    }

    script
}
```

### Step-By-Step: Web Components (Tier 2)

Some libraries ship Web Component wrappers with custom element tags (e.g., `<sui-card>`, `<sui-button>`). These are simpler — no SSR extraction needed:

```
1. Download the Web Component JS bundle
2. Template = just the custom element tag:
   <sui-button text="{{ input.text }}"></sui-button>
3. Ship the WC bundle alongside orrery-runtime.js
4. Browser natively understands custom element tags
```

This is the easiest path for component library authors who want Orrery compatibility.

### Prop Detection Without TypeScript Types

If the package doesn't ship `.d.ts` files, the engine detects props by diffing:

```javascript
// Render with no props
const emptyHtml = render(Component, {})
// '<button class="btn"></button>'

// Render with known test values
const testHtml = render(Component, {
  text: "TEST_ab12_text",
  variant: "TEST_ab12_variant"
})
// '<button class="btn TEST_ab12_variant">TEST_ab12_text</button>'

// Diff → "TEST_ab12_text" appears inside the tag → text goes there
// Diff → "TEST_ab12_variant" appears in class → variant goes there
```

### Extraction Caching

Component extraction runs **once per library version**. Results are cached:

```
.orrery/cache/
├── @shadcn/ui/1.0.0/
│   ├── templates/
│   │   ├── button.html
│   │   ├── card.html
│   │   └── dialog.html
│   ├── styles/
│   │   └── components.css
│   └── meta.json           ← version, timestamp, tier used
├── @juspay/svelte-ui-components/2.68.0/
│   └── ...
└── lucide/1.0.0/
    └── ...
```

Next server startup with the same library version: cache hit, skip extraction entirely (~5ms to load blobs from disk vs ~3-8 seconds for full extraction).

---

## How Icons Work

```yaml
use:
  icons: "lucide"
```

Icon libraries are simpler — they're just SVG strings.

```
Engine resolves "lucide"
    │
    ▼
Downloads/imports the package
lucide exposes: { "shopping-cart": "<svg>...</svg>", "home": "<svg>...</svg>", ... }
    │
    ▼
Registers each icon as a component template:
  "icon:shopping-cart" → template: '<svg xmlns="..." viewBox="...">...</svg>'
  "icon:home" → template: '<svg xmlns="..." viewBox="...">...</svg>'
```

Usage in YAML:
```yaml
components:
  - type: icon
    props:
      name: "shopping-cart"
      size: 24
      color: "$style.primary"
```

Engine looks up `icon:{name}` → injects the SVG inline → compiled to blob.

---

## How Themes Work

```yaml
use:
  theme: "https://cdn.myapp.com/themes/dark.yaml"
```

A theme file is just a YAML/JSON file with design tokens:

```yaml
# dark.yaml
colors:
  primary: "#238636"
  accent: "#58a6ff"
  surface: "#161b22"
  text: "#e6edf3"
  border: "#30363d"
  danger: "#f85149"

spacing:
  xs: 4
  sm: 8
  md: 16
  lg: 24
  xl: 32

radius:
  sm: 4
  md: 8
  lg: 16

typography:
  font-family: "Inter, system-ui, sans-serif"
  heading-weight: 600
  body-size: 14
```

Engine fetches this file → merges into `$style.*` tokens → available throughout the page:

```yaml
# These now resolve from the theme
style:
  background: "$style.surface"      # → #161b22
  color: "$style.text"              # → #e6edf3
  padding: "$style.spacing.md"     # → 16
```

The theme generates CSS custom properties:

```css
/* Generated from theme */
:root {
  --color-primary: #238636;
  --color-accent: #58a6ff;
  --color-surface: #161b22;
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --radius-md: 8px;
  --font-family: Inter, system-ui, sans-serif;
}
```

---

## How Fonts Work

```yaml
use:
  fonts:
    - "Inter"
    - "JetBrains Mono"
    - "https://cdn.myapp.com/fonts/CustomFont.woff2"
```

Engine resolves each font:

| Input | What Engine Does |
|-------|-----------------|
| `"Inter"` | Generates `<link>` to Google Fonts CDN |
| `"https://...CustomFont.woff2"` | Generates `@font-face` CSS rule |

Output in the page CSS:

```css
/* Generated from use.fonts */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700');
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500');

@font-face {
  font-family: 'CustomFont';
  src: url('https://cdn.myapp.com/fonts/CustomFont.woff2') format('woff2');
}
```

---

## How Layouts Work

```yaml
use:
  layout: "dashboard"
```

A layout is a pre-built region structure. Instead of writing the full region tree, you get a skeleton:

```yaml
# What "dashboard" layout provides:
# ┌──────────┬──────────────────────────────┐
# │          │         topbar               │
# │ sidebar  ├──────────────────────────────┤
# │          │                              │
# │          │         main                 │
# │          │                              │
# └──────────┴──────────────────────────────┘

# The user just fills in the regions:
regions:
  sidebar:
    components:
      - type: icon
        props: { name: "home" }
      - type: text
        props: { content: "Dashboard" }

  topbar:
    components:
      - type: text
        props: { content: "Welcome back" }

  main:
    components:
      - type: text
        props: { content: "Your content here" }
```

The layout defines the CSS grid/flex structure. The user just fills in the content.

---

## Pre-Rendering Pipeline

After resolving all `use:` references and compiling everything to blobs:

```
For each page:
━━━━━━━━━━━━━━

1. Load compiled blobs (component templates, compute expressions)
2. Fetch current data from providers
3. Execute blobs with data
4. Produce three files:

   index.html        ← Full rendered HTML (components + layout + data)
   page.css           ← Theme tokens + component styles + layout styles
   interactions.js    ← Event bindings (which element → which action)
```

### Storage Adapter

The consumer configures where files are stored:

```typescript
import { createOrreryServer, S3Adapter, FileSystemAdapter } from "@orrery/server"

// Production: store in S3 behind a CDN
const server = createOrreryServer({
  storage: new S3Adapter({ bucket: "my-app-pages", region: "ap-south-1" }),
  pages: "./pages",
  components: "./components"
})

// Development: store on local filesystem
const server = createOrreryServer({
  storage: new FileSystemAdapter({ path: "./cache/renders" }),
  pages: "./pages",
  components: "./components"
})
```

The `StorageAdapter` interface:

```rust
trait StorageAdapter {
    async fn store(&self, key: &str, data: &[u8], content_type: &str) -> Result<String>;
    async fn fetch(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> bool;
}
```

Orrery ships `FileSystemAdapter` and `MemoryAdapter` built-in. Consumer brings their own for production (S3, Redis, Cloudflare R2, Supabase Storage, etc.).

### What Gets Stored

```
storage/
├── manifest.json                    ← Route manifest (page → URL mapping)
├── assets/
│   ├── components.css               ← All component styles
│   ├── theme.css                    ← CSS custom properties from theme
│   └── orrery-runtime.js           ← @orrery/web (~5-8KB gzipped)
└── pages/
    ├── home/
    │   ├── index.html
    │   ├── page.css
    │   └── interactions.js
    ├── products/
    │   ├── index.html
    │   ├── page.css
    │   └── interactions.js
    └── cart/
        ├── index.html
        ├── page.css
        └── interactions.js
```

---

## The Route Manifest

```json
{
  "version": "v42",
  "pages": {
    "home": "/pages/home/index.html",
    "products": "/pages/products/index.html",
    "product-detail": "/pages/product-detail/index.html",
    "cart": "/pages/cart/index.html"
  },
  "assets": {
    "components": "/assets/components.css",
    "theme": "/assets/theme.css",
    "runtime": "/assets/orrery-runtime.js"
  }
}
```

On first page load, the browser receives this manifest. It's stored in:

| Where | Why |
|-------|-----|
| JS variable in memory | Instant access during navigation |
| `sessionStorage` | Survives page refresh, clears on tab close |
| ServiceWorker cache | Survives tab close (for PWA/offline) |

When the user navigates to another page, the browser already knows the URL from the manifest. It fetches the `.html` directly from CDN — no server round-trip.

---

## Browser Flow (Request Time)

### First Page Load

```
Browser hits https://myapp.com
        │
        ▼
Loads index.html (one static file, the shell):
  <!DOCTYPE html>
  <html>
  <head>
    <link rel="stylesheet" href="https://cdn/assets/components.css" />
    <link rel="stylesheet" href="https://cdn/assets/theme.css" />
    <script src="https://cdn/assets/orrery-runtime.js"></script>
  </head>
  <body>
    <div id="app"></div>
    <script>
      orrery.mount('#app', { cdn: 'https://cdn', startPage: 'home' })
    </script>
  </body>
  </html>
        │
        ▼
orrery-runtime.js (this is @orrery/web) does:
  1. Fetch manifest.json from CDN
  2. Store manifest in sessionStorage
  3. Fetch /pages/home/index.html from CDN
  4. Fetch /pages/home/interactions.js from CDN
  5. container.innerHTML = html  (browser parses at 152 MB/s)
  6. Bind events from interactions
  7. Prefetch pages linked from this page (background <link rel="prefetch">)
        │
        ▼
User sees the page. No framework. Just HTML + CSS + tiny JS for events.
```

### Navigation (Page 2, Page 3, ...)

```
User clicks "Products"
        │
        ▼
orrery-runtime intercepts the click
        │
        ▼
Looks up manifest: "products" → "/pages/products/index.html"
        │
        ▼
fetch("/pages/products/index.html")  ← from CDN, or browser cache if prefetched
        │
        ▼
container.innerHTML = newHtml
Bind new events
Prefetch linked pages
Update browser URL via history.pushState
        │
        ▼
User sees Products page. Feels instant (~10-30ms CDN latency).
If page was prefetched: 0ms. Already in browser cache.
```

---

## Why No PR Is Needed For UI Changes

The consumer's deployed app consists of:

```
index.html              ← static shell, never changes
orrery-runtime.js       ← from npm @orrery/web, updates rarely
components.css          ← regenerated only when component library changes
theme.css               ← regenerated only when theme changes
```

These are deployed once and rarely change.

The actual pages (`/pages/home/index.html`, etc.) are **generated files in storage**. They are NOT source code. They are NOT in the app's git repo. They are NOT part of the app's deploy pipeline.

```
Traditional flow:
  Want to change the homepage layout?
  → Edit HomePage.tsx
  → Create PR
  → Code review
  → Merge
  → CI builds the app
  → Deploy to production
  → 30 minutes to 2 hours

Orrery flow:
  Want to change the homepage layout?
  → Edit home.yaml
  → Engine re-renders home/index.html
  → Stores new file in S3
  → CDN serves new version
  → Done in seconds. No PR. No deploy. No CI.
```

The YAML files can be edited from:
- A CMS dashboard (non-technical people change UI)
- A git repo (if you want version control, but no app deploy needed — just re-render)
- A database (for A/B testing — render two versions, serve different ones)
- An admin panel (real-time UI editor)

### What Still Needs a PR

| Change | PR Needed? | Why |
|--------|-----------|-----|
| Change page layout | No | YAML change → re-render |
| Change text / copy | No | YAML change → re-render |
| Swap components | No | YAML change → re-render |
| Change theme / colors | No | Theme file change → re-render |
| Rearrange sections | No | YAML change → re-render |
| Add new handler (backend logic) | Yes | Code change in handler registration |
| Add new component to the library | Yes | New HTML template needs to be defined |
| Change server config | Yes | Code change |

UI changes = YAML only = no PR.
Backend logic changes = code = PR.

---

## Framework Dependencies (Auto-Downloaded)

The Orrery server needs framework packages to extract HTML from components (e.g., `react` + `react-dom` for React libraries, `svelte` for Svelte libraries). But the **developer doesn't install these manually**.

The server auto-downloads what it needs:

```
1. YAML declares: use: components: "@shadcn/ui"
2. Server downloads @shadcn/ui from npm registry (HTTP)
3. Reads package.json → detects React (peerDependencies: { "react": ">=18" })
4. Downloads react + react-dom from npm registry
5. esbuild bundles everything → bundle.js
6. QuickJS extracts HTML → done
7. React is never used again
```

No `npm install`. No `node_modules`. No Node.js. The server handles everything via HTTP downloads and caching.

- Using `@shadcn/ui` (React)? → Server auto-downloads React
- Using `@juspay/svelte-ui-components` (Svelte)? → Server auto-downloads Svelte compiler
- Using plain HTML templates? → Nothing to download

The framework is **only used at compile time** inside the embedded QuickJS runtime. It is **never shipped to the browser**. The browser only receives `.html`, `.css`, and the tiny `orrery-runtime.js`.

---

## CDN Cache Versioning

How the CDN ensures users always get fresh content without re-downloading unchanged files:

```
manifest.json       → Cache-Control: no-cache (always revalidated, ~500 bytes)
/pages/home/v42.html → Cache-Control: immutable (cached forever)
/pages/home/v43.html → Cache-Control: immutable (cached forever, NEW URL)
```

The manifest is tiny (~500 bytes) and always checked for freshness. Page files use versioned URLs — when content changes, a new version URL is created. Old URLs are still valid (cached forever), new URLs are new files. No cache invalidation needed.

```
YAML changes → server re-renders → stores /pages/home/v43.html → updates manifest version to v43
                                                                    │
Next user visit → fetches manifest (no-cache) → sees v43 (was v42) → fetches new page URLs
                                                                       │
Already-cached v42 files → still valid, just unused
New v43 files → fetched from CDN, cached forever
```

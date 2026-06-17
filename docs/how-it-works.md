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

These are just function calls on the server (Node.js). No browser involved.

### Step-By-Step: React Component

Given a React button from `@shadcn/ui`:

```jsx
// Inside the @shadcn/ui package
function Button({ text, variant, disabled }) {
  return <button className={cn("btn", variant)} disabled={disabled}>{text}</button>
}
```

**Step 1 — Import the component on the server:**

```javascript
const pkg = require("@shadcn/ui")
// pkg.Button = the component function
```

**Step 2 — Read the prop names:**

The engine reads the package's TypeScript types (`.d.ts` files) to find props:

```typescript
// @shadcn/ui/dist/Button.d.ts (ships with the npm package)
export interface ButtonProps {
  text: string;
  variant: "primary" | "secondary" | "ghost";
  disabled?: boolean;
}
```

Engine extracts: `["text", "variant", "disabled"]`

If no `.d.ts` exists, the engine can also look for an `orrery.json` manifest in the package, or detect props by rendering with test values (see below).

**Step 3 — Render with marker props:**

```javascript
import { renderToStaticMarkup } from "react-dom/server"

const html = renderToStaticMarkup(
  React.createElement(pkg.Button, {
    text: "___ORRERY_PROP_text___",
    variant: "___ORRERY_PROP_variant___",
    disabled: "___ORRERY_PROP_disabled___"
  })
)

// Result: '<button class="btn ___ORRERY_PROP_variant___" disabled="___ORRERY_PROP_disabled___">___ORRERY_PROP_text___</button>'
```

This is just calling React's "give me the HTML string" function. React runs for this one call, on the server, at compile time. Then it's never used again.

**Step 4 — Replace markers with temple placeholders:**

```javascript
const template = html.replace(/___ORRERY_PROP_(\w+)___/g, '{{ input.$1 }}')

// Result: '<button class="btn {{ input.variant }}" disabled="{{ input.disabled }}">{{ input.text }}</button>'
```

**Step 5 — Compile to temple blob:**

```javascript
const blob = templeCompile(template)
// blob = CBOR bytes (compiled template, ready to execute with any props)
```

**Step 6 — Extract CSS:**

The engine also captures styles from the component (CSS modules, styled-components output, Tailwind classes) and collects them into a single CSS file.

**Done.** The React component is now an HTML template compiled to a temple blob. React is thrown away. It never runs again. It never goes to the browser.

### Step-By-Step: Svelte Component

```svelte
<!-- Inside a Svelte component library -->
<script>
  export let text;
  export let variant = "primary";
</script>
<button class="btn {variant}">{text}</button>
```

```javascript
// Svelte components have a .render() method when imported in SSR mode
import Button from "@my-lib/Button.svelte"

const { html, css } = Button.render({
  text: "___ORRERY_PROP_text___",
  variant: "___ORRERY_PROP_variant___"
})

// html = '<button class="btn ___ORRERY_PROP_variant___">___ORRERY_PROP_text___</button>'
// css = '.btn { padding: 8px 16px; }'

// Same marker replacement → template → blob
```

### Step-By-Step: Vue Component

```vue
<!-- Inside a Vue component library -->
<template>
  <button :class="['btn', variant]">{{ text }}</button>
</template>
<script setup>
defineProps(['text', 'variant'])
</script>
```

```javascript
import { createSSRApp } from "vue"
import { renderToString } from "vue/server-renderer"
import Button from "@my-lib/Button.vue"

const app = createSSRApp(Button, {
  text: "___ORRERY_PROP_text___",
  variant: "___ORRERY_PROP_variant___"
})
const html = await renderToString(app)

// Same marker replacement → template → blob
```

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

## Framework Peer Dependencies

The Orrery server needs the framework's SSR function to extract HTML from components. These are optional peer dependencies — you only install the one matching your component library:

```json
{
  "peerDependencies": {
    "react": ">=18",
    "react-dom": ">=18",
    "svelte": ">=4",
    "vue": ">=3"
  },
  "peerDependenciesMeta": {
    "react": { "optional": true },
    "react-dom": { "optional": true },
    "svelte": { "optional": true },
    "vue": { "optional": true }
  }
}
```

- Using `@shadcn/ui` (React)? → `npm install react react-dom` on the server
- Using a Svelte library? → `npm install svelte` on the server
- Using plain HTML templates? → No framework needed at all

The framework is **only used at compile time** on the server to extract HTML. It is **never shipped to the browser**. The browser only receives `.html`, `.css`, and the tiny `orrery-runtime.js`.

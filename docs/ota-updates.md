# OTA Updates

How Orrery enables over-the-air UI updates without app store submissions.

---

## Why OTA Works

The Orrery adapter is a **static binary** that ships once to the App Store / Play Store. It's a data interpreter — it reads FlatBuffers from CDN and constructs native views. The adapter code never changes.

What changes is the **content** — pre-rendered FlatBuffers files on CDN. Those are data, not code.

```
Traditional native app:
  Change button color → edit Swift/Kotlin → build → submit to App Store
  → Apple review (24-48 hours) → users update app → they see the change
  Total: 2-7 days

Orrery app:
  Change button color → edit YAML → engine re-renders → new FlatBuffers on CDN
  → user opens app → adapter fetches new file → they see the change
  Total: seconds. No app store. No review. No user update.
```

---

## App Store Compliance

### Apple (App Store Review Guidelines 3.3.2)

> "An Application may not download or install executable code."

Orrery doesn't download executable code. It downloads **data** (FlatBuffers bytes) that the existing, already-reviewed adapter interprets. This is identical to:
- An app loading JSON from an API
- A news app loading article content
- A shopping app loading product listings
- Firebase Remote Config changing UI layout

The adapter's behavior doesn't change. It always does the same thing: read bytes → construct native views. Only the bytes change.

### Google Play

No restrictions on downloading interpreted content. Even more permissive than Apple.

---

## What Can Change Via OTA

| Can change (OTA, no app update) | Needs app store update |
|--------------------------------|----------------------|
| Page layout, ordering | New native component types |
| Text, colors, spacing, themes | New platform extensions (camera, biometrics) |
| Which components appear | Adapter version upgrade |
| Conditions (show/hide sections) | New built-in action types |
| Data bindings | |
| Interactions (which handler fires) | |
| Icons, fonts | |
| Adding entire new pages | |
| Removing pages | |
| Changing navigation flow | |

As long as the component types and extensions are already registered in the adapter, everything else is OTA.

---

## OTA Update Flow

```
APP FIRST LAUNCH:
━━━━━━━━━━━━━━━━
1. App starts (adapter code — static, from App Store)
2. Fetch manifest.json from CDN
3. Fetch start page FlatBuffers from CDN
4. Build native views → user sees the app

YAML CHANGES (NO APP UPDATE):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. Someone edits YAML (layout change, new page, text change)
2. Orrery server re-renders → stores new FlatBuffers on CDN
3. Manifest version bumps (v42 → v43)

NEXT TIME USER OPENS APP:
━━━━━━━━━━━━━━━━━━━━━━━━━
1. Adapter fetches manifest.json (Cache-Control: no-cache — always revalidated, ~500 bytes)
2. Sees version v43 (was v42) — page URLs have changed
3. Fetches new FlatBuffers from new versioned URLs (e.g., /pages/home/v43.fb)
4. Old v42 files still cached (immutable) but no longer referenced
5. Builds native views from new data
6. User sees updated UI — no app store update needed

BACKGROUND UPDATE (WHILE APP IS OPEN):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. Adapter periodically checks manifest (every 5 min, or on app foreground)
2. If version changed → fetch new page files
3. Apply diff to current screen (or full reload if structural change)
4. User sees update live, without restarting app
```

---

## Update Strategies

### iOS

```swift
// Strategy 1: Check on app launch
func application(_ application: UIApplication, didFinishLaunchingWithOptions...) {
    Task { await orreryAdapter.checkForUpdates() }
}

// Strategy 2: Check on foreground
func applicationWillEnterForeground(_ application: UIApplication) {
    Task { await orreryAdapter.checkForUpdates() }
}

// Strategy 3: Silent push notification triggers update
func application(_ application: UIApplication, didReceiveRemoteNotification...) {
    Task { await orreryAdapter.checkForUpdates() }
}

// Strategy 4: WebSocket for real-time updates
orreryAdapter.connectLiveUpdates(url: "wss://api.myapp.com/orrery/live")
```

### Android

```kotlin
// Strategy 1: Check on app launch
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        lifecycleScope.launch { orreryAdapter.checkForUpdates() }
    }
}

// Strategy 2: Check on resume
override fun onResume() {
    super.onResume()
    lifecycleScope.launch { orreryAdapter.checkForUpdates() }
}

// Strategy 3: WorkManager periodic check
val updateWork = PeriodicWorkRequestBuilder<OrreryUpdateWorker>(15, TimeUnit.MINUTES).build()
WorkManager.getInstance(context).enqueue(updateWork)

// Strategy 4: Firebase Cloud Messaging triggers update
class OrreryMessagingService : FirebaseMessagingService() {
    override fun onMessageReceived(message: RemoteMessage) {
        if (message.data["type"] == "orrery_update") {
            orreryAdapter.checkForUpdates()
        }
    }
}
```

---

## Version Check Logic

```swift
// iOS
public func checkForUpdates() async {
    let newManifestData = try await fetch(path: "/manifest.json")
    let newManifest = try JSONDecoder().decode(RouteManifest.self, from: newManifestData)

    guard newManifest.version != manifest?.version else { return }

    // Version changed — update manifest
    manifest = newManifest

    // Option A: Reload current page silently
    if let currentPage = navigationStack.last {
        await navigate(to: currentPage)
    }

    // Option B: Show "Update available" banner, let user choose
    // showUpdateBanner()
}
```

```kotlin
// Android
suspend fun checkForUpdates() {
    val newManifestData = fetch("/manifest.json")
    val newManifest = Json.decodeFromString<RouteManifest>(String(newManifestData))

    if (newManifest.version == manifest?.version) return

    manifest = newManifest
    // Reload current page — Compose recomposition handles the UI swap
    state.navigate(state.currentPageName)
}
```

---

## Offline Support

Pre-rendered files can be cached locally for offline access:

### iOS
```swift
// Cache pages to disk for offline use
let cache = URLCache(
    memoryCapacity: 50_000_000,   // 50MB memory
    diskCapacity: 200_000_000     // 200MB disk
)
URLSession.shared.configuration.urlCache = cache
```

### Android
```kotlin
// OkHttp cache for offline
val cache = Cache(cacheDir, 200L * 1024 * 1024) // 200MB
val client = OkHttpClient.Builder().cache(cache).build()
```

Flow with offline support:
```
1. App opens with no network
2. Adapter checks local cache for manifest + pages
3. If cached → render from cache (user sees last-known UI)
4. When network returns → check for updates → apply diff
```

---

## CDN Cache Versioning

How the app knows a new version exists, and how the CDN avoids serving stale content:

```
manifest.json           → Cache-Control: no-cache (always revalidated, ~500 bytes)
/pages/home/v42.fb      → Cache-Control: immutable (cached forever)
/pages/home/v43.fb      → Cache-Control: immutable (cached forever, NEW URL)
```

The manifest is the **only file that's always fresh**. It's tiny (~500 bytes), so revalidation costs nothing. Every page file uses a versioned URL — when content changes, the URL changes. Old URLs stay valid forever (no cache busting needed), new URLs are fetched fresh.

```
Why this works:
- manifest.json = no-cache = adapter always gets the latest version number
- Page files = versioned URLs = new version = new URL = CDN cache miss = fresh content
- Old version files = still cached = if user goes back (offline), they still work
- No cache invalidation API needed — just generate new URLs
```

---

## A/B Testing Via OTA

Since pages are just files on CDN, A/B testing is trivial:

```
manifest-v42-control.json  → links to control variant pages
manifest-v42-variant.json  → links to variant B pages

Server decides which manifest URL to give each user based on:
- User cohort
- Random assignment
- Feature flags
```

The adapter doesn't know or care about A/B testing. It just fetches whatever manifest URL it's given and renders the pages.

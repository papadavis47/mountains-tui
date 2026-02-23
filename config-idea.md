# Config Onboarding Idea

## Question

If I did not want users of this app to have to create a .env file - how would I implement this for users? What kind of onboarding would I do to have them input their Turso credential and DB url that are currently in ~/.mountains/.env?

## Options

**1. Make cloud sync opt-in (simplest)**
App works fully local by default. No credentials needed. If the user *wants* cloud sync, they run something like `mountains --setup-sync` or the app detects no credentials and offers a "Set up cloud sync?" option on the startup screen. Biggest UX win — most friction comes from *requiring* credentials before the app is usable at all.

**2. First-run TUI setup screen**
On first launch (no config file found), show a setup screen inside the TUI asking for:
- Turso DB URL
- Auth token

Save to a config file (e.g., `~/.mountains/config.toml`). Replaces the .env entirely. Reuse existing `InputHandler` infrastructure.

**3. CLI prompt before TUI starts**
If no config exists, use a simple `println!`/`stdin` prompt *before* entering the TUI:
```
No configuration found. Set up Turso Cloud sync?
DB URL: ___
Auth Token: ___
```
Less polished than a TUI screen but dead simple to implement.

**4. Config file with defaults instead of .env**
Replace `.env` with a structured config file (TOML is idiomatic Rust, use the `toml` crate). Auto-generate a template on first run with cloud sync disabled:
```toml
[sync]
enabled = false
# db_url = "libsql://..."
# auth_token = "..."
```
User edits when ready. App reads this instead of dotenv.

## Recommended Approach

Combine **1 + 2 + 4**:
- App works offline-first with zero config (no onboarding friction)
- Store settings in `config.toml` instead of `.env`
- On startup screen, add a new option like `C` — "Configure Cloud Sync" that opens a TUI setup flow
- Once configured, sync happens automatically as it does now
- If credentials are invalid/expired, degrade gracefully to offline mode (already handled)

This way the app is immediately usable with `cargo install && mountains` — no file creation, no env vars, no docs to read. Cloud sync becomes a feature they enable when ready.

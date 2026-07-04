# Mountains

## For mindfulness and motivation on the trails

### A Trail Running Training Log

#### _A digital tool to help runners get good at vert : )_

![Title on Startup](./images/title-startup-screen.png)

A TUI training log for trail running and food awareness. Tracks daily nutrition, body measurements, miles, elevation gain, strength & mobility work, and notes.

Works offline-first with a local `libsql` database at `~/.mountains/`. Optional Turso Cloud sync can be configured from within the app (press `c` on the startup screen).

# Installation

Clone the repo and install locally:

```shell

cargo install --path .

# add --force if installing over an older version

```

The `~/.mountains/` directory is created automatically on first run.

# Cloud Sync (Optional)

Cloud sync with Turso is opt-in. Configure it from the startup screen (`c`) or edit `~/.mountains/config.toml` directly:

```toml
[sync]
enabled = true
db_url = "libsql://your-db.turso.io"
auth_token = "your-token"
```

Syncs on startup (background) and on quit.

# Usage

```shell
cargo run
```

Or after installing:

```shell
mountains
```

### Made with [ratatui](https://ratatui.rs/) :)

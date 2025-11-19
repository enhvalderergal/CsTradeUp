# CsTradeUp

CsTradeUp is a small Rust-based desktop application that simulates buying, selling, opening, and trading-up Counter-Strike style skins. It is implemented with `eframe`/`egui` for the UI and `rusqlite` for a lightweight local SQLite database.

This README collects development notes, build/run instructions, and seeding guidance (previously in `SEEDING.md`).

---

## Features

- GUI client with screens: Buy, Sell, Tradeup, Open Skins, Inventory
- Local SQLite database with normalized `skins` and `inventory` tables
- Per-user balance, buy/sell flows, trade-up (consume 10 items → produce next-rarity item)
- Base64-decoded images are cached as textures with original image size to preserve aspect ratio
- Seedable skins catalog from `data/skins.json`

## Repo layout

- `Cargo.toml` — Rust project manifest
- `src/` — application sources
  - `main.rs` — app bootstrap and `CsApp` state
  - `db.rs` — SQLite schema and helpers
  - `models.rs` — domain models (Skin, Inventory row, etc.)
  - `scripts/` — higher-level operations (buy, open_case, tradeup, utilities)
  - `ui/` — `egui` UI modules and screen implementations
- `data/skins.json` — optional seed data (developer-provided)
- `cs_trade_up.db` — default runtime DB file (created at project root)

## Build & run

Prerequisites: Rust toolchain (stable). On Windows, PowerShell is used in examples.

Build:

```powershell
cargo build
```

Run the app:

```powershell
cargo run
```

The native window will open with the main menu. The app uses `cs_trade_up.db` in the project root by default.

## Seeding the skins catalog

The project includes a simple seeding flow. Seed data is read from `data/skins.json`, which should be a JSON array of skin objects. Each object may include the following fields:

- `name` (required)
- `rarity` (optional)
- `price` (optional, numeric)
- `collection` (optional)
- `weapon_type` (optional)
- `image_base64` (optional: a base64-encoded image string)

Auto-seed on launch

- `CsApp::default()` calls `db::init_db(&db_path)` which will create required tables and attempt to parse `data/skins.json` and insert skins. The seeding is best-effort — individual insert failures are ignored so the app can start even if some entries are invalid.

Manual seeding

If you prefer to run the seeder explicitly during development, a small binary is provided:

```powershell
cargo run --bin seed_skins
```

This reads `data/skins.json` and inserts each entry into the `skins` table. `add_skin` uses `INSERT OR IGNORE` to avoid duplicate names, making repeated runs idempotent with respect to `name`.

Example entry for `data/skins.json`:

```json
{
  "name": "AK-47 | Redline",
  "rarity": "Classified",
  "price": 12.5,
  "collection": "The Hydra Collection",
  "weapon_type": "AK-47",
  "image_base64": null
}
```

Notes

- If you want seeding to only run when the `skins` table is empty, this can be changed in `db::init_db`.
- `image_base64` is supported and converted to egui textures; textures are cached together with the original image size so the UI can preserve aspect ratio.

## Using the UI

- Main menu: quick navigation to Buy, Sell, Tradeup, Open Skins, Inventory
- Buy: browse boxed tiles for skins (images keep aspect ratio); buy button is disabled when you don't have enough balance
- Sell: list of inventory items with internal scroll area
- Open Skins: case-opening animation with weighted results; won skins are added to your inventory
- Tradeup: select exactly 10 items of the same rarity and click "Trade Up" to consume them and get a higher-rarity item (the UI disables the Trade Up button until selection is valid)

## Database schema (high-level)

- `users` (id INTEGER PRIMARY KEY, username TEXT UNIQUE, password_hash TEXT, balance REAL)
- `skins` (id INTEGER PRIMARY KEY, name TEXT UNIQUE, rarity TEXT, price REAL, collection TEXT, weapon_type TEXT, image_base64 TEXT)
- `inventory` (id INTEGER PRIMARY KEY, user_id INTEGER, skin_id INTEGER, created_at DATETIME)

See `src/db.rs` for the exact schema and queries.

## Development notes & TODOs

- Inventory UI: boxed tile grid (matching Buy) and wrapping behavior
- UX polish: disable Buy/Sell when unaffordable, highlight selections, tooltips for disabled buttons
- Tests: currently there are no automated tests; adding unit tests for `scripts/` and DB helpers would help future changes

## Contributing

If you'd like new features, open an issue or send a PR. For seeding or skin data updates, edit `data/skins.json` and run the seeder.

---

If you want, I can also:

- Run the seeder now and show newly inserted rows from `cs_trade_up.db`.
- Add example screenshot(s) and a short GIF demonstrating the case opening animation.

Enjoy!

## shinebot

"_when i shine...!_" utility bot specifically made for myself but has some applicability to others

---

### Usage

Intended for personal use, but has some vague flexibility for other users planned in the future.

#### Commands:

```
!help - Gives you this list of commands.
!todo - Sets reminder in database to do task.
!bus - Gives bus times relative to provided WT stop grouping.
!ping - Basic check for response.
```

---

### Dependancies

All listed within `Cargo.toml`, with the additional need for the [sqlx-cli](https://crates.io/crates/sqlx-cli) library within your hosting service.

---

### Startup

```
// install sqlx-cli
cargo install sqlx-cli

// create reference
sqlx db create
sqlx migrate run
cargo sqlx prepare

// run
cargo run
```

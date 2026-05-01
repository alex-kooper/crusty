# 🦀 Crusty

A minimal Rust CLI for managing Canton Coin (Amulet) on Canton Network. Query balances, create transfers, and manage parties via the JSON Ledger API.

## Commands (MVP)

| Command | Description |
|---------|-------------|
| `crusty login` | Browser-based OAuth2 + PKCE login via Keycloak |
| `crusty create-party` | Allocate a new party on the participant |
| `crusty list-parties` | List parties on the participant |
| `crusty balance` | Query Amulet/CC holdings balance |
| `crusty transfer` | Create a transfer instruction (offer) |
| `crusty accept` | Accept pending transfer(s) |
| `crusty reject` | Reject pending transfer(s) |

## Auth

Crusty supports multiple authentication modes:

- **Browser login** — `crusty login` opens a browser for OAuth2 Authorization Code + PKCE flow
- **Direct token** — `crusty --token <JWT> <command>` for scripting
- **Env file** — `crusty --env <file> <command>` for client credentials

## Building

```bash
cargo build --release
```

## License

MIT

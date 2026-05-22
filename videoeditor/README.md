# VideoEditor

A Linux desktop video editor for quick clip work.

See `docs/superpowers/specs/2026-05-22-videoeditor-design.md` for the design.

## Development

```bash
cd videoeditor
npm install
npm run tauri dev
```

## Tests

- Frontend logic: `npm test`
- Rust core: `cd src-tauri && cargo test`

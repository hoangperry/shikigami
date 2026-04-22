# Icons

Placeholder directory. Tauri 2 requires icon files at the paths declared in
`tauri.conf.json`:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png` (256×256)
- `icon.icns` (macOS bundle)

Generate real icons before first release build:

```bash
pnpm tauri icon path/to/source-1024.png
```

Source should be a square PNG, minimum 1024×1024, with transparency.

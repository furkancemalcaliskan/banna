# Banna brand assets

The canonical Banna logo is the circular beaver, dam, lake, forest, and mountain
illustration provided by Furkan Cemal Caliskan. `assets/brand/banna.png` is the
source of truth; derived assets must not introduce another symbol, wordmark, or
writing system.

## Source record

The author-provided source is `assets/brand/banna.png` (1254 x 1254 RGBA,
SHA-256
`11f3408281c53169a2cd83822498081ab3916c9682dfa430e1af500b8b68f029`).
It was placed in the repository by Furkan Cemal Caliskan and designated as the
official Banna logo on 2026-08-06.

## Derived mobile assets

Run `scripts/render-brand-assets.sh` with ImageMagick installed to recreate the
five assets consumed by the embedded Expo project. The script preserves the
logo artwork and performs only deterministic resizing, placement, palette
conversion, and metadata removal. No generated text or additional mark is
added. Repeated runs produce byte-identical PNG files:

- `icon.png` — 192 x 192 application icon;
- `adaptive-icon.png` — 1024 x 1024 transparent Android foreground;
- `splash.png` — 1242 x 2436 launch screen;
- `logo.png` — 256 x 256 in-application logo;
- `avatar.png` — 1024 x 1024 default profile artwork.

Treat `banna.png` and the rendering script as the editable source of truth. Do
not hand-edit derived PNG files. Run
`sha256sum --check assets/brand/SHA256SUMS` from the repository root to verify
the source and derivatives.

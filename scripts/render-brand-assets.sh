#!/usr/bin/env bash
set -euo pipefail

brand_repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
brand_source="$brand_repo_root/assets/brand/banna.png"
brand_output="$brand_repo_root/projects/Vanilla/react-native/assets"
brand_trimmed="$(mktemp --suffix=.png)"

trap 'rm -f "$brand_trimmed"' EXIT

command -v magick >/dev/null 2>&1 || {
  echo "ImageMagick (magick) is required" >&2
  exit 1
}

magick "$brand_source" -trim +repage "$brand_trimmed"

magick -size 192x192 xc:'#F5F0E4' \
  \( "$brand_trimmed" -resize 184x184 \) \
  -gravity center -composite -strip -colors 256 \
  -define png:exclude-chunk=date,time "PNG8:$brand_output/icon.png"

magick -size 1024x1024 xc:none \
  \( "$brand_trimmed" -resize 640x640 \) \
  -gravity center -composite -strip -colors 256 \
  -define png:exclude-chunk=date,time "PNG8:$brand_output/adaptive-icon.png"

magick -size 256x256 xc:none \
  \( "$brand_trimmed" -resize 248x248 \) \
  -gravity center -composite -strip -colors 256 \
  -define png:exclude-chunk=date,time "PNG8:$brand_output/logo.png"

magick -size 1024x1024 xc:'#F5F0E4' \
  \( "$brand_trimmed" -resize 940x940 \) \
  -gravity center -composite -strip -colors 256 \
  -define png:exclude-chunk=date,time "PNG8:$brand_output/avatar.png"

magick -size 1242x2436 xc:'#F5F0E4' \
  \( "$brand_trimmed" -resize 760x760 \) \
  -gravity center -composite -strip -colors 256 \
  -define png:exclude-chunk=date,time "PNG8:$brand_output/splash.png"

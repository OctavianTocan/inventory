#!/usr/bin/env bash
# Packages the desktop app for this OS, set to open the hosted library, into downloads/ where /download serves it.
# Usage: deploy/package-app.sh <library-url>
set -euo pipefail

url=${1:?usage: deploy/package-app.sh <library-url>}
root=$(cd "$(dirname "$0")/.." && pwd)
app="$root/desktop/src-tauri"
out="$root/downloads"
name="mymind-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT

# A separate target directory keeps the checkout's own build opening the local library.
(cd "$app" && MYMIND_URL="$url" CARGO_TARGET_DIR="$app/target/hosted" cargo build --release)

mkdir -p "$stage/mymind" "$out"
cp "$app/target/hosted/release/mymind-desktop" "$stage/mymind/mymind"
cp "$app/icons/icon.png" "$stage/mymind/mymind.png"
cat > "$stage/mymind/install.sh" <<'EOF'
#!/bin/sh
# Installs mymind for this user: ~/.local/bin/mymind and an app launcher entry.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
install -Dm755 "$here/mymind" "$HOME/.local/bin/mymind"
install -Dm644 "$here/mymind.png" "$HOME/.local/share/icons/hicolor/512x512/apps/mymind.png"
mkdir -p "$HOME/.local/share/applications"
cat > "$HOME/.local/share/applications/mymind.desktop" <<ENTRY
[Desktop Entry]
Type=Application
Name=Inventory
Exec=$HOME/.local/bin/mymind
Icon=mymind
StartupWMClass=mymind
Categories=Utility;
ENTRY
echo "Installed. Start mymind from your launcher, or run ~/.local/bin/mymind."
EOF
chmod +x "$stage/mymind/install.sh"
tar -czf "$out/$name" -C "$stage" mymind

node --input-type=module - "$out" "$name" "$url" <<'EOF'
import { createHash } from 'node:crypto';
import { readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
const [out, name, library] = process.argv.slice(2);
const manifestPath = path.join(out, 'manifest.json');
let manifest = { files: [] };
try { manifest = JSON.parse(readFileSync(manifestPath, 'utf8')); } catch {}
const [, os, arch] = name.match(/^mymind-([^-]+)-(.+)\.tar\.gz$/);
const entry = {
  name,
  label: `${{ linux: 'Linux', darwin: 'macOS' }[os] ?? os} (${arch})`,
  bytes: statSync(path.join(out, name)).size,
  sha256: createHash('sha256').update(readFileSync(path.join(out, name))).digest('hex'),
  built: new Date().toISOString(),
};
manifest = { library, files: [...manifest.files.filter((file) => file.name !== name), entry] };
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`${name}: ${(entry.bytes / 1_048_576).toFixed(1)} MB, sha256 ${entry.sha256}`);
EOF

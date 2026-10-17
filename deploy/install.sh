#!/usr/bin/env bash
# Hosts the replica on this computer for other machines: the server with Google sign-in, the packaged desktop app at
# /download, and the Cloudflare tunnel that publishes both, as systemd user services that start at boot. The library's
# own sign-in is the only gate: signed-out visitors through the tunnel get a plain sign-in page and nothing else.
# Usage: deploy/install.sh <hostname> <tunnel-id>
set -euo pipefail

usage='usage: deploy/install.sh <hostname> <tunnel-id>'
host=${1:?$usage}
tunnel=${2:?$usage}
root=$(cd "$(dirname "$0")/.." && pwd)
server="$root/rust/target/release/mymind"
cloudflared=$(command -v cloudflared)
credentials="$HOME/.cloudflared/$tunnel.json"
config="$HOME/.cloudflared/mymind.yml"
units="$HOME/.config/systemd/user"

[[ -x $server ]] || { echo "No $server: build it with rust/fetch-libvips.sh, then cargo build --release in rust/ (README)." >&2; exit 1; }
[[ -f $credentials ]] || { echo "No $credentials: create the tunnel with cloudflared tunnel create." >&2; exit 1; }
for name in GOOGLE_CLIENT_ID GOOGLE_CLIENT_SECRET OWNER_EMAIL; do
  grep -q "^$name=." "$root/.env" 2>/dev/null || { echo "$root/.env has no $name (README, Hosting)." >&2; exit 1; }
done

mkdir -p "$units"
cat > "$config" <<EOF
tunnel: $tunnel
credentials-file: $credentials
ingress:
  - hostname: $host
    path: ^/download(/.*)?\$
    service: http://127.0.0.1:4332
  - hostname: $host
    service: http://127.0.0.1:4330
  - service: http_status:404
EOF
"$cloudflared" tunnel --config "$config" ingress validate >/dev/null

cat > "$units/mymind-replica.service" <<EOF
[Unit]
Description=mymind replica with Google sign-in
After=network-online.target
Wants=network-online.target

[Service]
WorkingDirectory=$root
Environment=PORT=4330 LOCAL_PORT=4331 AUTH=google PUBLIC_URL=https://$host
ExecStart=$server serve
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

cat > "$units/mymind-tunnel.service" <<EOF
[Unit]
Description=Cloudflare tunnel publishing the mymind replica at $host
After=network-online.target mymind-replica.service
Wants=network-online.target
PartOf=mymind-replica.service

[Service]
ExecStart=$cloudflared --no-autoupdate --config $config tunnel run
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

cat > "$units/mymind-download.service" <<EOF
[Unit]
Description=mymind desktop app downloads for signed-in users
After=network-online.target
Wants=network-online.target
PartOf=mymind-replica.service

[Service]
WorkingDirectory=$root
Environment=DOWNLOAD_PORT=4332
ExecStart=$server download-server
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

services=(mymind-replica.service mymind-tunnel.service mymind-download.service)
systemctl --user daemon-reload
systemctl --user enable "${services[@]}"
systemctl --user restart "${services[@]}"
echo "https://$host is served by mymind-replica.service behind its sign-in; this machine can also use http://localhost:4331 without sign-in."

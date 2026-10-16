#!/usr/bin/env bash
# Hosts the replica on this computer for other machines: the server with Google sign-in and the Cloudflare tunnel
# that publishes it, as systemd user services that start at boot.
# Usage: deploy/install.sh <hostname> <tunnel-id>
set -euo pipefail

usage='usage: deploy/install.sh <hostname> <tunnel-id>'
host=${1:?$usage}
tunnel=${2:?$usage}
root=$(cd "$(dirname "$0")/.." && pwd)
node=$(node -p process.execPath)
cloudflared=$(command -v cloudflared)
credentials="$HOME/.cloudflared/$tunnel.json"
config="$HOME/.cloudflared/mymind.yml"
units="$HOME/.config/systemd/user"

[[ -f $credentials ]] || { echo "No $credentials: create the tunnel with cloudflared tunnel create." >&2; exit 1; }
for name in GOOGLE_CLIENT_ID GOOGLE_CLIENT_SECRET OWNER_EMAIL; do
  grep -q "^$name=." "$root/.env" 2>/dev/null || { echo "$root/.env has no $name (README, Sign-in)." >&2; exit 1; }
done

mkdir -p "$units"
cat > "$config" <<EOF
tunnel: $tunnel
credentials-file: $credentials
ingress:
  - hostname: $host
    service: http://127.0.0.1:4330
  - service: http_status:404
EOF

cat > "$units/mymind-replica.service" <<EOF
[Unit]
Description=mymind replica with Google sign-in
After=network-online.target
Wants=network-online.target

[Service]
WorkingDirectory=$root
Environment=PORT=4330 LOCAL_PORT=4331 AUTH=google PUBLIC_URL=https://$host SHARP_IGNORE_GLOBAL_LIBVIPS=1
ExecStart=$node server.mjs
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

systemctl --user daemon-reload
systemctl --user enable --now mymind-replica.service mymind-tunnel.service
echo "https://$host is served by mymind-replica.service; this machine can also use http://localhost:4331 without sign-in."

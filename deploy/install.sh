#!/usr/bin/env bash
# Hosts the replica on this computer for other machines: the server with Google sign-in and the Cloudflare tunnel
# that publishes it, as systemd user services that start at boot. With CF_ACCESS_TEAM_DOMAIN and CF_ACCESS_AUD in
# .env it also serves the packaged desktop app at /download, behind that Cloudflare Access application.
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
setting() { sed -n "s/^$1=//p" "$root/.env" | tail -1 | tr -d "\"'"; }
access_team=$(setting CF_ACCESS_TEAM_DOMAIN)
access_aud=$(setting CF_ACCESS_AUD)
services=(mymind-replica.service mymind-tunnel.service)

mkdir -p "$units"
{
  echo "tunnel: $tunnel"
  echo "credentials-file: $credentials"
  echo "ingress:"
  if [[ -n $access_team && -n $access_aud ]]; then
    printf '  - hostname: %s\n    path: ^/download(/.*)?$\n    service: http://127.0.0.1:4332\n' "$host"
  fi
  printf '  - hostname: %s\n    service: http://127.0.0.1:4330\n  - service: http_status:404\n' "$host"
} > "$config"

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

if [[ -n $access_team && -n $access_aud ]]; then
  cat > "$units/mymind-download.service" <<EOF
[Unit]
Description=mymind desktop app downloads behind Cloudflare Access
After=network-online.target
Wants=network-online.target
PartOf=mymind-replica.service

[Service]
WorkingDirectory=$root
Environment=DOWNLOAD_PORT=4332 CF_ACCESS_TEAM_DOMAIN=$access_team CF_ACCESS_AUD=$access_aud OWNER_EMAIL=$(setting OWNER_EMAIL)
ExecStart=$node deploy/download-server.mjs
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF
  services+=(mymind-download.service)
else
  systemctl --user disable --now mymind-download.service 2>/dev/null || true
  rm -f "$units/mymind-download.service"
  echo "/download is off: put CF_ACCESS_TEAM_DOMAIN and CF_ACCESS_AUD in .env (README, Hosting on this computer)."
fi

systemctl --user daemon-reload
systemctl --user enable "${services[@]}"
systemctl --user restart "${services[@]}"
echo "https://$host is served by mymind-replica.service; this machine can also use http://localhost:4331 without sign-in."

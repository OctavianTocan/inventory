#!/usr/bin/env bash
# Run beside `npm run dev` when the media container has to start under wrangler dev on this computer.
# wrangler's networking sidecar (cloudflare/proxy-everything) routes marked packets to itself. Its network namespace
# inherits the host's net.ipv4.conf.all.src_valid_mark=1, and with that the kernel drops the host's packets to the
# sidecar, so workerd never reaches it and the container never starts. This turns the setting off inside each sidecar
# as it starts; the host's own setting stays as it is. Needs sudo without a password prompt (nsenter).
set -uo pipefail
docker events --filter type=container --filter event=start --format '{{.Actor.ID}} {{.Actor.Attributes.name}}' |
  while read -r id name; do
    case "$name" in workerd-*-proxy) ;; *) continue ;; esac
    pid=$(docker inspect "$id" --format '{{.State.Pid}}') || continue
    sudo -n nsenter -t "$pid" -n sysctl -qw net.ipv4.conf.all.src_valid_mark=0 && echo "src_valid_mark off in $name"
  done

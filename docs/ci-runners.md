# MyMind CI runners

On 28 October 2026, two repository-scoped Linux runners were installed on
`mini-002-netcup`: `mymind-linux-1` and `mymind-linux-2`. MyMind workflow jobs
use the `mymind-linux` label. The repository must remain private.

The runners use the dedicated `mymindgha` account, without sudo or Docker group
membership. Runner installations and workspaces live in
`/opt/actions-runner-mymind-1` and `/opt/actions-runner-mymind-2`. Each runner
has a separate home and Actions tool cache in `/home/mymindgha/runner-N`; Rust
toolchains and Cargo cache are under `/home/mymindgha/.rustup` and `.cargo`.
Services are named
`actions.runner.OctavianTocan-mymind-reconstruction.mymind-linux-N.service`.

Check status on mini-002:

```sh
gh api repos/OctavianTocan/mymind-reconstruction/actions/runners --jq '.runners[] | {name,status,busy}'
systemctl is-active actions.runner.OctavianTocan-mymind-reconstruction.mymind-linux-{1,2}.service
```

These runners persist between jobs and share an account and host. Only trusted
collaborators should be able to run workflows; changing repository access also
changes who can run code on the host. Android publication and Alchemy cloud
operations retain their existing triggers and protections.

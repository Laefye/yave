# YAVE

YAVE is a small utility for managing local KVM/QEMU virtual machines through a unified context. The project is packaged as a Rust workspace: the core is responsible for preparing configurations, disk storage, and launching virtual machines; `cli` provides the command-line interface; and auxiliary crates encapsulate interaction with QMP, QEMU, and networking.

## Requirements
- Linux with KVM enabled.
- `qemu-system-x86_64`, `qemu-img`, `genisoimage`, `iproute2`, `nftables`, `bridge-utils` (for network scripts).
- Rust toolchain with Edition 2024 support (nightly 1.85+ as of December 2025).
- Write access to `debug/`, `netdevup`, `netdevdown`, and the directory where QCOW2 disks and sockets are stored.

## Repository Structure
- `src/` - core (`YaveContext`, `VmContext`, launch and networking utilities).
- `cli/` - command-line binary.
- `qemu/`, `qmp/`, `nft/`, `vm_types/` - child crates with low-level logic.
- `debug/` - example configuration, VNC table, and VM directories (`*.vm`).
- `netdevup`, `netdevdown` - user scripts invoked by QEMU when TAP interfaces are brought up/down.
- `web/` - Web API

## CLI Commands

* `create` — creates a VM. Options: `--image <basename>` (copy of a ready qcow2 from `debug/`), `--preset <name>` (directory `<name>.preset`), `--hostname`, `--root-password`, `--vnc-password`.
* `list` — lists `*.vm` directories in `debug/`.
* `run` — starts the VM, creates PID/QMP sockets in `debug/run/`, and sets the VNC password via QMP.
* `shutdown` — sends `quit` over QMP.
* `netdev --name <vm> --ifname <tap> <up|down>` — attaches a TAP interface to the master interface from the configuration and brings the link up.

Examples:

```bash
# VM from a preset (cloud image conversion + cloud-init)
cargo run -p cli -- create --name cloud --preset ubuntu.preset \
    --capacity 20480 --hostname cloud1 \
    --root-password s3cret --vnc-password 87654321

# Bring up the TAP interface assigned to the VM
sudo cargo run -p cli -- netdev --name cloud --ifname tap0 up
```

## Storage

* VM configs: `debug/<vm>.vm/config.yaml`.
* Disks: `debug/<vm>.vm/hd*.qcow2`.
* Cloud-init ISOs: temporarily created in `/tmp`.
* QMP sockets and PID files: `debug/run/<vm>.sock|pid`.
* VNC table: `debug/vnc_table.yaml`.

## Status

The project is under active development; APIs and file formats may change.

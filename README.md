# Fridge

Fridge is a command-line tool for taking snapshots on BTRFS and ZFS filesystems and synchronizing them locally or remotely via SSH. It currently assumes `.snapshots` directory exist in a path for containing the snapshots. The goal I'm hoping for is to have the usage to be as simple as possible while also being robust. It is currently neither as it is still very much in work-in-progress and proof-of-concept stage. Many things may change and feedbacks are very welcome!

## Usage

### Taking snapshots

```
fridge snapshot <name> <path> [suffix]
```

e.g.
```
fridge snapshot root /
```

or if suffix is desired
```
fridge snapshot root / _manual
```

### Synchronization

The synchronization will transfer all the snapshots that don't already exist in the destination. The source and destination can be local or remote.

```
fridge sync <name> <src> <dst>
```

NOTE:
`<src>` and `<dst>` refer to the paths that contain snapshots. By default, it automatically appends `.snapshots` to the paths. It can be overridden using `--append-suffix-src=<suffix>` and `--append-suffix-dst=<suffix>`.

e.g.

For local sync:
```
# snapshots are at /.snapshots and /media/user/EXT_HDD/.snapshots
fridge snapshot root / /media/user/EXT_HDD/
```

For remote sync
```
# snapshots are at /.snapshots and admin@192.168.0.2/.snapshots
fridge snapshot root / remote:admin@192.168.0.2:/
```

### Building

If you have installed Rust, you can compile it by running:

```
cargo build --release
```

### Installation

You can copy the resulting binary at `target/release/fridge` into `/usr/local/bin`.

### Scheduling

#### For snapshot

1. Set up systemd service at `/etc/systemd/system`

```
# fridge-daily-root.service
[Unit]
Description=Fridge - Daily snapshot (root)

[Service]
ExecStart=/usr/local/bin/fridge snapshot root / _daily --append-date
Type=oneshot

[Install]
WantedBy=multi-user.target
```

2. Set up systemd timer at `/etc/systemd/system`

```
# fridge-daily-root.timer
[Unit]
Description=Fridge - Daily snapshot timer (root)

[Timer]
OnCalendar=*-*-* 12:00:00

[Install]
WantedBy=timers.target
```

#### For sync

1. Set up systemd service at `/etc/systemd/system`

```
# fridge-daily-root-sync.service
[Unit]
Description=Fridge - Daily sync (root)

[Service]
ExecStart=/usr/local/bin/fridge sync root / remote:192.168.50.200:/ThinkPad-T495
Type=oneshot

[Install]
WantedBy=multi-user.target
```

2. Set up systemd timer at `/etc/systemd/system`

```
# fridge-daily-root-sync.timer
[Unit]
Description=Fridge - Daily sync timer (root)

[Timer]
OnCalendar=*-*-* 12:00:00

[Install]
WantedBy=timers.target
```
# Fridge

Fridge is a command-line tool for taking snapshots on BTRFS and ZFS filesystems and synchronizing them locally or remotely via SSH. It currently assumes `.snapshots` directory exist in a path for containing the snapshots. The goal I'm hoping for is to have the usage to be as simple as possible while also being robust. It is currently neither as it is still very much in work-in-progress and proof-of-concept stage. Many things may change and feedbacks are very welcome!

## Usage

### Building

If you have installed Rust, you can compile it by running:

```
cargo build --release
```

### Installation

You can copy the resulting binary at `target/release/fridge` into `/usr/local/bin`.

### Scheduling

The following is based on how I'm using it now.

1. Create configuration file at `/etc/fridge/fridge.toml`.

```
sudo mkdir /etc/fridge
sudo vim /etc/fridge/fridge.toml
```

then put content like this (which you mostly like need to change):

```
[local]
sudo = true
path = "/"

[[snapshots]]
name = "root"
path = "/"
hourly = 24
daily = 7
weekly = 4
monthly = 12
yearly = 3

[[snapshots]]
name = "home"
path = "/home"
hourly = 24
daily = 7
weekly = 4
monthly = 12
yearly = 3

[[remotes]]
user = "li"
host = "192.168.0.2"
path = "/.snapshots/"
suffix = "ThinkPad-T495"
sudo = true
```

NOTE: You can remove `[[remotes]]` section if you don't want or have remote backup destination.
NOTE: `sudo` is not needed when the program is run as root (for example, by systemd) but it's convenient to have if you want to run it manually.

2. Set up systemd service at `/etc/systemd/system/fridge.service`

```
[Unit]
Description=Fridge - Run

[Service]
ExecStart=/usr/local/bin/fridge run
Type=oneshot

[Install]
WantedBy=multi-user.target
```

3. Set up systemd timer at `/etc/systemd/system/fridge.timer`

```
[Unit]
Description=Fridge - Run

[Timer]
OnCalendar=hourly

[Install]
WantedBy=timers.target
```
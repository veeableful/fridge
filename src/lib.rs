use std::path::Path;
use std::process::{Command,Stdio};
use std::str;
use anyhow::{Result, bail};
use chrono::Utc;

#[derive(Default)]
pub struct SnapshotOpts {
    pub src: String,
    pub snapshot_name: String,
    pub suffix: Option<String>,
    pub dry_run: bool,
    pub verbose: i32,
}

// e.g. btrfs sub snap -r [path] [destination]
pub fn snapshot(opts: SnapshotOpts) -> Result<()> {
    let date = Utc::now();
    let date_str = format!("{}", date.format("%Y-%m-%d_%H:%M:%S"));
    let snapshot_name = if let Some(suffix) = opts.suffix {
        format!("{}@{}{}", &opts.snapshot_name, date_str, suffix)
    } else {
        format!("{}@{}", &opts.snapshot_name, date_str)
    };

    let dst_path = Path::new("/").join(".snapshots").join(&snapshot_name);
    let dst = dst_path.to_str().unwrap();
    if opts.dry_run {
        eprintln!("Would create snapshot of {} at {}", opts.src, dst);
        return Ok(());
    }

    let output = Command::new("btrfs")
        .args(["subvolume", "snapshot", "-r", &opts.src, dst])
        .output()?;

    eprintln!("Created snapshot at {}", dst);

    if !output.status.success() {
        bail!("Could not create snapshot: {}", str::from_utf8(&output.stderr).unwrap());
    }

    Ok(())
}

#[derive(Default)]
pub struct SyncOpts {
    pub name: String,
    pub src: String,
    pub dst: String,
    pub dry_run: bool,
    pub append_suffix_src: String,
    pub append_suffix_dst: String,
    pub verbose: i32,
}

// e.g. btrfs send /.snapshots/root@2001-02-03_04:05:06_daily | ssh admin@192.168.50.200 sudo btrfs receive /.snapshots
pub fn sync(opts: SyncOpts) -> Result<()> {
    let src = parse_sync_location(&opts.src, &opts.append_suffix_src)?;
    let dst = parse_sync_location(&opts.dst, &opts.append_suffix_dst)?;
    let src_list_string = list_snapshots(&opts.name, &src, opts.verbose)?;
    let dst_list_string = list_snapshots(&opts.name, &dst, opts.verbose)?;
    let src_list: Vec<&str> = src_list_string.iter().map(|s| &**s).collect();
    let dst_list: Vec<&str> = dst_list_string.iter().map(|s| &**s).collect();
    let diff_list = diff_snapshot_lists(&src_list, &dst_list)?;
    if diff_list.len() == 0 {
        eprintln!("Already up-to-date");
        return Ok(());
    }
    let mut parent: Option<&str> = parent_snapshot_to(diff_list[0], &src_list).unwrap();

    for snapshot in diff_list {
        let mut transfer_opts = TransferOpts::default();
        if let Some(parent) = parent {
            transfer_opts.parent_snapshot = Some(parent.to_string());
        }
        transfer_opts.snapshot = snapshot.to_string();
        transfer_opts.src = src.clone();
        transfer_opts.dst = dst.clone();
        transfer_opts.dry_run = opts.dry_run;
        transfer_opts.verbose = opts.verbose;
        transfer(transfer_opts)?;
        parent = Some(snapshot);
    }

    Ok(())
}

#[derive(Default)]
pub struct TransferOpts {
    pub parent_snapshot: Option<String>,
    pub snapshot: String,
    pub src: SyncLocation,
    pub dst: SyncLocation,
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn transfer(opts: TransferOpts) -> Result<()> {
    if opts.src.is_remote {
         
    } else {
        let snapshot_path = Path::new(&opts.src.path).join(&opts.snapshot).to_str().unwrap().to_string();
        let mut send_output_child = if let Some(parent_snapshot) = opts.parent_snapshot {
            let parent_snapshot_path = Path::new(&opts.src.path).join(parent_snapshot).to_str().unwrap().to_string();
            if opts.dry_run {
                eprintln!("Would transfer snapshot {} with parent {}", &snapshot_path, &parent_snapshot_path);
                return Ok(());
            }
            eprintln!("Transferring snapshot {} with parent {}", &snapshot_path, &parent_snapshot_path);
            Command::new("btrfs")
                .args(["send", "-p", &parent_snapshot_path, &snapshot_path])
                .stdout(Stdio::piped())
                .spawn()?
        } else {
            if opts.dry_run {
                eprintln!("Would transfer snapshot {}", &snapshot_path);
                return Ok(());
            }
            eprintln!("Transferring snapshot {}", &snapshot_path);
            Command::new("btrfs")
                .args(["send", &snapshot_path])
                .stdout(Stdio::piped())
                .spawn()?
        };

        if let Some(send_output) = send_output_child.stdout.take() {
            let (program, args) = if opts.dst.is_remote {
                if let Some(host) = opts.dst.host {
                    if let Some(user) = opts.dst.user {
                        let base_url = format!("{}@{}", user, host);
                        if let Some(port) = opts.dst.port {
                            ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                        } else {
                            ("ssh", vec![base_url, "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                        }
                    } else {
                        ("ssh", vec![host, "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                    }
                } else {
                    bail!("Could not sync remotely without host specified!");
                }
            } else {
                ("btrfs", vec!["receive".to_string(), opts.dst.path.to_string()])
            };

            let mut receive_output_child = Command::new(program)
                .stdin(send_output)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?;

            send_output_child.wait()?;
            receive_output_child.wait()?;
        }
    }

    Ok(())
}

pub struct Snapshot {
    pub name: String,
    pub date: chrono::Utc,
    pub suffix: String,
}

fn list_snapshots(name: &str, dst: &SyncLocation, verbose: i32) -> Result<Vec<String>> {
    // List snapshots
    let (program, args) = if dst.is_remote {
        if let Some(host) = dst.host.clone() {
            if let Some(user) = dst.user.clone() {
                let base_url = format!("{}@{}", user, host);
                if let Some(port) = dst.port {
                    ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), "-s".to_string(), dst.path.clone()])
                } else {
                    ("ssh", vec![base_url, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), "-s".to_string(), dst.path.clone()])
                }
            } else {
                ("ssh", vec![host, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), "-s".to_string(), dst.path.clone()])
            }
        } else {
            bail!("Could not sync remotely without host specified!");
        }
    } else {
        ("btrfs", vec!["subvolume".to_string(), "list".to_string(), "-s".to_string(), dst.path.clone()])
    };

    let output = Command::new(program)
        .args(args)
        .output()?;

    if !output.status.success() {
        bail!("Could not list snapshots with name {}: {}", name, str::from_utf8(&output.stderr).unwrap());
    }

    let stdout = str::from_utf8(&output.stdout).unwrap();
    if verbose > 0 {
        eprintln!("Stdout:\n{}", stdout);
    }

    let snapshot_list: Vec<String> = stdout
        .trim()
        .split("\n")
        .map(|line| line.split(" ").last().unwrap().to_string())
        .filter(|last_field| {
            let mut tokens = last_field.split("@");
            let a = tokens.next();
            if let Some(v) = a {
                v == name && tokens.next().is_some()
            } else {
                return false
            }
        })
        .collect();

    eprintln!("There are {} snapshots", snapshot_list.len());
    for snapshot_entry in &snapshot_list {
        println!("{}", snapshot_entry);
    }

    Ok(snapshot_list)
}

fn diff_snapshot_lists<'a>(src_list: &[&'a str], dst_list: &[&'a str]) -> Result<Vec<&'a str>> {
    let mut final_list = Vec::new();

    for entry in src_list {
        if dst_list.contains(entry) {
            continue;
        }

        final_list.push(*entry);
    }

    Ok(final_list)
}

fn parent_snapshot_to<'a>(name: &'a str, snapshot_list: &[&'a str]) -> Result<Option<&'a str>> {
    let mut p = None;

    for entry in snapshot_list {
        if *entry == name {
            break
        }
        p = Some(*entry);
    }

    Ok(p)
}

#[derive(Default,Clone)]
pub struct SyncLocation {
    user: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    is_remote: bool,
}

fn parse_sync_location(url: &str, append_suffix: &str) -> Result<SyncLocation> {
    let mut sync_location = SyncLocation::default();
    let mut s: String = url.to_string();

    // Parse local / remote
    let tokens: Vec<&str> = s.split(":").collect();
    if tokens.len() >= 2 {
        if tokens[0] == "remote" {
            sync_location.is_remote = true;
            s = tokens[1..].join(":");
        }
    }

    // Parse user
    let tokens: Vec<&str> = s.split("@").collect();
    if tokens.len() >= 2 {
        sync_location.user = Some(tokens[0].to_string());
        s = tokens[1].to_string();
    } else {
    }

    if sync_location.is_remote {
        // Parse host
        let tokens: Vec<&str> = s.split(":").collect();
        if tokens.len() == 1 {
            sync_location.host = Some(tokens[0].to_string());
        } else if tokens.len() == 2 {
            sync_location.host = Some(tokens[0].to_string());
            if let Ok(port) = tokens[1].parse::<u16>() {
                sync_location.port = Some(port);
            } else if !append_suffix.is_empty() {
                sync_location.path = Path::new(tokens[1]).join(append_suffix).to_str().unwrap().to_string();
            } else {
                sync_location.path = tokens[1].to_string();
            }
        } else if tokens.len() >= 3 {
            sync_location.host = Some(tokens[0].to_string());

            if let Ok(port) = tokens[1].parse::<u16>() {
                sync_location.port = Some(port);
            } else {
                bail!("Could not parse port {}", tokens[1]);
            }

            if !append_suffix.is_empty() {
                sync_location.path = Path::new(tokens[2]).join(append_suffix).to_str().unwrap().to_string();
            } else {
                sync_location.path = tokens[2].to_string();
            }
        }
    } else {
        if !append_suffix.is_empty() {
            sync_location.path = Path::new(&s).join(append_suffix).to_str().unwrap().to_string();
        } else {
            sync_location.path = s.to_string();
        }
    }

    Ok(sync_location)
}

pub fn restore(src: &str, dst: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        eprintln!("Would restore snapshot {src} to {dst}", src=src, dst=dst);
        return Ok(());
    }

    // Rename subvolume
    let dst_old = format!("{}.old", dst);
    let output = Command::new("mv")
        .args([dst, &dst_old])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to rename subvolume {}, {}", dst, str::from_utf8(&output.stderr).unwrap());
    }

    // Restore snapshot
    let output = Command::new("btrfs")
        .args(["subvolume", "snapshot", src, dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot {}: {}", src, str::from_utf8(&output.stderr).unwrap());
    }

    // Unmount subvolume
    let output = Command::new("umount")
        .args([dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to unmount subvolume {}: {}", dst, str::from_utf8(&output.stderr).unwrap());
    }

    // Re-mount subvolume
    let output = Command::new("mount")
        .args([dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to re-mount subvolume {}: {}", dst, str::from_utf8(&output.stderr).unwrap());
    }

    Ok(())
}

pub struct ListOpts {
    pub name: String,
    pub path: String,
    pub verbose: i32,
    pub append_suffix: String,
}

pub fn list(opts: ListOpts) -> Result<()> {
    let sync_location = parse_sync_location(&opts.path, &opts.append_suffix).unwrap();
    if opts.verbose > 0 {
        eprintln!("Is remote? {}", sync_location.is_remote);
    }
    list_snapshots(&opts.name, &sync_location, opts.verbose).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SnapshotOpts;

    #[test]
    fn test_snapshot() {
        let mut opts = SnapshotOpts::default();
        opts.src = String::from("/");
        opts.snapshot_name = String::from("test");
        opts.dry_run = true;
        super::snapshot(opts).unwrap();
    }

    #[test]
    fn test_parse_sync_location() {
        {
            let dst = "remote:root@192.168.1.2:22222:/home";
            let sync_location = super::parse_sync_location(dst, ".snapshots").unwrap();
            assert_eq!(sync_location.user.unwrap(), "root");
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(sync_location.port.unwrap(), 22222);
            assert_eq!(&sync_location.path, "/home/.snapshots");
        }
        {
            let dst = "remote:root@192.168.1.2:/home";
            let sync_location = super::parse_sync_location(dst, ".snapshots").unwrap();
            assert_eq!(sync_location.user.unwrap(), "root");
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(&sync_location.path, "/home/.snapshots");
        }
        {
            let dst = "remote:192.168.1.2:/home";
            let sync_location = super::parse_sync_location(dst, ".snapshots").unwrap();
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(&sync_location.path, "/home/.snapshots");
        }
        {
            let dst = "/run/media/EXTERNAL_HDD";
            let sync_location = super::parse_sync_location(dst, ".snapshots").unwrap();
            assert_eq!(&sync_location.path, "/run/media/EXTERNAL_HDD/.snapshots");
        }
        {
            let dst = "backup";
            let sync_location = super::parse_sync_location(dst, ".snapshots").unwrap();
            assert_eq!(&sync_location.path, "backup/.snapshots");
        }
        {
            let dst = "remote:root@192.168.1.2:22222:/home";
            let sync_location = super::parse_sync_location(dst, "").unwrap();
            assert_eq!(sync_location.user.unwrap(), "root");
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(sync_location.port.unwrap(), 22222);
            assert_eq!(&sync_location.path, "/home");
        }
        {
            let dst = "remote:root@192.168.1.2:/home";
            let sync_location = super::parse_sync_location(dst, "").unwrap();
            assert_eq!(sync_location.user.unwrap(), "root");
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(&sync_location.path, "/home");
        }
        {
            let dst = "remote:192.168.1.2:/home";
            let sync_location = super::parse_sync_location(dst, "").unwrap();
            assert_eq!(sync_location.host.unwrap(), "192.168.1.2");
            assert_eq!(&sync_location.path, "/home");
        }
        {
            let dst = "/run/media/EXTERNAL_HDD";
            let sync_location = super::parse_sync_location(dst, "").unwrap();
            assert_eq!(&sync_location.path, "/run/media/EXTERNAL_HDD");
        }
        {
            let dst = "backup";
            let sync_location = super::parse_sync_location(dst, "").unwrap();
            assert_eq!(&sync_location.path, "backup");
        }
    }

    #[test]
    fn test_diff_snapshot_lists() {
        let a = vec!["a", "b"];
        let b = vec!["a"];
        let c = super::diff_snapshot_lists(&a, &b).unwrap();
        assert_eq!(c, vec!["b"]);
    }
}
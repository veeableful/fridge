#[macro_use]
extern crate lazy_static;

use std::net::{SocketAddr,TcpStream};
use std::path::Path;
use std::process::{Command,Stdio};
use std::str::{self,FromStr};
use anyhow::{Result, bail};
use chrono::{DateTime, TimeZone, Utc, Duration};
use log::{debug,info,warn};
use thiserror::Error;

pub mod config;
use config::SnapshotConfig;

#[derive(Error, Debug)]
pub enum FridgeError {
    #[error("Could not parse {what:?} in {snapshot:?}")]
    ParseSnapshot {
        snapshot: String,
        what: &'static str,
    },
}

#[derive(Default)]
pub struct SnapshotOpts {
    pub src: String,
    pub name: String,
    pub suffix: Option<String>,
    pub sudo: bool,
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn snapshot(opts: &SnapshotOpts) -> Result<()> {
    let date = Utc::now();
    let date_str = format!("{}", date.format("%Y-%m-%d_%H:%M:%S"));
    let full_name = format!("{}@{}_{}", &opts.name, &date_str, opts.suffix.clone().unwrap_or("manual".to_string()));
    let dst_path = Path::new("/").join(".snapshots").join(&full_name);
    let dst = dst_path.to_str().unwrap();
    if opts.dry_run {
        info!("Would create snapshot of {} at {}", opts.src, dst);
        return Ok(());
    }

    let (program, args) = if opts.sudo {
        ("sudo", vec!["btrfs", "subvolume", "snapshot", "-r", &opts.src, dst])
    } else {
        ("btrfs", vec!["subvolume", "snapshot", "-r", &opts.src, dst])
    };
    let output = Command::new(program)
        .args(args)
        .output()?;

    if !output.status.success() {
        bail!("Could not create snapshot: {}", str::from_utf8(&output.stderr).unwrap());
    }

    info!("Created read-only snapshot at {}", dst);

    Ok(())
}

#[derive(Default)]
pub struct SyncOpts {
    pub name: String,
    pub src: String,
    pub src_sudo: bool,
    pub src_suffix: String,
    pub dst: String,
    pub dst_sudo: bool,
    pub dst_suffix: String,
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn sync(opts: &SyncOpts) -> Result<()> {
    let src = parse_snapshot_repository_location(&opts.src, &opts.src_suffix)?;
    let dst = parse_snapshot_repository_location(&opts.dst, &opts.dst_suffix)?;
    let src_list_string = list_snapshots(&opts.name, &src, opts.src_sudo, opts.verbose)?;
    let dst_list_string = list_snapshots(&opts.name, &dst, opts.dst_sudo, opts.verbose)?;
    let src_list: Vec<&str> = src_list_string.iter().map(|s| s.full_name.as_str()).collect();
    let dst_list: Vec<&str> = dst_list_string.iter().map(|s| s.full_name.as_str()).collect();
    let missing_snapshots_in_destination: Vec<&str> = src_list.iter()
        .filter(|snapshot_name| dst_list.contains(snapshot_name))
        .map(|snapshot_name| *snapshot_name)
        .collect();
    if missing_snapshots_in_destination.len() == 0 {
        info!("Already up-to-date");
        return Ok(());
    }
    let mut parent: Option<&str> = find_parent_snapshot_to(missing_snapshots_in_destination[0], &src_list);

    for snapshot in missing_snapshots_in_destination {
        let mut transfer_opts = TransferOpts::default();
        transfer_opts.parent_snapshot = parent.map_or(None, |s| Some(s.to_string()));
        transfer_opts.snapshot = snapshot.to_string();
        transfer_opts.src = src.clone();
        transfer_opts.src_sudo = opts.src_sudo;
        transfer_opts.dst = dst.clone();
        transfer_opts.dst_sudo = opts.dst_sudo;
        transfer_opts.dry_run = opts.dry_run;
        transfer_opts.verbose = opts.verbose;
        transfer(&transfer_opts)?;
        parent = Some(snapshot);
    }

    Ok(())
}

#[derive(Default)]
pub struct TransferOpts {
    pub parent_snapshot: Option<String>,
    pub snapshot: String,
    pub src: SnapshotRepositoryLocation,
    pub src_sudo: bool,
    pub dst: SnapshotRepositoryLocation,
    pub dst_sudo: bool,
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn transfer(opts: &TransferOpts) -> Result<()> {
    if opts.src.is_remote() {
         unimplemented!()
    } else {
        let snapshot_path = Path::new(&opts.src.path).join(&opts.snapshot).to_str().unwrap().to_string();
        let mut send_output_child = if let Some(parent_snapshot) = &opts.parent_snapshot {
            let parent_snapshot_path = Path::new(&opts.src.path).join(parent_snapshot).to_str().unwrap().to_string();
            if opts.dry_run {
                info!("Would transfer snapshot {} with parent {}", &snapshot_path, &parent_snapshot_path);
                return Ok(());
            }
            info!("Transferring snapshot {} with parent {}", &snapshot_path, &parent_snapshot_path);
            let (program, args) = if opts.src_sudo {
                ("sudo", vec!["btrfs", "send", "-p", &parent_snapshot_path, &snapshot_path])
            } else {
                ("btrfs", vec!["send", "-p", &parent_snapshot_path, &snapshot_path])
            };
            Command::new(program)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?
        } else {
            if opts.dry_run {
                info!("Would transfer snapshot {}", &snapshot_path);
                return Ok(());
            }
            let (program, args) = if opts.src_sudo {
                ("sudo", vec!["btrfs", "send", &snapshot_path])
            } else {
                ("btrfs", vec!["send", &snapshot_path])
            };
            info!("Transferring snapshot {}", &snapshot_path);
            Command::new(program)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?
        };

        if let Some(send_output) = send_output_child.stdout.take() {
            let (program, args) = if opts.dst.is_remote() {
                if let Some(host) = &opts.dst.host {
                    if let Some(user) = &opts.dst.user {
                        let base_url = format!("{}@{}", user, host);
                        if let Some(port) = opts.dst.port {
                            if opts.dst_sudo {
                                ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "sudo".to_string(), "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                            } else {
                                ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                            }
                        } else {
                            if opts.dst_sudo {
                                ("ssh", vec![base_url, "sudo".to_string(), "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                            } else {
                                ("ssh", vec![base_url, "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                            }
                        }
                    } else {
                        if opts.dst_sudo {
                            ("ssh", vec![host.clone(), "sudo".to_string(), "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                        } else {
                            ("ssh", vec![host.clone(), "btrfs".to_string(), "receive".to_string(), opts.dst.path.clone()])
                        }
                    }
                } else {
                    bail!("Could not sync remotely without host specified!");
                }
            } else {
                if opts.dst_sudo {
                    ("btrfs", vec!["sudo".to_string(), "receive".to_string(), opts.dst.path.to_string()])
                } else {
                    ("btrfs", vec!["receive".to_string(), opts.dst.path.to_string()])
                }
            };

            if opts.dry_run {
                info!("Would run the following command: {} {}", &program, args.join(" "));
                return Ok(());
            }

            if opts.verbose > 0 {
                info!("{} {}", &program, args.join(" "));
            }

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

#[derive(Default)]
pub struct RestoreOpts {
    pub src: String,
    pub dst: String,
    pub name: String,
    pub suffix: Option<String>,
    pub sudo: bool,
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn restore(opts: &RestoreOpts) -> Result<()> {
    if opts.dry_run {
        info!("Would restore snapshot {} to {}", &opts.src, &opts.dst);
        return Ok(());
    }

    // Rename subvolume
    let dst_old = format!("{}.old", &opts.dst);
    let output = Command::new("mv")
        .args([&opts.dst, &dst_old])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to rename subvolume {}, {}", &opts.dst, str::from_utf8(&output.stderr).unwrap());
    }

    // Restore snapshot
    let output = Command::new("btrfs")
        .args(["subvolume", "snapshot", &opts.src, &opts.dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot {}: {}", &opts.src, str::from_utf8(&output.stderr).unwrap());
    }

    // Unmount subvolume
    let output = Command::new("umount")
        .args([&opts.dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to unmount subvolume {}: {}", &opts.dst, str::from_utf8(&output.stderr).unwrap());
    }

    // Re-mount subvolume
    let output = Command::new("mount")
        .args([&opts.dst])
        .output()?;

    if !output.status.success() {
        bail!("Could not restore snapshot: Unable to re-mount subvolume {}: {}", &opts.dst, str::from_utf8(&output.stderr).unwrap());
    }

    Ok(())
}

#[derive(Default)]
pub struct ListOpts {
    pub name: String,
    pub path: String,
    pub sudo: bool,
    pub verbose: i32,
    pub suffix: String,
}

pub fn list(opts: &ListOpts) -> Result<()> {
    let snapshot_repository_location = parse_snapshot_repository_location(&opts.path, &opts.suffix).unwrap();
    if opts.verbose > 0 {
        debug!("Is remote? {}", snapshot_repository_location.is_remote());
    }
    let snapshots = list_snapshots(&opts.name, &snapshot_repository_location, opts.sudo, opts.verbose)?;
    for snapshot in &snapshots {
        println!("{}", &snapshot.full_name);
    }
    Ok(())
}

pub struct RunOpts {
    pub dry_run: bool,
    pub verbose: i32,
}

pub fn run(opts: &RunOpts) -> Result<()> {
    let cfg = config::load();

    if opts.verbose > 0 {
        info!("{:?}", &cfg);
    }

    info!("Running snapshots");

    // Run snapshots
    for snapshot_cfg in &cfg.snapshots {
        let snapshot_location = SnapshotRepositoryLocation{
            user: None,
            host: None,
            port: None,
            path: format!("{}{}", &cfg.local.path, &cfg.local.suffix),
        };
        let sudo = cfg.local.sudo;
        let snapshots = list_snapshots(&snapshot_cfg.name, &snapshot_location, sudo, opts.verbose)?;
        run_snapshot(snapshot_cfg, &snapshots, "hourly", Duration::hours(1), sudo, opts.dry_run, opts.verbose)?;
        run_snapshot(snapshot_cfg, &snapshots, "daily", Duration::days(1), sudo, opts.dry_run, opts.verbose)?;
        run_snapshot(snapshot_cfg, &snapshots, "weekly", Duration::weeks(1), sudo, opts.dry_run, opts.verbose)?;
        run_snapshot(snapshot_cfg, &snapshots, "monthly", Duration::days(31), sudo, opts.dry_run, opts.verbose)?;
        run_snapshot(snapshot_cfg, &snapshots, "yearly", Duration::days(365), sudo, opts.dry_run, opts.verbose)?;
    }

    info!("Running synchronizations");

    // Run synchronizations
    for remote_cfg in &cfg.remotes {
        if let Some(host) = &remote_cfg.host {
            if let Some(port) = remote_cfg.port {
                let addr_str = format!("{}:{}", host, port);
                let addr = SocketAddr::from_str(&addr_str)?;
                if let Err(e) = TcpStream::connect_timeout(&addr, std::time::Duration::new(10, 0)) {
                    warn!("Could not connect to {}: {}", &addr, &e);
                    continue;
                }
            }
        }

        for snapshot_cfg in &cfg.snapshots {
            let sync_opts = SyncOpts {
                name: snapshot_cfg.name.clone(),
                src: cfg.local.path.clone(),
                src_sudo: cfg.local.sudo,
                src_suffix: cfg.local.suffix.clone(),
                dst: remote_cfg.location_string()?,
                dst_sudo: remote_cfg.sudo,
                dst_suffix: remote_cfg.suffix.clone(),
                dry_run: opts.dry_run,
                verbose: opts.verbose,
            };

            sync(&sync_opts)?;
        }
    }

    // Clean-up remote snapshots
    for remote_cfg in &cfg.remotes {
        if let Some(host) = &remote_cfg.host {
            if let Some(port) = remote_cfg.port {
                let addr_str = format!("{}:{}", host, port);
                let addr = SocketAddr::from_str(&addr_str)?;
                if let Err(e) = TcpStream::connect_timeout(&addr, std::time::Duration::new(10, 0)) {
                    warn!("Could not connect to {}: {}", &addr, &e);
                    continue;
                }
            }
        }

        let repository = remote_cfg.to_snapshot_repository_location();
        for snapshot_cfg in &cfg.snapshots {
            let mut snapshots = list_snapshots(&snapshot_cfg.name, &repository, remote_cfg.sudo, opts.verbose)?;
            let max_snapshot_count = match remote_cfg.suffix.as_str() {
                "hourly" => snapshot_cfg.hourly,
                "daily" => snapshot_cfg.daily,
                "weekly" => snapshot_cfg.weekly,
                "monthly" => snapshot_cfg.monthly,
                "yearly" => snapshot_cfg.yearly,
                _ => 1_000_000,
            };

            while snapshots.len() > max_snapshot_count {
                if let Some(snapshot) = snapshots.first() {
                    snapshot.delete(remote_cfg.sudo, opts.verbose)?;
                    info!("Deleted remote snapshot {}", &snapshot.full_name);
                }
                snapshots.remove(0);
            }
        }
    }

    Ok(())
}

fn run_snapshot(cfg: &SnapshotConfig, snapshots: &[Snapshot], suffix: &str, duration: Duration, sudo: bool, dry_run: bool, verbose: i32) -> Result<()> {
    let mut snapshots: Vec<&Snapshot> = snapshots.iter().filter(|s| s.name == cfg.name && s.suffix == suffix).collect();
    let last_snapshot = snapshots.last();

    if let Some(last_snapshot) = last_snapshot {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(last_snapshot.datetime);
        if elapsed < duration {
            return Ok(())
        }
    }

    let opts = SnapshotOpts {
        name: cfg.name.clone(),
        src: cfg.path.clone(),
        suffix: Some(String::from(suffix)),
        sudo,
        dry_run,
        verbose,
    };
    snapshot(&opts)?;

    if opts.dry_run {
        return Ok(());
    }

    let max_snapshot_count = match suffix {
        "hourly" => cfg.hourly,
        "daily" => cfg.daily,
        "weekly" => cfg.weekly,
        "monthly" => cfg.monthly,
        "yearly" => cfg.yearly,
        _ => 1_000_000,
    };

    while snapshots.len() > max_snapshot_count {
        if let Some(snapshot) = snapshots.first() {
            snapshot.delete(sudo, opts.verbose)?;
            info!("Deleted snapshot {}", &snapshot.full_name);
        }
        snapshots.remove(0);
    }

    Ok(())
}

pub struct Snapshot {
    pub full_name: String,
    pub name: String,
    pub suffix: String,
    pub datetime: DateTime<Utc>,
    pub repository: SnapshotRepositoryLocation,
}

impl Snapshot {
    fn delete(&self, sudo: bool, verbose: i32) -> Result<()> {
        let path = Path::new(&self.repository.path.clone()).join(&self.full_name).to_str().unwrap().to_string();
        let repository = &self.repository;
        let (program, args) = if let Some(host) = repository.host.clone() {
            if let Some(user) = repository.user.clone() {
                let base_url = format!("{}@{}", user, host);
                if let Some(port) = repository.port {
                    if sudo {
                        ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                    } else {
                        ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                    }
                } else {
                    if sudo {
                        ("ssh", vec![base_url, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                    } else {
                        ("ssh", vec![base_url, "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                    }
                }
            } else {
                if sudo {
                    ("ssh", vec![host, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                } else {
                    ("ssh", vec![host, "btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
                }
            }
        } else {
            if sudo {
                ("sudo", vec!["btrfs".to_string(), "subvolume".to_string(), "delete".to_string(), path.clone()])
            } else {
                ("btrfs", vec!["subvolume".to_string(), "delete".to_string(), path.clone()])
            }
        };

        let output = Command::new(program)
            .args(args)
            .output()?;

        if !output.status.success() {
            if repository.is_remote() {
                bail!("Could not delete remote snapshot at {}: {}", &path, str::from_utf8(&output.stderr).unwrap());
            } else {
                bail!("Could not delete snapshot at {}: {}", &path, str::from_utf8(&output.stderr).unwrap());
            }
        }

        let stdout = str::from_utf8(&output.stdout).unwrap();
        if verbose > 0 {
            info!("{}", stdout);
        }

        Ok(())
    }
}

#[derive(Clone,Debug,Default)]
pub struct SnapshotRepositoryLocation {
    user: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
}

impl SnapshotRepositoryLocation {
    fn is_remote(&self) -> bool {
        self.host.is_some()
    }
}

/// Parse snapshot repository location URL string into SnapshotRepositoryLocation instance
///
/// It will automatically determine whether the snapshot repository location is remote or local
/// based on whether or not the URL contains @ character which it assumes to mean there's a user
/// portion (e.g. user@192.168.0.1).
fn parse_snapshot_repository_location(url: &str, suffix: &str) -> Result<SnapshotRepositoryLocation> {
    let mut snapshot_repository_location = SnapshotRepositoryLocation::default();
    let tokens: Vec<&str> = url.split("@").collect();
    let is_remote = tokens.len() >= 2;

    if is_remote {
        snapshot_repository_location.user = Some(tokens[0].to_string());

        let tokens: Vec<&str> = tokens[1].split(":").collect();
        if tokens.len() >= 1 {
            snapshot_repository_location.host = Some(tokens[0].to_string());
        }
        if tokens.len() >= 2 {
            if let Ok(port) = tokens[1].parse::<u16>() {
                snapshot_repository_location.port = Some(port);
            } else if tokens.len() == 2 {
                snapshot_repository_location.port = Some(22);
                if suffix.is_empty() {
                    snapshot_repository_location.path = tokens[1].to_string();
                } else {
                    snapshot_repository_location.path = Path::new(tokens[1]).join(suffix).to_str().unwrap().to_string();
                }
            }
        }
        if tokens.len() >= 3 {
            if suffix.is_empty() {
                snapshot_repository_location.path = tokens[2].to_string();
            } else {
                snapshot_repository_location.path = Path::new(tokens[2]).join(suffix).to_str().unwrap().to_string();
            }
        }
    } else {
        if suffix.is_empty() {
            snapshot_repository_location.path = url.to_string();
        } else {
            snapshot_repository_location.path = Path::new(&url).join(suffix).to_str().unwrap().to_string();
        }
    }

    Ok(snapshot_repository_location)
}

pub fn list_snapshots(name: &str, repository: &SnapshotRepositoryLocation, sudo: bool, verbose: i32) -> Result<Vec<Snapshot>> {
    let (program, args) = if repository.is_remote() {
        debug!("Listing remote snapshots");
        if let Some(host) = repository.host.clone() {
            if let Some(user) = repository.user.clone() {
                let base_url = format!("{}@{}", user, host);
                if let Some(port) = repository.port {
                    if sudo {
                        ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                    } else {
                        ("ssh", vec!["-p".to_string(), format!("{}", port), base_url, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                    }
                } else {
                    if sudo {
                        ("ssh", vec![base_url, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                    } else {
                        ("ssh", vec![base_url, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                    }
                }
            } else {
                if sudo {
                    ("ssh", vec![host, "sudo".to_string(), "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                } else {
                    ("ssh", vec![host, "btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
                }
            }
        } else {
            bail!("Could not sync remotely without host specified!");
        }
    } else {
        debug!("Listing local snapshots");
        if sudo {
            ("sudo", vec!["btrfs".to_string(), "subvolume".to_string(), "list".to_string(), repository.path.clone()])
        } else {
            ("btrfs", vec!["subvolume".to_string(), "list".to_string(), repository.path.clone()])
        }
    };

    if verbose > 0 {
        debug!("{} {}", program, args.join(" "));
    }

    let output = Command::new(program)
        .args(args)
        .output()?;

    if !output.status.success() {
        bail!("Could not list snapshots with name {}: {}", name, str::from_utf8(&output.stderr).unwrap());
    }

    let stdout = str::from_utf8(&output.stdout).unwrap();
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

    Ok(parse_snapshot_list(&snapshot_list, repository)?)
}

/// Parses a list of snapshot name strings into list of Snapshot instances
fn parse_snapshot_list(snapshot_list: &[String], repository: &SnapshotRepositoryLocation) -> Result<Vec<Snapshot>> {
    let mut snapshots = Vec::new();

    for snapshot_name in snapshot_list {
        snapshots.push(parse_snapshot_name(snapshot_name, repository)?);
    }

    Ok(snapshots)
}

/// Transforms snapshot name string into a Snapshot instance
fn parse_snapshot_name(snapshot_name: &str, repository: &SnapshotRepositoryLocation) -> Result<Snapshot> {
    let mut split = snapshot_name.split("@");
    let name = split.next().ok_or_else(|| FridgeError::ParseSnapshot{what: "name", snapshot: snapshot_name.to_string()})?;
    let datetime_and_suffix = split.next().ok_or_else(|| FridgeError::ParseSnapshot{what: "datetime and suffix", snapshot: snapshot_name.to_string()})?;

    let mut split = datetime_and_suffix.split("_");
    let date = split.next().ok_or_else(|| FridgeError::ParseSnapshot{what: "date", snapshot: snapshot_name.to_string()})?;
    let time = split.next().ok_or_else(|| FridgeError::ParseSnapshot{what: "time", snapshot: snapshot_name.to_string()})?;
    let suffix = split.next().ok_or_else(|| FridgeError::ParseSnapshot{what: "suffix", snapshot: snapshot_name.to_string()})?;
    let datetime_str = format!("{} {}", date, time);
    let datetime = Utc.datetime_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")?;

    Ok(Snapshot{
        full_name: snapshot_name.to_string(),
        name: name.to_string(),
        suffix: suffix.to_string(),
        datetime,
        repository: repository.clone(),
    })
}

/// Finds parent snapshot to a specific snapshot in a list
fn find_parent_snapshot_to<'a>(name: &'a str, snapshot_list: &[&'a str]) -> Option<&'a str> {
    let mut p = None;

    for entry in snapshot_list {
        if *entry == name {
            break
        }
        p = Some(*entry);
    }

    p
}

#[cfg(test)]
mod tests {
    use chrono::{Utc,TimeZone};
    use super::*;

    #[test]
    fn test_snapshot() {
        let mut opts = SnapshotOpts::default();
        opts.src = String::from("/");
        opts.name = String::from("test");
        opts.dry_run = true;
        super::snapshot(&opts).unwrap();
    }

    #[test]
    fn test_parse_snapshot_repository_location() {
        {
            let dst = "root@192.168.1.2:22222:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, ".snapshots").unwrap();
            assert_eq!(snapshot_repository_location.user.as_ref().unwrap(), "root");
            assert_eq!(snapshot_repository_location.host.as_ref().unwrap(), "192.168.1.2");
            assert_eq!(snapshot_repository_location.port.unwrap(), 22222);
            assert_eq!(&snapshot_repository_location.path, "/home/.snapshots");
            assert_eq!(snapshot_repository_location.is_remote(), true);
        }
        {
            let dst = "root@192.168.1.2:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, ".snapshots").unwrap();
            assert_eq!(snapshot_repository_location.user.as_ref().unwrap(), "root");
            assert_eq!(snapshot_repository_location.host.as_ref().unwrap(), "192.168.1.2");
            assert_eq!(&snapshot_repository_location.path, "/home/.snapshots");
            assert_eq!(snapshot_repository_location.is_remote(), true);
        }
        {
            let dst = "192.168.1.2:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, ".snapshots").unwrap();
            assert_eq!(&snapshot_repository_location.path, "192.168.1.2:/home/.snapshots");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
        {
            let dst = "/run/media/EXTERNAL_HDD";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, ".snapshots").unwrap();
            assert_eq!(&snapshot_repository_location.path, "/run/media/EXTERNAL_HDD/.snapshots");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
        {
            let dst = "backup";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, ".snapshots").unwrap();
            assert_eq!(&snapshot_repository_location.path, "backup/.snapshots");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
        {
            let dst = "root@192.168.1.2:22222:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, "").unwrap();
            assert_eq!(snapshot_repository_location.user.as_ref().unwrap(), "root");
            assert_eq!(snapshot_repository_location.host.as_ref().unwrap(), "192.168.1.2");
            assert_eq!(snapshot_repository_location.port.unwrap(), 22222);
            assert_eq!(&snapshot_repository_location.path, "/home");
            assert_eq!(snapshot_repository_location.is_remote(), true);
        }
        {
            let dst = "root@192.168.1.2:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, "").unwrap();
            assert_eq!(snapshot_repository_location.user.as_ref().unwrap(), "root");
            assert_eq!(snapshot_repository_location.host.as_ref().unwrap(), "192.168.1.2");
            assert_eq!(&snapshot_repository_location.path, "/home");
            assert_eq!(snapshot_repository_location.is_remote(), true);
        }
        {
            let dst = "192.168.1.2:/home";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, "").unwrap();
            assert_eq!(&snapshot_repository_location.path, "192.168.1.2:/home");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
        {
            let dst = "/run/media/EXTERNAL_HDD";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, "").unwrap();
            assert_eq!(&snapshot_repository_location.path, "/run/media/EXTERNAL_HDD");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
        {
            let dst = "backup";
            let snapshot_repository_location = super::parse_snapshot_repository_location(dst, "").unwrap();
            assert_eq!(&snapshot_repository_location.path, "backup");
            assert_eq!(snapshot_repository_location.is_remote(), false);
        }
    }

    #[test]
    fn test_parse_snapshot_name() {
        let snapshot = super::parse_snapshot_name("root@2000-01-02_03:04:05_daily", "/.snapshots").unwrap();
        assert_eq!(&snapshot.name, "root");
        let datetime = Utc.datetime_from_str("2000-01-02 03:04:05", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(snapshot.datetime.timestamp(), datetime.timestamp());
        assert_eq!(snapshot.suffix, "daily");
    }
}
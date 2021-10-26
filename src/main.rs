pub mod lib;

use clap::Parser;

use fridge::{snapshot,sync,restore,list,SnapshotOpts,SyncOpts,ListOpts};

#[derive(Parser)]
#[clap(version = "0.0.1", author = "Lilis Iskandar <lilis@veand.co>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Snapshot(SnapshotSubCommand),
    Sync(SyncSubCommand),
    Restore(RestoreSubCommand),
    List(ListSubCommand),
}

/// A subcommand for snapshot
#[derive(Parser)]
struct SnapshotSubCommand {
    /// Base name snapshot subvolume
    name: String,
    /// Source subvolume to snapshot
    src: String,
    /// Suffix to append at end of full snapshot name 
    suffix: Option<String>,
    /// Dry run
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool,
}

/// A subcommand for sync
#[derive(Parser)]
struct SyncSubCommand {
    /// Name
    name: String,
    /// Source
    src: String,
    /// Destination
    dst: String,
    /// Append suffix to source directory
    #[clap(long("append-suffix-src"), default_value(".snapshots"))]
    append_suffix_src: String,
    /// Append suffix to destination directory
    #[clap(long("append-suffix-dst"), default_value(".snapshots"))]
    append_suffix_dst: String,
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool,
}

/// A subcommand for restore
#[derive(Parser)]
struct RestoreSubCommand {
    /// Source
    src: String,
    /// Destination
    dst: String,
    /// Dry run
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool
}

/// A subcommand for list
#[derive(Parser)]
struct ListSubCommand {
    /// Snapshot name
    name: String,
    /// Path to snapshot
    path: String,
    /// Append suffix
    #[clap(long("append-suffix"), default_value(".snapshots"))]
    append_suffix: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Snapshot(t) => {
            let mut opts = SnapshotOpts::default();
            opts.src = t.src;
            opts.snapshot_name = t.name;
            opts.suffix = t.suffix;
            opts.dry_run = t.dry_run;
            crate::snapshot(opts).unwrap();
        }
        SubCommand::Sync(t) => {
            let mut opts = SyncOpts::default();
            opts.name = t.name;
            opts.src = t.src;
            opts.dst = t.dst;
            opts.append_suffix_src = t.append_suffix_src;
            opts.append_suffix_dst = t.append_suffix_dst;
            opts.dry_run = t.dry_run;
            crate::sync(opts).unwrap();
        }
        SubCommand::Restore(t) => {
            crate::restore(&t.src, &t.dst, t.dry_run).unwrap();
        }
        SubCommand::List(t) => {
            let sub_opts = ListOpts {
                name: t.name,
                path: t.path,
                append_suffix: t.append_suffix,
                verbose: opts.verbose,
            };
            crate::list(sub_opts).unwrap();
        }
    }
}
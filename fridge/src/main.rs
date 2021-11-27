use clap::Parser;

use fridge::{snapshot,sync,restore,list,run,SnapshotOpts,SyncOpts,ListOpts,RunOpts,RestoreOpts};

#[derive(Parser)]
#[clap(version = "0.0.1", author = "Lilis Iskandar <lilis@veand.co>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Snapshot(SnapshotSubCommand),
    Sync(SyncSubCommand),
    Restore(RestoreSubCommand),
    List(ListSubCommand),
    Run(RunSubCommand),
}

/// A subcommand for snapshot
#[derive(Parser)]
struct SnapshotSubCommand {
    /// Base name snapshot subvolume
    name: String,
    /// Source subvolume to snapshot
    src: String,
    /// Suffix for the snapshot
    suffix: Option<String>,
    /// Sudo
    #[clap(long("sudo"))]
    sudo: bool,
    /// Dry run
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
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
    /// Use sudo at src
    #[clap(long("src-sudo"))]
    src_sudo: bool,
    /// Use sudo at dst
    #[clap(long("dst-sudo"))]
    dst_sudo: bool,
    /// Append suffix to source directory
    #[clap(long("src-suffix"), default_value(".snapshots"))]
    src_suffix: String,
    /// Append suffix to destination directory
    #[clap(long("dst-suffix"), default_value(".snapshots"))]
    dst_suffix: String,
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
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
    dry_run: bool,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

/// A subcommand for list
#[derive(Parser)]
struct ListSubCommand {
    /// Snapshot name
    name: String,
    /// Path to snapshot
    path: String,
    /// Use sudo
    #[clap(long("sudo"))]
    sudo: bool,
    /// Append suffix
    #[clap(long("suffix"), default_value(".snapshots"))]
    suffix: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

/// A subcommand for run
#[derive(Parser)]
struct RunSubCommand {
    /// Dry run
    #[clap(short('n'), long("dry-run"))]
    dry_run: bool,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Snapshot(t) => {
            let mut opts = SnapshotOpts::default();
            opts.src = t.src;
            opts.name = t.name;
            opts.suffix = t.suffix;
            opts.dry_run = t.dry_run;
            opts.verbose = t.verbose;
            snapshot(&opts).unwrap();
        }
        SubCommand::Sync(t) => {
            let mut opts = SyncOpts::default();
            opts.name = t.name;
            opts.src = t.src;
            opts.dst = t.dst;
            opts.src_sudo = t.src_sudo;
            opts.dst_sudo = t.dst_sudo;
            opts.src_suffix = t.src_suffix;
            opts.dst_suffix = t.dst_suffix;
            opts.dry_run = t.dry_run;
            opts.verbose = t.verbose;
            sync(&opts).unwrap();
        }
        SubCommand::Restore(t) => {
            let mut opts = RestoreOpts::default();
            opts.src = t.src;
            opts.dst = t.dst;
            opts.dry_run = t.dry_run;
            opts.verbose = t.verbose;
            restore(&opts).unwrap();
        }
        SubCommand::List(t) => {
            let sub_opts = ListOpts {
                name: t.name,
                path: t.path,
                suffix: t.suffix,
                sudo: t.sudo,
                verbose: t.verbose,
            };
            list(&sub_opts).unwrap();
        }
        SubCommand::Run(t) => {
            let sub_opts = RunOpts {
                dry_run: t.dry_run,
                verbose: t.verbose,
            };
            run(&sub_opts).unwrap();
        }
    }
}
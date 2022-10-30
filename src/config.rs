use std::fs;

use anyhow::{Result,bail};
use log::error;
use serde::Deserialize;

const DEFAULT_HOURLY: usize = 24;
const DEFAULT_DAILY: usize = 7;
const DEFAULT_WEEKLY: usize = 4;
const DEFAULT_MONTHLY: usize = 12;
const DEFAULT_YEARLY: usize = 3;

use crate::fridge::SnapshotOpts;

#[derive(Clone, Debug, Deserialize, PartialEq, Default)]
pub struct Config {
	pub local: LocalConfig,
	pub snapshots: Vec<SnapshotConfig>,
	pub remotes: Vec<RemoteConfig>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Default)]
pub struct LocalConfig {
	pub sudo: bool,
	pub path: String,
	pub suffix: String,
}

lazy_static! {
	static ref DEFAULT_CONFIG: Config = Config {
		local: LocalConfig {
			sudo: false,
			path: "/".to_string(),
			suffix: ".snapshots".to_string(),
		},
		snapshots: vec![
			SnapshotConfig {
				name: "root".to_string(),
				path: "/".to_string(),
				hourly: DEFAULT_HOURLY,
				daily: DEFAULT_DAILY,
				weekly: DEFAULT_WEEKLY,
				monthly: DEFAULT_MONTHLY,
				yearly: DEFAULT_YEARLY,
			},
			SnapshotConfig {
				name: "home".to_string(),
				path: "/home".to_string(),
				hourly: DEFAULT_HOURLY,
				daily: DEFAULT_DAILY,
				weekly: DEFAULT_WEEKLY,
				monthly: DEFAULT_MONTHLY,
				yearly: DEFAULT_YEARLY,
			},
		],
		remotes: vec![],
	};
}

#[derive(Clone, Debug, Deserialize, PartialEq, Default)]
pub struct SnapshotConfig {
	pub name: String,
	pub path: String,
	pub hourly: usize,
	pub daily: usize,
	pub weekly: usize,
	pub monthly: usize,
	pub yearly: usize,
}

impl SnapshotConfig {
	pub fn to_snapshot_opts(&self, suffix: Option<&str>, sudo: bool, dry_run: bool, verbose: i32) -> SnapshotOpts {
		SnapshotOpts {
			src: self.path.clone(),
			name: self.name.clone(),
			suffix: suffix.map(|v| v.to_string()),
			sudo,
			dry_run,
			verbose,
		}
	}
}

#[derive(Clone, Debug, Deserialize, PartialEq, Default)]
pub struct RemoteConfig {
	pub user: Option<String>,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub path: String,
	pub sudo: bool,
	pub suffix: String,
}

impl RemoteConfig {
	pub fn location_string(&self) -> Result<String> {
		if let Some(user) = &self.user {
			if let Some(host) = &self.host {
				if let Some(port) = &self.port {
					Ok(format!("{}@{}:{}:{}", user, host, port, self.path))
				} else {
					Ok(format!("{}@{}:{}", user, host, self.path))
				}
			} else {
				bail!("Could not convert remote config into location string")
			}
		} else {
			Ok(self.path.clone())
		}
	}
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawConfig {
	local: Option<RawLocalConfig>,
	snapshots: Option<Vec<RawSnapshotConfig>>,
	remotes: Option<Vec<RawRemoteConfig>>,
}

impl From<RawConfig> for Config {
	fn from(raw: RawConfig) -> Self {
		Self {
			local: raw.local.map_or(LocalConfig::default(), |local| local.into()),
			snapshots: raw.snapshots.map_or(Vec::new(), |snapshots| snapshots.iter().map(|v| SnapshotConfig::from(v)).collect()),
			remotes: raw.remotes.map_or(Vec::new(), |remotes| remotes.iter().map(|v| RemoteConfig::from(v)).collect()),
		}
	}
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawLocalConfig {
	sudo: bool,
	path: Option<String>,
	suffix: Option<String>,
}

impl From<RawLocalConfig> for LocalConfig {
	fn from(raw: RawLocalConfig) -> Self {
		Self {
			sudo: raw.sudo,
			path: raw.path.unwrap_or("/".to_string()),
			suffix: raw.suffix.unwrap_or(".snapshots".to_string()),
		}
	}
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawSnapshotConfig {
	name: String,
	path: String,
	hourly: Option<usize>,
	daily: Option<usize>,
	weekly: Option<usize>,
	monthly: Option<usize>,
	yearly: Option<usize>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawRemoteConfig {
	user: Option<String>,
	host: Option<String>,
	port: Option<u16>,
	path: Option<String>,
	suffix: Option<String>,
	sudo: Option<bool>,
}

impl From<RawSnapshotConfig> for SnapshotConfig {
	fn from(raw: RawSnapshotConfig) -> Self {
		SnapshotConfig{
			name: raw.name,
			path: raw.path,
			hourly: raw.hourly.unwrap_or(0),
			daily: raw.daily.unwrap_or(0),
			weekly: raw.weekly.unwrap_or(0),
			monthly: raw.monthly.unwrap_or(0),
			yearly: raw.yearly.unwrap_or(0),
		}
	}
}

impl From<&RawSnapshotConfig> for SnapshotConfig {
	fn from(raw: &RawSnapshotConfig) -> Self {
		SnapshotConfig{
			name: raw.name.clone(),
			path: raw.path.clone(),
			hourly: raw.hourly.unwrap_or(0),
			daily: raw.daily.unwrap_or(0),
			weekly: raw.weekly.unwrap_or(0),
			monthly: raw.monthly.unwrap_or(0),
			yearly: raw.yearly.unwrap_or(0),
		}
	}
}

impl From<RawRemoteConfig> for RemoteConfig {
	fn from(raw: RawRemoteConfig) -> Self {
		RemoteConfig{
			user: raw.user,
			host: raw.host,
			port: raw.port,
			path: raw.path.unwrap_or("/".to_string()),
			suffix: raw.suffix.unwrap_or(".snapshots".to_string()),
			sudo: raw.sudo.unwrap_or(false),
		}
	}
}

impl From<&RawRemoteConfig> for RemoteConfig {
	fn from(raw: &RawRemoteConfig) -> Self {
		RemoteConfig{
			user: raw.user.clone(),
			host: raw.host.clone(),
			port: raw.port,
			path: raw.path.clone().unwrap_or("/".to_string()),
			suffix: raw.suffix.clone().unwrap_or(".snapshots".to_string()),
			sudo: raw.sudo.unwrap_or(false),
		}
	}
}

pub fn load() -> Config {
	let path = "/etc/fridge/fridge.toml";
	match parse_config_at(path) {
		Ok(config) => config,
		Err(e) => {
			error!("Could not load configuration file at {}: {}", path, e);
			DEFAULT_CONFIG.clone()
		}
	}
}

fn parse_config_at(path: &str) -> Result<Config> {
	let s = fs::read_to_string(path)?;
	let config: RawConfig = toml::from_str(&s)?;
	Ok(config.into())
}

#[cfg(test)]
mod tests {

use super::*;

static SAMPLE_CONFIG: &'static str = r#"
[local]
sudo = true

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
hourly = 0
daily = 7
weekly = 4
monthly = 12
yearly = 0

[[remotes]]
path = "/run/media/LILIS_5T/.snapshots"
suffix = "ThinkPad-T495"

[[remotes]]
user = "li"
host = "192.168.0.2"
port = 22
sudo = true
suffix = "ThinkPad-T495"
"#;

#[test]
fn test_parse_config_str() {
	let config: RawConfig = toml::from_str(SAMPLE_CONFIG).unwrap();
	assert_eq!(config, RawConfig{
		local: Some(RawLocalConfig {
			sudo: true,
			path: None,
			suffix: None,
		}),
		snapshots: Some(vec![
			RawSnapshotConfig {
				name: "root".to_string(),
				path: "/".to_string(),
				hourly: Some(24),
				daily: Some(7),
				weekly: Some(4),
				monthly: Some(12),
				yearly: Some(3),
			},
			RawSnapshotConfig {
				name: "home".to_string(),
				path: "/home".to_string(),
				hourly: Some(0),
				daily: Some(7),
				weekly: Some(4),
				monthly: Some(12),
				yearly: Some(0),
			},
		]),
		remotes: Some(vec![
			RawRemoteConfig {
				user: None,
				host: None,
				port: None,
				path: Some("/run/media/LILIS_5T/.snapshots".to_string()),
				sudo: None,
				suffix: Some("ThinkPad-T495".to_string()),
			},
			RawRemoteConfig {
				user: Some("li".to_string()),
				host: Some("192.168.0.2".to_string()),
				port: Some(22),
				path: None,
				sudo: Some(true),
				suffix: Some("ThinkPad-T495".to_string()),
			},
		])
	})
}

}

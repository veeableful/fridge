use std::fs;
use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
				hourly: 24,
				daily: 7,
				weekly: 4,
				monthly: 12,
				yearly: 3,
				sync_hourly: false,
				sync_daily: true,
				sync_weekly: false,
				sync_monthly: false,
				sync_yearly: false,
			},
			SnapshotConfig {
				name: "home".to_string(),
				path: "/home".to_string(),
				hourly: 0,
				daily: 7,
				weekly: 4,
				monthly: 12,
				yearly: 0,
				sync_hourly: false,
				sync_daily: true,
				sync_weekly: false,
				sync_monthly: false,
				sync_yearly: false,
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
	pub sync_hourly: bool,
	pub sync_daily: bool,
	pub sync_weekly: bool,
	pub sync_monthly: bool,
	pub sync_yearly: bool,
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

#[derive(Debug, Deserialize, PartialEq)]
struct RawConfig {
	local: Option<RawLocalConfig>,
	snapshots: Option<Vec<RawSnapshotConfig>>,
	remotes: Option<Vec<RawRemoteConfig>>,
}

impl From<RawConfig> for Config {
	fn from(raw: RawConfig) -> Self {
		Self {
			local: if let Some(local) = raw.local { local.into() } else { LocalConfig::default() },
			snapshots: if let Some(snapshots) = raw.snapshots { snapshots.iter().map(|v| SnapshotConfig::from(v)).collect() } else { Vec::new() },
			remotes: if let Some(remotes) = raw.remotes { remotes.iter().map(|v| RemoteConfig::from(v)).collect() } else { Vec::new() },
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
			path: if let Some(path) = raw.path { path } else { "/".to_string() },
			suffix: if let Some(suffix) = raw.suffix { suffix } else { ".snapshots".to_string() },
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
	sync_hourly: Option<bool>,
	sync_daily: Option<bool>,
	sync_weekly: Option<bool>,
	sync_monthly: Option<bool>,
	sync_yearly: Option<bool>,
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
			hourly: if let Some(hourly) = raw.hourly { hourly } else { 0 },
			daily: if let Some(daily) = raw.daily { daily } else { 0 },
			weekly: if let Some(weekly) = raw.weekly { weekly } else { 0 },
			monthly: if let Some(monthly) = raw.monthly { monthly } else { 0 },
			yearly: if let Some(yearly) = raw.yearly { yearly } else { 0 },
			sync_hourly: if let Some(sync_hourly) = raw.sync_hourly { sync_hourly } else { false },
			sync_daily: if let Some(sync_daily) = raw.sync_daily { sync_daily } else { false },
			sync_weekly: if let Some(sync_weekly) = raw.sync_weekly { sync_weekly } else { false },
			sync_monthly: if let Some(sync_monthly) = raw.sync_monthly { sync_monthly } else { false },
			sync_yearly: if let Some(sync_yearly) = raw.sync_yearly { sync_yearly } else { false },
		}
	}
}

impl From<&RawSnapshotConfig> for SnapshotConfig {
	fn from(raw: &RawSnapshotConfig) -> Self {
		SnapshotConfig{
			name: raw.name.clone(),
			path: raw.path.clone(),
			hourly: if let Some(hourly) = raw.hourly { hourly } else { 0 },
			daily: if let Some(daily) = raw.daily { daily } else { 0 },
			weekly: if let Some(weekly) = raw.weekly { weekly } else { 0 },
			monthly: if let Some(monthly) = raw.monthly { monthly } else { 0 },
			yearly: if let Some(yearly) = raw.yearly { yearly } else { 0 },
			sync_hourly: if let Some(sync_hourly) = raw.sync_hourly { sync_hourly } else { false },
			sync_daily: if let Some(sync_daily) = raw.sync_daily { sync_daily } else { false },
			sync_weekly: if let Some(sync_weekly) = raw.sync_weekly { sync_weekly } else { false },
			sync_monthly: if let Some(sync_monthly) = raw.sync_monthly { sync_monthly } else { false },
			sync_yearly: if let Some(sync_yearly) = raw.sync_yearly { sync_yearly } else { false },
		}
	}
}

impl From<RawRemoteConfig> for RemoteConfig {
	fn from(raw: RawRemoteConfig) -> Self {
		RemoteConfig{
			user: raw.user,
			host: raw.host,
			port: raw.port,
			path: if let Some(path) = raw.path { path } else { "/".to_string() },
			suffix: if let Some(suffix) = raw.suffix { suffix } else { ".snapshots".to_string() },
			sudo: if let Some(sudo) = raw.sudo { sudo } else { false },
		}
	}
}

impl From<&RawRemoteConfig> for RemoteConfig {
	fn from(raw: &RawRemoteConfig) -> Self {
		RemoteConfig{
			user: raw.user.clone(),
			host: raw.host.clone(),
			port: raw.port,
			path: if let Some(path) = &raw.path { path.clone() } else { "/".to_string() },
			suffix: if let Some(suffix) = &raw.suffix { suffix.clone() } else { ".snapshots".to_string() },
			sudo: if let Some(sudo) = raw.sudo { sudo } else { false },
		}
	}
}

pub fn load() -> Config {
	if let Ok(config) = parse_config_at("/etc/fridge/fridge.toml") {
		config
	} else {
		DEFAULT_CONFIG.clone()
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
sync_daily = true

[[snapshots]]
name = "home"
path = "/home"
hourly = 0
daily = 7
weekly = 4
monthly = 12
yearly = 0
sync_daily = true

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
				sync_hourly: None,
				sync_daily: Some(true),
				sync_weekly: None,
				sync_monthly: None,
				sync_yearly: None,
			},
			RawSnapshotConfig {
				name: "home".to_string(),
				path: "/home".to_string(),
				hourly: Some(0),
				daily: Some(7),
				weekly: Some(4),
				monthly: Some(12),
				yearly: Some(0),
				sync_hourly: None,
				sync_daily: Some(true),
				sync_weekly: None,
				sync_monthly: None,
				sync_yearly: None,
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
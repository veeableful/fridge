mod imp;

use adw::Application;
use adw::subclass::prelude::ObjectSubclassIsExt;
use anyhow::Result;
use chrono::{Duration, Utc};
use glib::Object;
use gtk::prelude::SettingsExt;
use gtk::traits::{WidgetExt, ButtonExt};
use gtk::{gio, glib};
use gtk::gio::Settings;
use gtk::glib::{clone, g_log, LogLevel};
use log::{info};
use zbus::blocking::Connection;
use zbus_polkit::policykit1::*;

use crate::APP_ID;
use crate::fridge::{list_snapshots, parse_sync_location, find_parent_snapshot_to, transfer, TransferOpts};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        //Object::new(&[("application", app)]).expect("Could not create Window")
        Object::builder::<Self>().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("Could not set `Settings`");
    }


    fn settings(&self) -> &Settings {
        self.imp().settings.get().expect("Could not get settings.")
    }

    fn setup_callbacks(&self) {
        self.imp().snapshot_button.connect_clicked(
            clone!(@weak self as window => move |_| {
                if let Err(e) = window.snapshot() {
                    g_log!(LogLevel::Error, "Could not take snapshot: {e}");
                }
            }),
        );
        self.imp().backup_button.connect_clicked(
            clone!(@weak self as window => move |_| {
                if let Err(e) = window.backup() {
                    g_log!(LogLevel::Error, "Could not do backup: {e}");
                }
            }),
        );
    }

    fn refresh_last_snapshot_label(&self) -> Result<()> {
        let dst = crate::fridge::SnapshotRepositoryLocation{
            user: None,
            host: None,
            port: None,
            path: "/.snapshots".to_string(),
        };
        let snapshots = list_snapshots("root", &dst, true, 0)?;
        if snapshots.len() == 0 {
            self.imp().last_snapshot_label.set_label("No snapshots found");
            return Ok(())
        }

        let last_snapshot = snapshots.last();
        if let Some(last_snapshot) = last_snapshot {
            let datetime_str = last_snapshot.datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            let label = format!("Last snapshot at {datetime_str}");
            self.imp().last_snapshot_label.set_label(&label);
        }

        Ok(())
    }

    fn authenticate(&self) -> Result<bool> {
        let connection = Connection::system()?;
        let proxy = AuthorityProxyBlocking::new(&connection)?;
        let subject = Subject::new_for_owner(std::process::id(), None, None)?;
        let result = proxy.check_authorization(
            &subject,
            "org.freedesktop.policykit.exec",
            &std::collections::HashMap::new(),
            CheckAuthorizationFlags::AllowUserInteraction.into(),
            "",
        )?;

        let authorized = result.is_authorized;
        if authorized {
            self.imp().snapshot_button.set_sensitive(true);
            self.imp().backup_button.set_sensitive(true);
        }

        Ok(authorized)
    }

    fn snapshot(&self) -> Result<()> {
        let settings = self.settings();
        let max_hourly_snapshots = settings.uint("max-hourly-snapshots") as usize;
        Self::do_snapshot("root", "/", "/.snapshots", "hourly", max_hourly_snapshots, Duration::hours(1))?;
        Self::do_snapshot("home", "/home", "/home/.snapshots", "hourly", max_hourly_snapshots, Duration::hours(1))?;
        self.refresh_last_snapshot_label()?;

        Ok(())
    }

    fn backup(&self) -> Result<()> {
        let settings = self.settings();
        let remote_user_value = settings.string("remote-user");
        let remote_host_value = settings.string("remote-host");
        let remote_directory_value = settings.string("remote-directory");
        let remote_user = remote_user_value.as_str();
        let remote_host = remote_host_value.as_str();
        let remote_directory = remote_directory_value.as_str();
        let dst = format!("{remote_user}@{remote_host}:22:{remote_directory}");
        Self::do_backup("root", "/.snapshots", &dst)?;
        Self::do_backup("home", "/home/.snapshots", &dst)?;

        Ok(())
    }

    fn do_snapshot(name: &str, src: &str, dst: &str, suffix: &str, max_snapshot_count: usize, duration: Duration) -> Result<()> {
        let name = name.to_string();
        let src = src.to_string();
        let dst = crate::fridge::SnapshotRepositoryLocation{
            user: None,
            host: None,
            port: None,
            path: dst.to_string(),
        };
        let mut snapshots = list_snapshots(&name, &dst, true, 0)?;
        let last_snapshot = snapshots.last();
        if let Some(last_snapshot) = last_snapshot {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_snapshot.datetime);
            if elapsed < duration {
                return Ok(())
            }
        }
        let opts = crate::fridge::SnapshotOpts{
            src,
            name,
            suffix: Some(suffix.to_string()),
            sudo: true,
            dry_run: false,
            verbose: 0,
        };
        crate::fridge::snapshot(&opts)?;

        while snapshots.len() > max_snapshot_count {
            if let Some(snapshot) = snapshots.first() {
                snapshot.delete(true, 0)?;
                info!("Deleted snapshot {}", &snapshot.full_name);
            }
            snapshots.remove(0);
        }

        Ok(())
    }

    fn do_backup(name: &str, src: &str, dst: &str) -> Result<()> {
        let src = parse_sync_location(src)?;
        let dst = parse_sync_location(dst)?;
        let src_list_string = list_snapshots(name, &src, true, 0)?;
        let dst_list_string = list_snapshots(name, &dst, true, 0)?;
        let src_list: Vec<&str> = src_list_string.iter().map(|s| s.full_name.as_str()).collect();
        let dst_list: Vec<&str> = dst_list_string.iter().map(|s| s.full_name.as_str()).collect();
        g_log!(LogLevel::Debug,"Source snapshot count: {}", src_list.len());
        g_log!(LogLevel::Debug,"Destination snapshot count: {}", dst_list.len());
        let missing_snapshots_in_destination: Vec<&str> = src_list.iter()
            .filter(|snapshot_name| !dst_list.contains(snapshot_name))
            .map(|snapshot_name| *snapshot_name)
            .collect();
        if missing_snapshots_in_destination.len() == 0 {
            g_log!(LogLevel::Info, "Remote snapshots are up-to-date");
            return Ok(());
        }
        let mut parent: Option<&str> = find_parent_snapshot_to(missing_snapshots_in_destination[0], &src_list);

        for snapshot in missing_snapshots_in_destination {
            let mut transfer_opts = TransferOpts::default();
            transfer_opts.parent_snapshot = parent.map_or(None, |s| Some(s.to_string()));
            transfer_opts.snapshot = snapshot.to_string();
            transfer_opts.src = src.clone();
            transfer_opts.src_sudo = true;
            transfer_opts.dst = dst.clone();
            transfer_opts.dst_sudo = true;
            transfer_opts.dry_run = false;
            transfer_opts.verbose = 0;
            g_log!(LogLevel::Info, "Sending snapshot {}", snapshot);
            transfer(&transfer_opts)?;
            parent = Some(snapshot);
        }

        Ok(())
    }
}

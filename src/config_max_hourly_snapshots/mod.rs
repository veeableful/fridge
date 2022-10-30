mod imp;

use gtk::glib;
use gtk::gio::{Settings, SettingsBindFlags};
use gtk::prelude::SettingsExtManual;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct ConfigMaxHourlySnapshots(ObjectSubclass<imp::ConfigMaxHourlySnapshots>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ConfigMaxHourlySnapshots {
    fn setup_settings(&self) {
        let settings = Settings::new(crate::APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("Could not set `Settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp().settings.get().expect("Could not get settings.")
    }

    fn bind_settings(&self) {
        let spin = self.imp().spin.get();
        self.settings()
            .bind("max-hourly-snapshots", &spin, "value")
            .flags(SettingsBindFlags::DEFAULT)
            .build();
    }
}
mod imp;

use gtk::glib;
use gtk::gio::{Settings, SettingsBindFlags};
use gtk::prelude::SettingsExtManual;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct ConfigRemoteHost(ObjectSubclass<imp::ConfigRemoteHost>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ConfigRemoteHost {
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
        let entry = self.imp().entry.get();
        self.settings()
            .bind("remote-host", &entry, "text")
            .flags(SettingsBindFlags::DEFAULT)
            .build();
    }
}
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::glib::subclass::InitializingObject;

use crate::config_hourly_snapshot::ConfigHourlySnapshot;
use crate::config_max_hourly_snapshots::ConfigMaxHourlySnapshots;
use crate::config_remote_backup::ConfigRemoteBackup;
use crate::config_remote_user::ConfigRemoteUser;
use crate::config_remote_host::ConfigRemoteHost;
use crate::config_remote_directory::ConfigRemoteDirectory;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/PreferencesWindow.ui")]
pub struct PreferencesWindow {
    #[template_child]
    pub hourly_snapshot_schedule: TemplateChild<ConfigHourlySnapshot>,
    #[template_child]
    pub max_hourly_snapshots: TemplateChild<ConfigMaxHourlySnapshots>,
    #[template_child]
    pub remote_backup: TemplateChild<ConfigRemoteBackup>,
    #[template_child]
    pub remote_user: TemplateChild<ConfigRemoteUser>,
    #[template_child]
    pub remote_host: TemplateChild<ConfigRemoteHost>,
    #[template_child]
    pub remote_directory: TemplateChild<ConfigRemoteDirectory>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for PreferencesWindow {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "PreferencesWindow";
    type Type = super::PreferencesWindow;
    type ParentType = adw::PreferencesWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for PreferencesWindow {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
    }
}

// Trait shared by all widgets
impl WidgetImpl for PreferencesWindow {}

impl PreferencesWindowImpl for PreferencesWindow {}

impl WindowImpl for PreferencesWindow {}

impl AdwWindowImpl for PreferencesWindow {}

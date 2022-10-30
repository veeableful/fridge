mod imp;

use adw::Application;
use gtk::{glib::{self, Object}};

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new(app: &Application) -> Self {
        // Create new window
        //Object::new(&[("application", app)]).expect("Could not create PreferencesWindow")
        Object::builder::<Self>().property("application", app).build()
    }
}

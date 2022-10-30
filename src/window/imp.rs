use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::gio::Settings;
use gtk::glib::subclass::InitializingObject;
use once_cell::sync::OnceCell;

use crate::header_bar::HeaderBar;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/Window.ui")]
pub struct Window {
    #[template_child]
    pub header: TemplateChild<HeaderBar>,
    #[template_child]
    pub last_snapshot_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub snapshot_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub backup_button: TemplateChild<gtk::Button>,
    pub settings: OnceCell<Settings>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Window";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        let obj = self.obj();
        match obj.authenticate() {
            Ok(authorized) => {
                if !authorized {
                    obj.close();
                }
            },
            Err(_) => {
                obj.close();
            },
        }

        obj.setup_settings();
        obj.setup_callbacks();
        obj.refresh_last_snapshot_label().unwrap();
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

// Trait shared by all application windows
impl AdwApplicationWindowImpl for Window {}

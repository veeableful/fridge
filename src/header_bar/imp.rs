use adw::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::glib::subclass::InitializingObject;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/HeaderBar.ui")]
pub struct HeaderBar {
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for HeaderBar {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "HeaderBar";
    type Type = super::HeaderBar;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for HeaderBar {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
    }
}

// Trait shared by all widgets
impl WidgetImpl for HeaderBar {}

impl BoxImpl for HeaderBar {}

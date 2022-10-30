use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::glib::subclass::InitializingObject;
use gtk::gio::Settings;
use once_cell::sync::{Lazy, OnceCell};
use std::cell::RefCell;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/ConfigRemoteDirectory.ui")]
pub struct ConfigRemoteDirectory {
    #[template_child]
    pub entry: TemplateChild<gtk::Entry>,
    pub text: RefCell<String>,
    pub settings: OnceCell<Settings>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ConfigRemoteDirectory {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "ConfigRemoteDirectory";
    type Type = super::ConfigRemoteDirectory;
    type ParentType = adw::ActionRow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for ConfigRemoteDirectory {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        let obj = self.obj();
        obj.setup_settings();
        obj.bind_settings();
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> =
            Lazy::new(|| vec![
                ParamSpecString::builder("text").build(),
            ]);
        PROPERTIES.as_ref()
    }

    fn set_property( &self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "text" => {
                let text = value.get().expect("The value needs to be of type `String`.");
                self.text.replace(text);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "text" => self.text.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

// Trait shared by all widgets
impl WidgetImpl for ConfigRemoteDirectory {}

impl ListBoxRowImpl for ConfigRemoteDirectory {}

impl PreferencesRowImpl for ConfigRemoteDirectory {}

impl ActionRowImpl for ConfigRemoteDirectory {}

#[gtk::template_callbacks]
impl ConfigRemoteDirectory {
    #[template_callback]
    fn on_changed(editable: gtk::Editable) {
        eprintln!("ConfigRemoteDirectory: {}", editable.text());
    }
}

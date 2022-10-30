use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::Switch;
use gtk::{glib, CompositeTemplate};
use gtk::glib::{ParamSpec, ParamSpecBoolean, Value};
use gtk::glib::subclass::InitializingObject;
use gtk::gio::Settings;
use once_cell::sync::{Lazy, OnceCell};
use std::cell::Cell;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/ConfigHourlySnapshot.ui")]
pub struct ConfigHourlySnapshot {
    #[template_child]
    pub switch: TemplateChild<Switch>,
    pub state: Cell<bool>,
    pub active: Cell<bool>,
    pub settings: OnceCell<Settings>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ConfigHourlySnapshot {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "ConfigHourlySnapshot";
    type Type = super::ConfigHourlySnapshot;
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
impl ObjectImpl for ConfigHourlySnapshot {
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
                ParamSpecBoolean::builder("state").build(),
                ParamSpecBoolean::builder("active").build(),
            ]);
        PROPERTIES.as_ref()
    }

    fn set_property( &self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "state" => {
                let state = value.get().expect("The value needs to be of type `bool`.");
                self.state.replace(state);
            }
            "active" => {
                let active = value.get().expect("The value needs to be of type `bool`.");
                self.active.replace(active);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "state" => self.state.get().to_value(),
            "active" => self.active.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

// Trait shared by all widgets
impl WidgetImpl for ConfigHourlySnapshot {}

impl ListBoxRowImpl for ConfigHourlySnapshot {}

impl PreferencesRowImpl for ConfigHourlySnapshot {}

impl ActionRowImpl for ConfigHourlySnapshot {}

#[gtk::template_callbacks]
impl ConfigHourlySnapshot {
    #[template_callback]
    fn on_state_set(switch: Switch) -> bool {
        eprintln!("ConfigHourlySnapshot: {}", switch.is_active());
        return false;
    }
}

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::SpinButton;
use gtk::{glib, CompositeTemplate};
use gtk::glib::{ParamSpec, ParamSpecUInt, Value};
use gtk::glib::subclass::InitializingObject;
use gtk::gio::Settings;
use once_cell::sync::{Lazy, OnceCell};
use std::cell::Cell;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/co/veand/fridge/ConfigMaxHourlySnapshots.ui")]
pub struct ConfigMaxHourlySnapshots {
    #[template_child]
    pub spin: TemplateChild<SpinButton>,
    pub value: Cell<u32>,
    pub settings: OnceCell<Settings>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ConfigMaxHourlySnapshots {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "ConfigMaxHourlySnapshots";
    type Type = super::ConfigMaxHourlySnapshots;
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
impl ObjectImpl for ConfigMaxHourlySnapshots {
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
                ParamSpecUInt::builder("value").build(),
            ]);
        PROPERTIES.as_ref()
    }

    fn set_property( &self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "value" => {
                let state = value.get().expect("The value needs to be of type `u32`.");
                self.value.replace(state);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "value" => self.value.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

// Trait shared by all widgets
impl WidgetImpl for ConfigMaxHourlySnapshots {}

impl ListBoxRowImpl for ConfigMaxHourlySnapshots {}

impl PreferencesRowImpl for ConfigMaxHourlySnapshots {}

impl ActionRowImpl for ConfigMaxHourlySnapshots {}

#[gtk::template_callbacks]
impl ConfigMaxHourlySnapshots {
    #[template_callback]
    fn on_value_changed(spin: SpinButton) {
        eprintln!("ConfigMaxHourlySnapshots: {}", spin.value_as_int());
    }
}

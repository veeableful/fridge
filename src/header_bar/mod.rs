mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct HeaderBar(ObjectSubclass<imp::HeaderBar>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl HeaderBar {
}

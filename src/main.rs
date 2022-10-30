mod window;
mod header_bar;
mod preferences_window;
mod config_hourly_snapshot;
mod config_max_hourly_snapshots;
mod config_remote_backup;
mod config_remote_user;
mod config_remote_host;
mod config_remote_directory;
mod fridge;

use gio::SimpleAction;
use glib::clone;
use gtk::prelude::*;
use gtk::gio;
use adw::Application;
use window::Window;
use preferences_window::PreferencesWindow;

const APP_ID: &str = "co.veand.Fridge";

fn build_ui(app: &Application) {
    let main_window = Window::new(app);

    // Setup Actions
    let action_preferences = SimpleAction::new("preferences", None);
    action_preferences.connect_activate(clone!(@strong app, @strong main_window => move |_, _| {
        let preferences_window = PreferencesWindow::new(&app);
        preferences_window.set_transient_for(Some(&main_window));
        preferences_window.present();
    }));
    app.add_action(&action_preferences);

    main_window.present();
}

fn main() {
    pretty_env_logger::init();

    // Register and include resources
    gio::resources_register_include!("fridge.gresource")
        .expect("Could not register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

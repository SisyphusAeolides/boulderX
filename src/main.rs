mod app;
mod channels;
mod config;
mod discord;
mod irc;
mod matrix;
mod notify;
mod runtime;
mod theme;
mod ui;

use adw;
use gtk::prelude::ApplicationExt;
use relm4::gtk;
use relm4::RelmApp;

use app::AppModel;

fn main() {
    // Install the async runtime before any Matrix work from the UI thread.
    let _ = runtime::handle();

    gtk::init().expect("Failed to initialize GTK");
    let application = adw::Application::new(Some(notify::APP_ID), Default::default());
    application.connect_startup(|_| {
        theme::load_css();
        notify::setup_application_icon();
    });
    let relm_app = RelmApp::from_app(application);
    relm_app.run::<AppModel>(());
}

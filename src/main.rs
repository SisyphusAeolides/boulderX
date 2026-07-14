mod app;
mod channels;
mod config;
mod irc;
mod matrix;
mod notify;
mod theme;
mod ui;

use app::AppModel;
use relm4::RelmApp;

fn main() {
    gtk::init().expect("Failed to initialize GTK");
    use relm4::gtk;
    let application = adw::Application::new(Some(notify::APP_ID), Default::default());
    application.connect_startup(|_| {
        theme::load_css();
        notify::setup_application_icon();
    });
    use adw;
    let relm_app = RelmApp::from_app(application);
    relm_app.run::<AppModel>(());
}

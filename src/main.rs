use adw::prelude::*;

mod ddc;
mod models;
mod ui;

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("com.ddc.brightness")
        .build();

    app.connect_activate(ui::window::build_window);
    app.run()
}

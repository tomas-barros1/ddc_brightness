use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use adw::prelude::*;

use crate::ddc;
use crate::models::monitor::Monitor;

enum UIMessage {
    MonitorsDetected(Vec<Monitor>, Option<(u8, u8)>),
    BrightnessRead(u8, u8),
    BrightnessSet,
    Error(String),
}

pub fn build_window(app: &adw::Application) {
    let monitors: Rc<RefCell<Vec<Monitor>>> = Rc::new(RefCell::new(Vec::new()));
    let selected_display: Rc<RefCell<Option<u8>>> = Rc::new(RefCell::new(None));
    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let is_setting_value: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let is_initializing: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));

    let (sender, receiver) = mpsc::channel::<UIMessage>();
    let receiver = Rc::new(RefCell::new(receiver));

    // --- Start detection before any UI work (I/O runs in background while we build widgets) ---
    let detect_sender = sender.clone();
    std::thread::spawn(move || {
        let msg = match ddc::detect::detect_monitors() {
            Ok(list) => {
                let brightness = if !list.is_empty() {
                    ddc::brightness::read_brightness(list[0].display_number).ok()
                } else {
                    None
                };
                UIMessage::MonitorsDetected(list, brightness)
            }
            Err(e) => UIMessage::Error(e),
        };
        detect_sender.send(msg).ok();
    });

    // --- Build minimal window skeleton ---
    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("DDC Brightness"));
    window.set_default_size(380, 200);

    let spinner = gtk::Spinner::new();
    spinner.set_halign(gtk::Align::Center);
    spinner.set_valign(gtk::Align::Center);
    spinner.set_size_request(32, 32);
    spinner.start();

    let toast_overlay = adw::ToastOverlay::new();
    let header = adw::HeaderBar::new();

    let string_list = gtk::StringList::new(&[] as &[&str]);
    let combo_row = adw::ComboRow::builder()
        .title("Monitor")
        .model(&string_list)
        .build();

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    scale.set_sensitive(false);

    let percentage_label = gtk::Label::new(Some("--%"));
    percentage_label.set_halign(gtk::Align::Center);
    percentage_label.add_css_class("heading");

    let brightness_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    brightness_box.set_margin_top(12);
    brightness_box.set_margin_bottom(12);
    brightness_box.set_margin_start(12);
    brightness_box.set_margin_end(12);
    brightness_box.append(&scale);
    brightness_box.append(&percentage_label);

    let brightness_row = adw::PreferencesRow::new();
    brightness_row.set_child(Some(&brightness_box));

    let brightness_group = adw::PreferencesGroup::new();
    brightness_group.set_title("Brightness");
    brightness_group.add(&brightness_row);

    let monitor_group = adw::PreferencesGroup::new();
    monitor_group.add(&combo_row);

    let page = adw::PreferencesPage::new();
    page.add(&monitor_group);
    page.add(&brightness_group);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(500);
    clamp.set_tightening_threshold(400);
    clamp.set_child(Some(&page));

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content_box.append(&header);
    content_box.append(&clamp);

    toast_overlay.set_child(Some(&content_box));

    let main_overlay = gtk::Overlay::new();
    main_overlay.set_child(Some(&toast_overlay));
    main_overlay.add_overlay(&spinner);

    window.set_content(Some(&main_overlay));

    // --- Main thread message handler ---
    let rx_msg = receiver.clone();
    let m_monitors = monitors.clone();
    let m_selected = selected_display.clone();
    let m_is_setting = is_setting_value.clone();
    let m_is_init = is_initializing.clone();
    let m_string_list = string_list.clone();
    let m_combo = combo_row.clone();
    let m_scale = scale.clone();
    let m_percentage = percentage_label.clone();
    let m_spinner = spinner.clone();
    let m_toast = toast_overlay.clone();
    let m_sender = sender.clone();

    glib::idle_add_local(move || {
        let rx = rx_msg.borrow_mut();
        loop {
            let msg = match rx.try_recv() {
                Ok(msg) => msg,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return glib::ControlFlow::Break,
            };

            match msg {
                UIMessage::MonitorsDetected(list, brightness) => {
                    *m_monitors.borrow_mut() = list;
                    let mon_list = m_monitors.borrow();

                    for m in mon_list.iter() {
                        m_string_list.append(&m.name);
                    }

                    if !mon_list.is_empty() {
                        *m_is_init.borrow_mut() = true;
                        m_combo.set_selected(0);
                        *m_is_init.borrow_mut() = false;

                        let display = mon_list[0].display_number;
                        *m_selected.borrow_mut() = Some(display);

                        if let Some((cur, max)) = brightness {
                            m_scale.set_range(0.0, max as f64);
                            m_scale.set_value(cur as f64);
                            m_percentage.set_text(&format!("{}%", cur));
                            m_scale.set_sensitive(true);
                            m_spinner.stop();
                        } else {
                            m_spinner.start();
                            m_scale.set_sensitive(false);
                            let s = m_sender.clone();
                            std::thread::spawn(move || {
                                match ddc::brightness::read_brightness(display) {
                                    Ok((cur, max)) => {
                                        s.send(UIMessage::BrightnessRead(cur, max)).ok()
                                    }
                                    Err(e) => s.send(UIMessage::Error(e)).ok(),
                                };
                            });
                        }
                    }
                }
                UIMessage::BrightnessRead(current, max) => {
                    *m_is_setting.borrow_mut() = true;
                    m_scale.set_range(0.0, max as f64);
                    m_scale.set_value(current as f64);
                    *m_is_setting.borrow_mut() = false;
                    m_percentage.set_text(&format!("{}%", current));
                    m_scale.set_sensitive(true);
                    m_spinner.stop();
                }
                UIMessage::Error(err) => {
                    m_scale.set_sensitive(true);
                    m_spinner.stop();
                    let toast = adw::Toast::new(&err);
                    m_toast.add_toast(toast);
                }
                UIMessage::BrightnessSet => {}
            }
        }
        glib::ControlFlow::Continue
    });

    // --- Monitor selection ---
    let sel_is_init = is_initializing.clone();
    let sel_monitors = monitors.clone();
    let sel_selected = selected_display.clone();
    let sel_spinner = spinner.clone();
    let sel_scale = scale.clone();
    let sel_sender = sender.clone();

    combo_row.connect_selected_notify(move |row| {
        if *sel_is_init.borrow() {
            return;
        }

        let idx = row.selected() as usize;
        let mon_list = sel_monitors.borrow();
        if idx >= mon_list.len() {
            return;
        }

        let display = mon_list[idx].display_number;
        *sel_selected.borrow_mut() = Some(display);

        sel_spinner.start();
        sel_scale.set_sensitive(false);
        let s = sel_sender.clone();
        std::thread::spawn(move || {
            match ddc::brightness::read_brightness(display) {
                Ok((cur, max)) => s.send(UIMessage::BrightnessRead(cur, max)).ok(),
                Err(e) => s.send(UIMessage::Error(e)).ok(),
            };
        });
    });

    // --- Slider with debounce ---
    let sld_percentage = percentage_label.clone();
    let sld_is_setting = is_setting_value.clone();
    let sld_selected = selected_display.clone();
    let sld_debounce = debounce_source.clone();
    let sld_sender = sender.clone();

    scale.connect_value_changed(move |s| {
        let val = s.value() as u8;
        sld_percentage.set_text(&format!("{}%", val));

        if *sld_is_setting.borrow() {
            return;
        }

        if let Some(id) = sld_debounce.borrow_mut().take() {
            id.remove();
        }

        let display = match *sld_selected.borrow() {
            Some(d) => d,
            None => return,
        };

        let ds = sld_debounce.clone();
        let ss = sld_sender.clone();

        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(200),
            move || {
                *ds.borrow_mut() = None;
                std::thread::spawn(move || {
                    match ddc::brightness::set_brightness(display, val) {
                        Ok(_) => ss.send(UIMessage::BrightnessSet).ok(),
                        Err(e) => ss.send(UIMessage::Error(e)).ok(),
                    };
                });
            },
        );
        *sld_debounce.borrow_mut() = Some(id);
    });

    window.present();
}

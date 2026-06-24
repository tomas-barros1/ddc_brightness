use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

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
    let is_syncing: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    let (sender, receiver) = mpsc::channel::<UIMessage>();
    let receiver = Rc::new(RefCell::new(receiver));

    // --- Serialize ddcutil setvcp calls to prevent I2C bus races ---
    let setting_flag = Arc::new(AtomicBool::new(false));

    // --- Start detection before any UI work ---
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

    // --- Build window ---
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
    scale.set_hexpand(true);
    scale.set_sensitive(false);

    let spin_button = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
    spin_button.set_halign(gtk::Align::End);
    spin_button.set_sensitive(false);
    spin_button.set_width_chars(3);

    let percent_label = gtk::Label::new(Some("%"));
    percent_label.set_halign(gtk::Align::Start);

    let value_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    value_box.set_halign(gtk::Align::Center);
    value_box.append(&spin_button);
    value_box.append(&percent_label);

    let brightness_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    brightness_box.set_margin_top(12);
    brightness_box.set_margin_bottom(12);
    brightness_box.set_margin_start(12);
    brightness_box.set_margin_end(12);
    brightness_box.append(&scale);
    brightness_box.append(&value_box);

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
    let m_spin = spin_button.clone();
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
                            m_spin.set_range(0.0, max as f64);
                            m_spin.set_value(cur as f64);
                            m_scale.set_sensitive(true);
                            m_spin.set_sensitive(true);
                            m_spinner.stop();
                        } else {
                            m_spinner.start();
                            m_scale.set_sensitive(false);
                            m_spin.set_sensitive(false);
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
                    m_spin.set_range(0.0, max as f64);
                    m_spin.set_value(current as f64);
                    *m_is_setting.borrow_mut() = false;
                    m_scale.set_sensitive(true);
                    m_spin.set_sensitive(true);
                    m_spinner.stop();
                }
                UIMessage::Error(err) => {
                    m_scale.set_sensitive(true);
                    m_spin.set_sensitive(true);
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
    let sel_spin = spin_button.clone();
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
        sel_spin.set_sensitive(false);
        let s = sel_sender.clone();
        std::thread::spawn(move || {
            match ddc::brightness::read_brightness(display) {
                Ok((cur, max)) => s.send(UIMessage::BrightnessRead(cur, max)).ok(),
                Err(e) => s.send(UIMessage::Error(e)).ok(),
            };
        });
    });

    // --- Slider changes → update spin button + debounce ---
    let sld_syncing = is_syncing.clone();
    let sld_spin = spin_button.clone();
    let sld_is_setting = is_setting_value.clone();
    let sld_selected = selected_display.clone();
    let sld_debounce = debounce_source.clone();
    let sld_sender = sender.clone();
    let sld_flag = setting_flag.clone();

    scale.connect_value_changed(move |s| {
        let val = s.value() as u8;

        *sld_syncing.borrow_mut() = true;
        sld_spin.set_value(val as f64);
        *sld_syncing.borrow_mut() = false;

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
        let sf = sld_flag.clone();

        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(200),
            move || {
                *ds.borrow_mut() = None;
                std::thread::spawn(move || {
                    if sf.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                        return;
                    }
                    match ddc::brightness::set_brightness(display, val) {
                        Ok(_) => {
                            sf.store(false, Ordering::SeqCst);
                            ss.send(UIMessage::BrightnessSet).ok();
                        }
                        Err(e) => {
                            sf.store(false, Ordering::SeqCst);
                            ss.send(UIMessage::Error(e)).ok();
                        }
                    };
                });
            },
        );
        *sld_debounce.borrow_mut() = Some(id);
    });

    // --- Spin button changes → update slider (triggers slider's debounce) ---
    let sp_syncing = is_syncing.clone();
    let sp_is_setting = is_setting_value.clone();
    let sp_scale = scale.clone();
    let sp_selected = selected_display.clone();
    let sp_debounce = debounce_source.clone();
    let sp_sender = sender.clone();
    let sp_flag = setting_flag.clone();

    spin_button.connect_value_changed(move |sb| {
        if *sp_syncing.borrow() {
            return;
        }

        let val = sb.value() as u8;

        *sp_is_setting.borrow_mut() = true;
        sp_scale.set_value(val as f64);
        *sp_is_setting.borrow_mut() = false;

        if let Some(id) = sp_debounce.borrow_mut().take() {
            id.remove();
        }

        let display = match *sp_selected.borrow() {
            Some(d) => d,
            None => return,
        };

        let ds = sp_debounce.clone();
        let ss = sp_sender.clone();
        let sf = sp_flag.clone();

        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(200),
            move || {
                *ds.borrow_mut() = None;
                std::thread::spawn(move || {
                    if sf.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                        return;
                    }
                    match ddc::brightness::set_brightness(display, val) {
                        Ok(_) => {
                            sf.store(false, Ordering::SeqCst);
                            ss.send(UIMessage::BrightnessSet).ok();
                        }
                        Err(e) => {
                            sf.store(false, Ordering::SeqCst);
                            ss.send(UIMessage::Error(e)).ok();
                        }
                    };
                });
            },
        );
        *sp_debounce.borrow_mut() = Some(id);
    });

    // --- Mouse wheel on slider ---
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    let sc_scale = scale.clone();
    scroll.connect_scroll(move |_ctrl, _dx, dy| {
        if dy == 0.0 {
            return glib::Propagation::Proceed;
        }
        let adj = sc_scale.adjustment();
        let step = 5.0;
        let current = adj.value();
        let new_val = (current - dy * step).clamp(adj.lower(), adj.upper());
        if (new_val - current).abs() < 0.5 {
            return glib::Propagation::Stop;
        }
        sc_scale.set_value(new_val);
        glib::Propagation::Stop
    });
    scale.add_controller(scroll);

    // --- Keyboard shortcuts ---
    let key_controller = gtk::EventControllerKey::new();
    let kc_scale = scale.clone();
    let kc_spin = spin_button.clone();
    let kc_display = selected_display.clone();
    let kc_debounce = debounce_source.clone();
    let kc_flag = setting_flag.clone();
    key_controller.connect_key_pressed(move |_ctrl, keyval, _code, _state| {
        let adj = kc_scale.adjustment();
        let current = adj.value();
        let step = 5.0;

        let new_val = match keyval {
            gtk::gdk::Key::Up | gtk::gdk::Key::KP_Up => {
                (current + step).clamp(adj.lower(), adj.upper())
            }
            gtk::gdk::Key::Down | gtk::gdk::Key::KP_Down => {
                (current - step).clamp(adj.lower(), adj.upper())
            }
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                if !kc_spin.has_focus() {
                    return glib::Propagation::Proceed;
                }
                if let Some(id) = kc_debounce.borrow_mut().take() {
                    id.remove();
                }
                let display = match *kc_display.borrow() {
                    Some(d) => d,
                    None => return glib::Propagation::Stop,
                };
                let val = kc_spin.value() as u8;
                let sf = kc_flag.clone();
                std::thread::spawn(move || {
                    if sf.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                        return;
                    }
                    match ddc::brightness::set_brightness(display, val) {
                        Ok(_) => sf.store(false, Ordering::SeqCst),
                        Err(e) => {
                            sf.store(false, Ordering::SeqCst);
                            // Enter handler has no channel sender, ignore errors
                            let _ = e;
                        }
                    };
                });
                return glib::Propagation::Stop;
            }
            _ => return glib::Propagation::Proceed,
        };

        if (new_val - current).abs() < 0.5 {
            return glib::Propagation::Stop;
        }

        kc_scale.set_value(new_val);
        glib::Propagation::Stop
    });
    window.add_controller(key_controller);

    window.present();
}

use adw::gdk::{Key, ModifierType};
use adw::glib;
use adw::prelude::*;
use glib::{Propagation, SourceId};
use gtk::{Align, ApplicationWindow, Box, Entry, EventControllerKey, Label, Orientation};
use log::{info, warn};
use std::cell::RefCell;
use std::os::unix::prelude::CommandExt;
use std::rc::Rc;

mod autocomplete;

const APP_ID: &str = "com.github.oliverbestmann.RsRun";

fn main() -> glib::ExitCode {
    static GLIB_LOGGER: glib::GlibLogger = glib::GlibLogger::new(
        glib::GlibLoggerFormat::Plain,
        glib::GlibLoggerDomain::CrateTarget,
    );

    log::set_logger(&GLIB_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    let app_state = AppState::from(RefCell::new(AppStateInner::new()));

    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(move |app| build_ui(app, app_state.clone()));

    // load list of binaries in the background
    autocomplete::preload();

    // Run the application
    app.run()
}

pub struct AppStateInner {
    timeout_id: Option<SourceId>,
}

impl AppStateInner {
    pub fn new() -> Self {
        Self { timeout_id: None }
    }
}

type AppState = Rc<RefCell<AppStateInner>>;

/// Create the text entry view
fn make_entry_view() -> Entry {
    Entry::builder().placeholder_text("Command").build()
}

fn make_info_label() -> Label {
    Label::builder().label("Run:").halign(Align::Start).build()
}

fn make_container_view() -> Box {
    Box::builder()
        .spacing(8)
        .orientation(Orientation::Vertical)
        .halign(Align::Fill)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build()
}

#[derive(Clone)]
struct Views {
    info: Label,
    entry: Entry,
    container: Box,
}

fn make_views() -> Views {
    let info = make_info_label();
    let entry = make_entry_view();

    let container = make_container_view();
    container.append(&info);
    container.append(&entry);

    Views {
        info,
        entry,
        container,
    }
}

fn build_ui(app: &adw::Application, state: AppState) {
    let views = make_views();

    connect_key_handler(&views, &state);

    // Connect to "clicked" signal of `button`
    views.entry.connect_activate(|entry| {
        let text = entry.text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        execute_command(text);
        std::process::exit(0);
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("rsrun")
        .child(&views.container)
        .default_width(500)
        .resizable(false)
        .build();

    // Present window
    window.present();
}

fn connect_key_handler(views: &Views, state: &AppState) {
    let ev = EventControllerKey::new();

    ev.connect_key_pressed({
        let views = views.clone();
        let state = state.clone();
        move |_, key, _code, modifier| handle_key_press(&views, key, modifier, &state)
    });

    views.entry.add_controller(ev);
}

fn handle_key_press(
    views: &Views,
    key: Key,
    modifier: ModifierType,
    state: &AppState,
) -> Propagation {
    let ctrl = modifier.contains(ModifierType::CONTROL_MASK);

    if key == Key::Escape {
        std::process::exit(0);
    }

    if key == Key::Tab {
        autocomplete(views, &state);
        return Propagation::Stop;
    }

    if ctrl && key == Key::Return {
        let text = views.entry.text();
        if text.is_empty() {
            execute_command("ptyxis --new-window");
        } else {
            execute_command_in_terminal(text.as_str());
        }

        std::process::exit(0);
    }

    if ctrl && key == Key::R {
        println!("search activated");
        return Propagation::Stop;
    }

    Propagation::Proceed
}

fn autocomplete(views: &Views, app_state: &AppState) {
    let text = views.entry.text();

    let cursor = views.entry.position();

    // start end and end the election
    let (cursor, selection_end) = views.entry.selection_bounds().unwrap_or((cursor, cursor));

    let cursor = cursor.min(text.len() as i32);
    let selection_end = selection_end.min(text.len() as i32);

    // get the text before the cursor
    let prefix = &text[..cursor as usize];

    let mut state = app_state.borrow_mut();

    let candidates = autocomplete::candidates(prefix);

    info!("Candidates: {:?}", candidates);

    let suffix = &text[selection_end as usize..];

    match &*candidates {
        [] => views.info.set_text(&format!("No command for {:?}", prefix)),

        [candidate] => {
            // replace the prefix with the candidate
            let new_text = format!("{} {}", candidate, suffix);

            views.entry.set_text(&new_text);
            views.entry.set_position(candidate.len() as i32 + 1);
        }

        [candidate, rest @ ..] => {
            views
                .info
                .set_text(&format!("Found {} candidates", rest.len() + 1));

            // replace the prefix with the candidate
            let new_text = format!("{} {}", candidate, suffix);

            views.entry.set_text(&new_text);
            views.entry.set_position(cursor);

            // select the part that is new
            views
                .entry
                .select_region(cursor, candidate.len() as i32 + 1);
        }
    }

    if let Some(timeout_id) = state.timeout_id.take() {
        timeout_id.remove();
    }

    let timeout_id = glib::timeout_add_seconds_local_once(1, {
        let info = views.info.clone();
        let app_state = app_state.clone();

        move || {
            app_state.borrow_mut().timeout_id = None;
            info.set_label("Run:");
        }
    });

    state.timeout_id = Some(timeout_id);
}

fn execute_command(command: &str) {
    info!("Running command: {:?}", command);
    let err = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .exec();

    warn!("Failed to run: {:?}", err);
}

fn execute_command_in_terminal(command: &str) {
    let command = ["ptyxis", "--", "sh", "-c", command];
    let err = std::process::Command::new("ptyxis").args(command).exec();
    warn!("Failed to run: {:?}", err);
}

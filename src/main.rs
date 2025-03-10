use adw::gdk::{Key, ModifierType};
use adw::glib;
use adw::prelude::*;
use gtk::{Align, ApplicationWindow, Box, Entry, EventControllerKey, Label, Orientation};
use std::os::unix::prelude::CommandExt;

const APP_ID: &str = "com.github.oliverbestmann.RsRun";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &adw::Application) {
    // Create a button with label and margins
    let entry = Entry::builder().placeholder_text("Command").build();

    let ev = EventControllerKey::new();

    ev.connect_key_pressed({
        let entry = entry.clone();

        move |_, key, code, modifier| {
            println!(
                "Pressed key: {:?}, code: {}, modifier: {:?}",
                key, code, modifier
            );

            let ctrl = modifier.contains(ModifierType::CONTROL_MASK);

            if key == Key::Escape {
                std::process::exit(0);
            }

            if ctrl && key == Key::Return {
                let text = entry.text();
                if text.is_empty() {
                    execute_command("ptyxis --new-window");
                } else {
                    execute_command_in_terminal(text.as_str());
                }

                std::process::exit(0);
            }

            if ctrl && key == Key::R {
                println!("search activated");
                return glib::Propagation::Stop;
            }

            glib::Propagation::Proceed
        }
    });

    entry.add_controller(ev);

    // Connect to "clicked" signal of `button`
    entry.connect_activate(|entry| {
        let text = entry.text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        execute_command(text);
        std::process::exit(0);
    });

    let info = Label::builder().label("Run:").halign(Align::Start).build();

    let container = Box::builder()
        .spacing(8)
        .orientation(Orientation::Vertical)
        .halign(Align::Fill)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    container.append(&info);
    container.append(&entry);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Launch")
        .child(&container)
        .width_request(500)
        .resizable(false)
        .build();

    // Present window
    window.present();
}

fn execute_command(command: &str) {
    println!("Running command: {:?}", command);
    let err = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .exec();

    println!("Failed to run: {:?}", err);
}

fn execute_command_in_terminal(command: &str) {
    let command = ["ptyxis", "--", "sh", "-c", command];
    let err = std::process::Command::new("ptyxis").args(command).exec();
    println!("Failed to run: {:?}", err);
}

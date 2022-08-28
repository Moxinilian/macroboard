use std::{io::Write, time::Duration};

use macroboard::{KeyMappingCode, KeyboardTriggers, ListeningCmd, Triggers};

fn main() {
    std::thread::spawn(|| {
        // Security in case devices were grabbed unintentionally
        // to avoid being locked out of the computer
        std::thread::sleep(Duration::from_secs(120));
        std::process::exit(1);
    });

    // Obtain the name of the keyboard
    let mut keyboard_name = String::new();
    print!("Please provide the name of your macro keyboard: ");
    std::io::stdout().flush().expect("failed to flush stdout");
    std::io::stdin()
        .read_line(&mut keyboard_name)
        .expect("failed to read keyboard name");

    // Register the keyboard triggers
    let mut keyboard_triggers = KeyboardTriggers::new(keyboard_name.trim());

    keyboard_triggers.insert_with_release(
        &[KeyMappingCode::ControlLeft, KeyMappingCode::KeyE],
        || println!("Ran combination!"),
        || println!("Released combination!"),
    );

    keyboard_triggers.insert_with_release(
        &[KeyMappingCode::KeyR],
        || println!("Trigger start!"),
        || println!("Trigger stop!"),
    );

    keyboard_triggers.insert(&[KeyMappingCode::KeyE], || println!("Hello!"));

    keyboard_triggers.insert(&[KeyMappingCode::Escape], || ListeningCmd::Stop);

    let mut triggers = Triggers::default();
    triggers.insert(keyboard_triggers);

    // Start the event loop
    triggers.listen();

    println!("Listener gracefully stopped.");
}

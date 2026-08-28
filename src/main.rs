mod lib;
use std::println;

enum Options {
    Print,
    Move(String, u32),
}

fn print_windows(monitor_window_collections: Vec<lib::MonitorWindowCollection>) {
    let mut current_monitor: Option<&lib::MonitorInfo> = None;
    let current_window_handle =
        lib::get_current_monitor_window_collections(&monitor_window_collections);

    for collection in &monitor_window_collections {
        println!(
            "Monitor {}: left={}, top={}, right={}, bottom={}",
            collection.monitor.id,
            collection.monitor.bounds.left,
            collection.monitor.bounds.top,
            collection.monitor.bounds.right,
            collection.monitor.bounds.bottom
        );

        for idx in 0..collection.windows.len() {
            let window = &collection.windows[idx];
            if window.is_visible() {
                println!(
                    "\t{}-{}: {}",
                    collection.monitor.id, idx, collection.windows[idx].title
                );
            }

            if collection.windows[idx].handle == current_window_handle {
                current_monitor = Some(&collection.monitor);
            }
        }
        println!("")
    }

    println!("");

    if let Some(monitor) = current_monitor {
        println!("Current window is on monitor {}", monitor.id);
    } else {
        println!("Current window is not on any known monitor");
    }
}

fn move_window(
    monitor_window_collections: Vec<lib::MonitorWindowCollection>,
    window_title: &str,
    monitor_id: u32,
) {
    let monitor = match monitor_window_collections
        .iter()
        .find(|collection| collection.monitor.id == monitor_id)
    {
        Some(collection) => collection.monitor,
        None => {
            println!("Monitor with id {} not found", monitor_id);
            return;
        }
    };

    let window_title = window_title.to_lowercase();

    for collection in monitor_window_collections {
        for idx in 0..collection.windows.len() {
            let window = &collection.windows[idx];
            let window_id = format!("{}-{}", collection.monitor.id, idx);
            if window.title.to_lowercase().contains(&window_title)
                || window_id.to_lowercase() == window_title
            {
                println!(
                    "Moving window '{}' to monitor {}",
                    window.to_string(),
                    monitor_id
                );
                lib::move_window_to_monitor(&window, &monitor);
                return;
            }
        }
    }
    println!(
        "Window '{}' not found on monitor {}",
        window_title, monitor_id
    );
}

fn print_help() {
    println!("Usage:");
    println!("  > win-move.exe");
    println!("      prints all windows and their associated monitors");
    println!("  > win-move.exe help");
    println!("      prints this help message");
    println!("  > win-move.exe <window_title> <monitor_id>");
    println!("      supports partial window title matching");
    println!("      also supports window id matching in the format <monitor_id>-<window_index>");
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let option = if args.len() == 1 {
        Options::Print
    } else if args.len() == 3 {
        let window_title = args[1].clone();
        let monitor_id = match args[2].parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                eprintln!("Invalid monitor id: {}", args[2]);
                std::process::exit(1);
            }
        };
        Options::Move(window_title, monitor_id)
    } else {
        print_help();
        std::process::exit(1);
    };

    let mut windows = lib::enumerate_windows(false);
    let monitors = lib::enumerate_monitors();
    let monitor_window_collections = lib::map_windows_to_monitors(&mut windows, &monitors);

    match option {
        Options::Print => print_windows(monitor_window_collections),
        Options::Move(window_title, monitor_id) => {
            move_window(monitor_window_collections, &window_title, monitor_id)
        }
    }
}

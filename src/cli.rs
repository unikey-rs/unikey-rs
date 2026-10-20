//! Unified command-line surface.

use crate::config::Config;
use crate::layout::Layout;
use crate::wayland;
use clap::{Parser, Subcommand};
use std::io::{self, IsTerminal, Read, Write};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(
    name = "unikey-rs",
    version,
    about = "UniKey RS 0.x — multi-layout Wayland-native text composer",
    long_about = "Compose Vietnamese text with Telex, VNI, or VIQR.\n\nThe beta alias is intentionally `UniKey RS`; all releases remain on the 0.x line."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List built-in keyboard layouts.
    Layouts,
    /// Compose raw keystrokes into display text.
    Compose {
        /// Layout: en, telex, vni, or viqr.
        #[arg(short, long, value_parser = parse_layout)]
        layout: Option<Layout>,
        /// Raw keystrokes. Read stdin when omitted.
        text: Option<String>,
        /// Disable repeated-modifier restore behaviour (for example, `ass` -> `ás`).
        #[arg(long)]
        no_restore_keys: bool,
    },
    /// Open the native Wayland GUI control panel.
    Gui {
        /// Initial layout.
        #[arg(short, long, value_parser = parse_layout)]
        layout: Option<Layout>,
    },
    /// Run the native Wayland input-method service.
    InputMethod {
        /// Layout used while the service is active.
        #[arg(short, long, value_parser = parse_layout)]
        layout: Option<Layout>,
    },
    /// Inspect the Wayland session and distro-specific capabilities.
    Doctor,
    /// Print the effective configuration.
    ConfigShow,
    /// Save a configuration value.
    ConfigSet {
        key: String,
        value: String,
    },
}

pub fn run() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Layouts => {
            for layout in Layout::ALL {
                println!("{:<7} {:<21} {}", layout.id(), layout.display_name(), layout.description());
            }
            Ok(())
        }
        Command::Compose { layout, text, no_restore_keys } => {
            let config = Config::load()?;
            let layout = layout.unwrap_or(config.layout);
            let from_argument = text.is_some();
            let raw = match text {
                Some(text) => text,
                None if !io::stdin().is_terminal() => {
                    let mut text = String::new();
                    io::stdin().read_to_string(&mut text)?;
                    text
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provide TEXT or pipe raw keystrokes on stdin",
                    ))
                }
            };
            let options = crate::layout::ComposerOptions {
                restore_keys: config.restore_keys && !no_restore_keys,
            };
            let composed = crate::layout::compose_with_options(layout, &raw, options);
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(composed.as_bytes())?;
            if from_argument {
                stdout.write_all(b"\n")?;
            }
            Ok(())
        }
        Command::Gui { layout } => {
            let config = Config::load()?;
            crate::gui::run(layout.unwrap_or(config.layout))
        }
        Command::InputMethod { layout } => {
            let config = Config::load()?;
            crate::wayland::input_method::InputMethodService::new(layout.unwrap_or(config.layout))
                .run()
        }
        Command::Doctor => doctor(),
        Command::ConfigShow => {
            print!("{}", Config::load()?.render());
            Ok(())
        }
        Command::ConfigSet { key, value } => set_config(&key, &value),
    }
}

fn parse_layout(value: &str) -> Result<Layout, String> {
    Layout::from_str(value).map_err(|error| error.to_string())
}

fn set_config(key: &str, value: &str) -> io::Result<()> {
    let mut config = Config::load()?;
    match key {
        "layout" => config.layout = Layout::from_str(value).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?,
        "restore_keys" => config.restore_keys = parse_bool(value)?,
        "daemon" => config.daemon = parse_bool(value)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown config key; expected layout, restore_keys, or daemon",
            ))
        }
    }
    config.save()?;
    println!("saved {}", Config::config_path().display());
    Ok(())
}

fn parse_bool(value: &str) -> io::Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "expected true/false")),
    }
}

fn doctor() -> io::Result<()> {
    println!("UniKey RS          {} (0.x beta)", env!("CARGO_PKG_VERSION"));
    println!("OS                 {}", read_os_name().unwrap_or_else(|| "unknown Linux".to_owned()));
    println!("Session type       {}", env_or("XDG_SESSION_TYPE", "unknown"));
    println!("Desktop            {}", env_or("XDG_CURRENT_DESKTOP", "unknown"));
    println!("Wayland display    {}", env_or("WAYLAND_DISPLAY", "wayland-0"));
    println!("XDG_RUNTIME_DIR    {}", env_or("XDG_RUNTIME_DIR", "<missing>"));
    println!("Platform note      immutable distros (Fedora Atomic, Silverblue, COSMIC Atomic) supported via AppImage");

    match wayland::socket_path() {
        Ok(path) => println!("Wayland socket     {} ({})", path.display(), if path.exists() { "found" } else { "missing" }),
        Err(error) => println!("Wayland socket     error: {error}"),
    }

    match wayland::probe_globals() {
        Ok(globals) => {
            println!("Wayland globals    {}", globals.len());
            for global in &globals {
                println!("  {:<32} v{:<2} id={}", global.interface, global.version, global.name);
            }
            let im_v2 = globals.iter().any(|global| global.interface == "zwp_input_method_manager_v2");
            let im_v1 = globals.iter().any(|global| global.interface == "zwp_input_method_v1");
            println!("Input method v2    {}", if im_v2 { "available" } else { "not advertised" });
            println!("Input method v1    {}", if im_v1 { "available" } else { "not advertised" });
        }
        Err(error) => println!("Wayland globals    error: {error}"),
    }
    Ok(())
}

fn read_os_name() -> Option<String> {
    let text = std::fs::read_to_string("/etc/os-release").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_owned())
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_owned())
}

use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger};

pub fn setup_logger() {
    match CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Error, Config::default(), TerminalMode::Stdout, ColorChoice::Auto),
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Stdout, ColorChoice::Auto),
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Stdout, ColorChoice::Auto),
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Stdout, ColorChoice::Auto),
        ]
    ) {
        Ok(()) => println!("Logger initialized"),
        Err(err) => println!("Failed to init logger: {}", err)
    }

}

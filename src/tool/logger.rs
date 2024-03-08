use std::{env::var, path::Path};

use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            policy::{
                self, compound::roll::fixed_window::FixedWindowRoller,
                compound::trigger::size::SizeTrigger,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Logger, Root},
};

pub use log::{debug, error};

const LOGGER_SIZE: u64 = 10 * 1024 * 1024;
const LOGGER_FILE_COUNT: u32 = 10;

pub fn init_logger(log_path: &Path) -> Result<(), SetLoggerError> {
    let log_level = get_log_level();
    let rolling_file_path = log_path.join("sys-kaleido.{}.log");
    let rolling_file_str = rolling_file_path
        .as_os_str()
        .to_str()
        .expect("rolling file name error");

    let app_debug = match var("KALEIDO_DEBUG") {
        Ok(v) => "true" == v,
        Err(_) => false,
    };
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
    let trigger = SizeTrigger::new(LOGGER_SIZE);
    let roller = FixedWindowRoller::builder()
        .build(rolling_file_str, LOGGER_FILE_COUNT)
        .unwrap();
    let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let rolling_file = RollingFileAppender::builder()
        // .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}\n")))
        .build(rolling_file_path, Box::new(policy))
        .unwrap();

    let config = if is_release() && !app_debug {
        Config::builder()
            .appender(Appender::builder().build("roller", Box::new(rolling_file)))
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .logger(Logger::builder().build("sys_kaleido", log_level))
            .logger(Logger::builder().build("html5ever", LevelFilter::Warn))
            .build(
                Root::builder()
                    .appender("stderr")
                    .appender("roller")
                    .build(LevelFilter::Warn),
            )
            .unwrap()
    } else {
        Config::builder()
            .appender(Appender::builder().build("roller", Box::new(rolling_file)))
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .logger(Logger::builder().build("sys_kaleido", log_level))
            .logger(Logger::builder().build("html5ever", LevelFilter::Info))
            .build(
                Root::builder()
                    .appender("roller")
                    .appender("stderr")
                    .build(LevelFilter::Debug),
            )
            .unwrap()
    };

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config)?;

    Ok(())
}

#[cfg(build = "release")]
fn is_release() -> bool {
    true
}

#[cfg(not(build = "release"))]
fn is_release() -> bool {
    false
}

#[cfg(build = "release")]
fn get_log_level() -> LevelFilter {
    LevelFilter::Info
}

#[cfg(not(build = "release"))]
fn get_log_level() -> LevelFilter {
    LevelFilter::Debug
}

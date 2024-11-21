use log::LevelFilter;
use simplelog::{format_description, CombinedLogger, ConfigBuilder, LevelPadding, WriteLogger};
use std::fs::{remove_file, OpenOptions};

use crate::constants::{LOG_FILE_PATH, LOG_LEVEL};

pub(crate) fn init() {
    let _ = remove_file(LOG_FILE_PATH);
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LOG_FILE_PATH)
        .expect("Error: failed to open log file");

    // Configure the logger to show only the timestamp
    let config = ConfigBuilder::new()
        .set_level_padding(LevelPadding::Off)
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_max_level(LevelFilter::Off)
        .set_time_format_custom(format_description!(
            "[hour repr:24]:[minute]:[second]:[subsecond digits:6]"
        ))
        .build();

    CombinedLogger::init(vec![WriteLogger::new(
        LOG_LEVEL, config, // Use the custom configuration
        log_file,
    )])
    .expect("Error: failed to initialize logger");
}

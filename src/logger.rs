use dirs::home_dir;
use flexi_logger::{DeferredNow, FileSpec, LogSpecification, Logger as logger, Record};
use std::{
    io::{self, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Logger {
    pub log_file: PathBuf,
    pub log_to_stdout: bool,
    pub log_to_stderr: bool,
}

fn log_format(
    write: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), io::Error> {
    let time = now.format("%Y-%m-%d %H:%M");
    write!(write, "[{time} {}] {}", record.level(), record.args())
}
impl Logger {
    pub fn logger_parse(self) {
        let spec = LogSpecification::trace();
        let file;
        if self
            .log_file
            .to_str()
            .expect("log file change failed")
            .is_empty()
        {
            file = FileSpec::try_from(home_dir().unwrap().join(".at.log")).unwrap();
        } else {
            file = FileSpec::try_from(self.log_file).unwrap();
        }
        println!("log info:{:?}", file);
        if (self.log_to_stdout) && (self.log_to_stderr) {
            logger::with(spec)
                .log_to_file(file)
                .log_to_stdout()
                .log_to_stderr()
                .format(log_format)
                .start()
                .unwrap();
        } else {
            logger::with(spec)
                .log_to_file(file)
                .format(log_format)
                .start()
                .unwrap();
        }
    }
}



/*

// There is no sense to implement own log::Log for compile time
// because (in rust) it should be MANUALLY installed/set up (for example, in java logger uses auto set up):
//  * what is impossible for macros
//  * dangerous/ambiguous in case if every macros sets up its own log implementation.
//
// For that reasons I've decided to use my own compile_log_warn macros

use std::sync::Once;
use log::{ Record, Level, Metadata, LevelFilter };


#[allow(dead_code)]
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => eprintln!("Compile ERROR : {} - {}", record.level(), record.args()),
                Level::Warn  => eprintln!("Compile WARN  : {} - {}", record.level(), record.args()),
                Level::Info  => println! ("Compile INFO  : {} - {}", record.level(), record.args()),
                Level::Debug => println! ("Compile DEBUG : {} - {}", record.level(), record.args()),
                Level::Trace => println! ("Compile TRACE : {} - {}", record.level(), record.args()),
            }
        }
    }

    fn flush(&self) { }
}


static INIT: Once = Once::new();
static LOGGER: SimpleLogger = SimpleLogger;

#[allow(dead_code)]
pub fn init_compile_logger() {
    INIT.call_once(||
        log::set_logger(&LOGGER)
            .map(|_| log::set_max_level(LevelFilter::Info))
            .unwrap()
    )
}

*/

use std::time::Instant;

use log::info;

//THINK Portability - string type, drop println method?
/**
 * On creation, stores time.  On drop, `info!` `Timer [NAME]: ELAPSED_TIME`.
 */
pub struct Autotimer {
    name: String,
    start: Instant,
}

impl Autotimer {
    pub fn new(name: impl Into<String>) -> Autotimer {
        return Autotimer { name: name.into(), start: Instant::now() };
    }
}

impl Drop for Autotimer {
    fn drop(&mut self) {
        let d = self.start.elapsed();
        info!("Timer [{}]: {:?}", self.name, d);
    }
}
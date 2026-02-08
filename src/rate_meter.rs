
//MISC Ugh, I don't like all the std/nostd back and forth

#[cfg(not(feature = "std"))]
use core::option::{Option, Option::None, Option::Some};

#[cfg(feature = "std")]
use std::time::{Instant, Duration};

// From esp_hal
#[cfg(not(feature = "std"))]
type Instant = fugit::Instant<u64, 1, 1_000_000>;
#[cfg(not(feature = "std"))]
type Duration = fugit::Duration<u64, 1, 1_000_000>;

pub struct RateMeter {
  pub count: u64,
  pub last_time: Option<Instant>,
  pub interval: Duration,
}

impl RateMeter {
  pub const fn default() -> RateMeter {
    #[cfg(not(feature = "std"))]
    let interval = Duration::millis(1000);
    
    #[cfg(feature = "std")]
    let interval = Duration::from_millis(1000);

    return RateMeter {
      count: 0,
      last_time: None,
      interval,
    };
  }

  pub fn new(
    #[cfg(not(feature = "std"))]
    now: Instant,
  ) -> RateMeter {
    #[cfg(feature = "std")]
    let now = Instant::now();

    #[cfg(not(feature = "std"))]
    let interval = Duration::millis(1000);
    
    #[cfg(feature = "std")]
    let interval = Duration::from_millis(1000);

    return RateMeter {
      count: 0,
      last_time: Some(now),
      interval,
    };
  }

  /**
   * Adds n to the count.
   */
  pub fn add(&mut self, n: u64) {
    self.count += n;
  }

  /**
   * Adds 1 to the count.
   */
  pub fn inc(&mut self) {
    self.count += 1;
  }

  /**
   * Adds 1 to the count.  Checks time interval.  If elapsed, reset and return rate (count / second) since last reset.
   */
  pub fn auto(
    &mut self,
    #[cfg(not(feature = "std"))]
    now: Instant,
  ) -> Option<f64> {
    self.inc();
    return self.check(
      #[cfg(not(feature = "std"))]
      now,
    );
  }

  /**
   * Checks time interval.  If elapsed, reset and return rate (count / second) since last reset.
   */
  pub fn check(
    &mut self,
    #[cfg(not(feature = "std"))]
    now: Instant,
  ) -> Option<f64> {
    #[cfg(feature = "std")]
    let now = Instant::now();
    
    let last_time = match self.last_time {
        Some(lt) => lt,
        None => {
            self.last_time = Some(now);
            now
        },
    };

    if now >= last_time + self.interval {
      #[cfg(feature = "std")]
      let r = Some((self.count as f64) / (now - last_time).as_secs_f64());

      #[cfg(not(feature = "std"))]
      let r = {
        //LEAK It would be nice to support nanos without losing precision for large times
        let us = (now - last_time).to_micros();
        let s = (us as f64) / 1_000_000_f64;
        let r = Some((self.count as f64) / s);
        r
      };

      self.last_time = Some(now);
      self.count = 0;
      return r;
    } else {
      return None;
    }
  }

  /**
   * Reset and return rate (count / second) since last reset.
   */
  pub fn measure(
    &mut self,
    #[cfg(not(feature = "std"))]
    now: Instant,
  ) -> f64 {
    #[cfg(feature = "std")]
    let now = Instant::now();

    let last_time = match self.last_time {
        Some(lt) => lt,
        None => {
            self.last_time = Some(now);
            now
        },
    };

    #[cfg(feature = "std")]
    let r = (self.count as f64) / (now - last_time).as_secs_f64();

    #[cfg(not(feature = "std"))]
    let r = {
      //LEAK It would be nice to support nanos without losing precision for large times
      let us = (now - last_time).to_micros();
      let s = (us as f64) / 1_000_000_f64;
      let r = (self.count as f64) / s;
      r
    };

    self.last_time = Some(now);
    self.count = 0;
    return r;
  }
}
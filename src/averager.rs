use core::ops::AddAssign;

use num::traits::ConstZero;

/**
 * Takes N numbers one at a time and then averages them.
 * Essentially handles the bookkeeping of how many numbers you've
 * added so far, and every N numbers outputs the average.
 * (Or sum, if you're using auto_sum instead of auto.)
 */
pub struct Averager<T, COUNT = usize> where
  T: num::Num + AddAssign,
  COUNT: num::Integer, {
  pub current_count: COUNT,
  pub target_count: COUNT,
  pub sum: T,
}

impl<T, COUNT> Averager<T, COUNT> where
  T: num::Num + AddAssign + Copy + ConstZero,
  COUNT: num::Integer + ConstZero + AddAssign, {
  pub const fn new(target_count: COUNT) -> Averager<T, COUNT> {
    return Averager {
      current_count: COUNT::ZERO,
      target_count,
      sum: T::ZERO,
    };
  }

  pub fn reset(&mut self) {
    self.sum = T::ZERO;
    self.current_count = COUNT::ZERO;
  }

  /**
   * Adds x to the sum.  Increments current_count.
   */
  pub fn add(&mut self, x: T) {
    self.sum += x;
    self.current_count += COUNT::one();
  }

  /**
   * Adds x to the sum.  Checks count.  If >= target, reset and return average (sum / count) since last reset.
   */
  pub fn auto(
    &mut self,
    x: T,
  ) -> Option<f64> where
    f64: From<T> + From<COUNT>,
    COUNT: Copy, {
    self.add(x);
    return self.check();
  }

  /**
   * Adds x to the sum.  Checks count.  If >= target, reset and return sum since last reset.
   */
  pub fn auto_sum(
    &mut self,
    x: T,
  ) -> Option<T> {
    self.add(x);
    let sum = self.sum;
    if self.current_count >= self.target_count {
      self.reset();
      return Some(sum);
    } else {
      return None;
    }
  }

  /**
   * Checks count.  If >= target, reset and return average (sum / count) since last reset.
   */
  pub fn check(
    &mut self,
  ) -> Option<f64> where
    f64: From<T> + From<COUNT>,
    COUNT: Copy, {
    if self.current_count >= self.target_count {
      let r = self.measure();
      self.reset();
      return Some(r);
    }
    return None;
  }

  /**
   * Reset and return average (sum / count) since last reset.
   */
  pub fn measure(
    &mut self,
  ) -> f64 where
    f64: From<T> + From<COUNT>,
    COUNT: Copy, {
    let sum: f64 = self.sum.into();
    let cc: f64 = self.current_count.into();
    let r: f64 = sum / cc;
    self.reset();
    return r;
  }




  /**
   * Adds x to the sum.  Checks count.  If >= target, reset and return average (sum / count) since last reset.
   * Does not convert to f64, does math as T.
   */
  pub fn auto_t(
    &mut self,
    x: T,
  ) -> Option<T>  where
    T: From<COUNT>,
    COUNT: Copy, {
    self.add(x);
    return self.check_t();
  }

  /**
   * Checks count.  If >= target, reset and return average (sum / count) since last reset.
   * Does not convert to f64, does math as T.
   */
  pub fn check_t(
    &mut self,
  ) -> Option<T> where
    T: From<COUNT>,
    COUNT: Copy, {
    if self.current_count >= self.target_count {
      let r = self.measure_t();
      self.reset();
      return Some(r);
    }
    return None;
  }

  /**
   * Reset and return average (sum / count) since last reset.
   * Does not convert to f64, does math as T.
   */
  pub fn measure_t(
    &mut self,
  ) -> T where
    T: From<COUNT>,
    COUNT: Copy, {
    let cc: T = self.current_count.into();
    let r: T = self.sum / cc;
    self.reset();
    return r;
  }
}
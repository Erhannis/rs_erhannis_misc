use std::collections::VecDeque;

// Claude Opus 4.6

pub struct RollingStats {
    buffer: VecDeque<f32>,
    capacity: usize,
    sum: f64,
    sum_of_squares: f64,
}

impl RollingStats {
    pub fn new(count: usize) -> Self {
        assert!(count > 0, "COUNT must be greater than zero");
        Self {
            buffer: VecDeque::with_capacity(count),
            capacity: count,
            sum: 0.0,
            sum_of_squares: 0.0,
        }
    }

    /// Push a new sample, evicting the oldest if the buffer is full.
    pub fn push(&mut self, value: f32) {
        let v = value as f64;
        self.sum += v;
        self.sum_of_squares += v * v;

        if self.buffer.len() == self.capacity {
            let old = self.buffer.pop_front().unwrap() as f64;
            self.sum -= old;
            self.sum_of_squares -= old * old;
        }

        self.buffer.push_back(value);
    }

    /// Number of samples currently in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Whether the buffer is full.
    pub fn is_full(&self) -> bool {
        self.buffer.len() == self.capacity
    }

    /// Arithmetic mean of the buffered samples.
    pub fn mean(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        (self.sum / self.len() as f64) as f32
    }

    /// Population variance (σ²) over the buffered samples.
    pub fn variance(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        let n = self.len() as f64;
        let mean = self.sum / n;
        // Var = E[x²] - (E[x])²
        ((self.sum_of_squares / n) - (mean * mean)).max(0.0) as f32
    }

    /// Sample variance (using Bessel's correction, n-1).
    pub fn sample_variance(&self) -> f32 {
        if self.len() < 2 {
            return 0.0;
        }
        let n = self.len() as f64;
        let mean = self.sum / n;
        let var = (self.sum_of_squares - n * mean * mean).max(0.0);
        (var / (n - 1.0)) as f32
    }

    /// Population standard deviation (σ).
    pub fn std_dev(&self) -> f32 {
        self.variance().sqrt()
    }

    /// Sample standard deviation (s, using Bessel's correction).
    pub fn sample_std_dev(&self) -> f32 {
        self.sample_variance().sqrt()
    }

    /// Root mean square.
    pub fn rms(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        ((self.sum_of_squares / self.len() as f64).sqrt()) as f32
    }

    /// Min value in the buffer. Returns `None` if empty.
    pub fn min(&self) -> Option<f32> {
        self.buffer.iter().copied().reduce(f32::min)
    }

    /// Max value in the buffer. Returns `None` if empty.
    pub fn max(&self) -> Option<f32> {
        self.buffer.iter().copied().reduce(f32::max)
    }

    /// Clear all samples and reset accumulators.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.sum = 0.0;
        self.sum_of_squares = 0.0;
    }
}
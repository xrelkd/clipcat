use std::{collections::BTreeMap, time::Duration};

#[derive(Clone, Debug)]
pub struct RetryInterval {
    count: usize,
    limit: usize,
    phases: BTreeMap<usize, Duration>,
}

impl RetryInterval {
    #[must_use]
    pub fn new(max_try_count: usize, fallback_interval: Duration) -> Self {
        Self {
            count: 0,
            limit: max_try_count,
            phases: BTreeMap::from([(max_try_count, fallback_interval)]),
        }
    }

    #[must_use]
    pub fn add_phase(mut self, upper_bound: usize, interval: Duration) -> Self {
        let _unused = self.phases.insert(upper_bound, interval);
        self
    }

    pub fn reset(&mut self) { self.count = 0; }

    #[must_use]
    pub const fn limit(&self) -> usize { self.limit }
}

impl Default for RetryInterval {
    fn default() -> Self {
        Self::new(10000, Duration::from_millis(5000))
            .add_phase(10, Duration::from_millis(100))
            .add_phase(50, Duration::from_millis(500))
            .add_phase(100, Duration::from_millis(2500))
    }
}

impl Iterator for RetryInterval {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count <= self.limit {
            self.phases.range(self.count..).next().map(|(_, interval)| interval).copied()
        } else {
            None
        }
    }
}

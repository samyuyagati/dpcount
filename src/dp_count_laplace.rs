use std::f64;
use randomkit::dist::Laplace;
use randomkit::{Rng, Sample};
// Differentially-private count using the Laplace mechanism to add noise.
pub struct DPCounterLP {
    epsilon: f64,
    count: f64,
    noise: f64,
    count_updated: bool,
    noise_distr: Laplace,
    rng: Rng,
}

impl DPCounterLP {
    pub fn new(e: f64) -> DPCounterLP {
        DPCounterLP {
            epsilon: e,
            count: 0.0,
            noise: 0.0,
            count_updated: true,
            noise_distr: Laplace::new(0.0, 1.0/e).unwrap(),
            rng: Rng::from_seed(1),
        }
    }

    pub fn process_record(&mut self, r: bool) {
        if r {
            self.count += 1.0;
        }
        // TODO: should count_updated be set to true every time process_record
        // is called, or only when the count is actually modified?
        self.count_updated = true;
    }

    pub fn get_epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn get_true_count(&self) -> f64 {
        self.count
    }

    pub fn get_count(&mut self) -> f64 {
        // TODO: Should count be forced to be an integer?
        // If count_updated, calculate new noise & update state.
        if self.count_updated {
            self.noise = self.noise_distr.sample(&mut self.rng);
        }
        // Return count + noise
        self.count_updated = false;
        self.count + self.noise
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test_lp() {
        let mut counter = DPCounterLP::new(2.0);
        let votes = [true, true, false, true, false, false, true, true, true, true, true, false];
        for v in votes.iter() {
            counter.process_record(*v);
            println!("record: {}, true_count: {}, count: {}",
                     v, counter.get_true_count(), counter.get_count());
        }
    }
}

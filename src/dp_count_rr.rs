use std::f64;
use rand::thread_rng;
use rand::Rng;

// Differentially-private count using the randomized response (coin flip) method

// q is the probability that a given record is a lie.
pub struct DPCounterRR {
    q: f64,
    epsilon: f64,
    count: u32,
}

impl DPCounterRR {
    pub fn new(e: f64) -> DPCounterRR {
        DPCounterRR {
            q: 1.0/(1.0 + f64::consts::E.powf(e)),
            epsilon: e,
            count: 0,
        }
    }

    pub fn process_record(&mut self, r: bool) {
        if r {
            self.count += 1;
        }
    }

    pub fn get_epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn get_q(&self) -> f64 {
        self.q
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn create_record(&self, truth: bool) -> bool {
        let mut rng = thread_rng();
        let x: f64 = rng.gen();
        if x <= self.q {
            return truth
        }
        return !truth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test_rr() {
        // epsilon = ln(2), so q should be 1/3.
        let mut counter = DPCounterRR::new(f64::ln(2.0));
        assert!(counter.get_q()==(1.0/3.0));
        // TODO: how to test that value of count is within expected range?
        let votes = [true, true, false, true, false, false, true, true, true, true, true, false];
        let mut count = 0;
        for v in votes.iter() {
            if *v {
                count += 1;
            }
            let r = counter.create_record(*v);
            counter.process_record(r);
            println!("record: {}, true_count: {}, count: {}",
                     v, count, counter.get_count());
        }
    }
}

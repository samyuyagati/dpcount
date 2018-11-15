use std::f64;
use std::cmp::Ordering;
use rand;
use randomkit::dist::Laplace;
use randomkit::{Rng, Sample};
use std::collections::HashMap;

// Implementation on Binary Mechanism for finite T
pub struct BinaryMechanism {
    alphas: HashMap<u32, u32>,
    noisy_alphas: HashMap<u32, f64>,
    T: f64,
    t: f64,
    noise_distr: Laplace,
    rng: Rng,
    previous_output: f64,
}

impl BinaryMechanism {
    pub fn new(T: f64, e: f64) -> BinaryMechanism {
        BinaryMechanism {
            alphas: HashMap::new(),
            noisy_alphas: HashMap::new(),
            T: T,
            t: 1.0,
            noise_distr: Laplace::new(0.0, T.log2()/e).unwrap(),
            rng: Rng::from_seed(1),
            previous_output: 0.0,
        }
    }
    
    pub fn step_forward(&mut self, element: bool) -> f64 {
        if self.t > self.T {
            return self.previous_output;
        }

        // Get lowest nonzero bit
        let t_prime = self.t as i32;
        let i = ((t_prime & -t_prime) as f64).log(2.0) as u32;
//        println!("i: {}", i);

        // Create and store a new psum that includes this timestep
        let mut value = element as u32;
        for j in 0..i {
            value += *self.alphas.entry(j).or_insert(1000); // better default value to indicate error?
        }
        self.alphas.insert(
            i,
            value,
        );

        // Delete any psums contained in the new psum
        for j in 0..i {
            // Note: may have to zero instead of removing
            self.alphas.remove(&j);
            self.noisy_alphas.remove(&j);
        }

        // Update noisy_alphas
        self.noisy_alphas.insert(
            i,
            (value as f64) + self.noise_distr.sample(&mut self.rng),
        );
        
        // Calculate the output
        let t_bin = format!("{:b}", self.t as u32).chars().rev().collect::<String>();
        let mut output = 0.0;
        // for debugging
//        let mut true_output = 0;
        for char_index in t_bin.char_indices() {
            let (j, elt) = char_index;
            if elt == '1' {
                output += *self.noisy_alphas.entry(j as u32).or_insert(1000.0);
//                true_output += *self.alphas.entry(j as u32).or_insert(1000);
            }
        }
//        println!("True count: {}", true_output);
        
        // Update previous_output, increment t and t_bin, and return
        self.t += 1.0;
        self.previous_output = output;
        output
    }
}

// Implementation of the Logarithmic Mechanism
pub struct LogarithmicMechanism {
    beta: f64,
    t: f64,
    prev_output: f64,
    noise_distr: Laplace,
    rng: Rng,
}

impl LogarithmicMechanism {
    pub fn new(e: f64) -> LogarithmicMechanism {
        LogarithmicMechanism {
            beta: 0.0,
            t: 1.0,
            prev_output: 0.0,
            noise_distr: Laplace::new(0.0, 1.0/e).unwrap(),
            rng: Rng::from_seed(1),
        }
    }

    pub fn step_forward(&mut self, element: bool) -> f64 {
        self.beta += (element as u32) as f64;
        // If t is not a power of 2, return previous output
        if self.t.log2().floor() != self.t.log2().ceil() {
            self.t += 1.0;
            return self.prev_output
        }
        // t is a power of 2; update beta and return new output
        println!("UPDATE (t = {})", self.t);
        self.beta += self.noise_distr.sample(&mut self.rng);
        self.prev_output = self.beta;
        self.t += 1.0;
        self.beta
    }
}

// Implementation of the Hybrid Mechanism                                                                      
pub struct HybridMechanism {
    l: LogarithmicMechanism,
    b: BinaryMechanism,
    e: f64,
    t: f64,
}

impl HybridMechanism {
    pub fn new(e: f64) -> HybridMechanism {
        HybridMechanism {
            l: LogarithmicMechanism::new(e/2.0),
            b: BinaryMechanism::new(2.0, e/2.0),
            e: e,
            t: 1.0,
        }
    }

    pub fn step_forward(&mut self, element: bool) -> f64 {
        let l_out = self.l.step_forward(element);
        
        // If t is a power of 2, initialize new binary mechanism.
        if self.t > 1.0 && self.t.log2().floor() == self.t.log2().ceil() {
            self.b = BinaryMechanism::new(self.t, self.e/2.0);
            self.t += 1.0;
            return l_out
        }

        // t = 1 or t is not a power of 2; update binary mechanism.
        let b_out = self.b.step_forward(element);
        self.t += 1.0;
        l_out + b_out
    }
}

fn cmp_f64(a: &f64, b: &f64) -> Ordering {
    if a.is_nan() {
        return Ordering::Greater;
    }
    if b.is_nan() {
        return Ordering::Less;
    }
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_bin_mech_test() {
        let mut b = BinaryMechanism::new(8.0, 3.0);
        let stream = vec![true, false, true, true, false, false, false, true];
        let mut count = 0;
        for s in stream {
            count += s as u32;
            println!("True count: {}", count);
            let output = b.step_forward(s);
            println!("Noised count: {}", output);
        }
    }

    #[test]
    fn long_bin_mech_test() {
        let mut b = BinaryMechanism::new(100.0, 3.0);
        let mut stream = vec![];
        println!("True, Noised");
        let mut count = 0;
        for _t in 0..100 {
            let s = rand::random();
            stream.push(s);
            count += s as u32;
            let output = b.step_forward(s);
            println!("{}, {}", count, output);
        }
        println!("Stream: {:?}", stream);
    }

    #[test]
    fn log_mech_test() {
        let mut l = LogarithmicMechanism::new(3.0);
        let mut stream = vec![];
        println!("True, Noised");
        let mut count = 0;
        let mut outputs = vec![];

        let stream_length: f64  = 50.0;
        for t in 0..(stream_length as u32) {
            let s = rand::random();
            stream.push(s);
            count += s as u32;
            let output = l.step_forward(s);
            outputs.push(output);
            println!("t: {}, count: {}, output: {}", t + 1, count, output);
        }
        println!("Stream: {:?}", stream);
        outputs.sort_by(cmp_f64);
        outputs.dedup();
        assert_eq!(outputs.len(), (stream_length.log2().ceil() + 1.0) as usize);
    }

    #[test]
    fn hybrid_mech_test() {
        let mut l = HybridMechanism::new(3.0);
        let mut stream = vec![];
        println!("True, Noised");
        let mut count = 0;
        let mut outputs = vec![];

        let stream_length: f64  = 50.0;
        for t in 0..(stream_length as u32) {
            let s = rand::random();
            stream.push(s);
            count += s as u32;
            let output = l.step_forward(s);
            outputs.push(output);
            println!("t: {}, count: {}, output: {}, diff: {}", t + 1, count, output, count as f64 - output);
        }
    }
}

/// DSP skeleton for a delay effect. Contains basic circular buffer and parameters.

pub struct DelayProcessor {
    sample_rate: f32,
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    delay_ms: f32,
    feedback: f32, // 0..<1
    mix: f32,      // 0..1 (wet)
    gain_db: f32,  // output gain
}

impl DelayProcessor {
    pub fn new_with_sample_rate(sample_rate: f32) -> Self {
        let max_seconds = 5.0; // max delay buffer 5s
        let len = (sample_rate * max_seconds) as usize;
        Self {
            sample_rate,
            buffer: vec![0.0; len],
            write_pos: 0,
            delay_samples: 0,
            delay_ms: 0.0,
            feedback: 0.0,
            mix: 0.0,
            gain_db: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            let max_seconds = 5.0;
            let len = (sr * max_seconds) as usize;
            self.buffer.resize(len, 0.0);
            self.write_pos %= self.buffer.len();
        }
    }

    pub fn set_delay_ms(&mut self, ms: f32) {
        self.delay_ms = ms.clamp(0.0, 5000.0);
        self.delay_samples = ((self.delay_ms / 1000.0) * self.sample_rate) as usize % self.buffer.len();
    }

    pub fn set_feedback(&mut self, fb: f32) {
        self.feedback = fb.clamp(0.0, 0.99);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db = db.clamp(-60.0, 12.0);
    }

    fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Process a single input sample and return output sample.
    /// This is a simple delay: read delayed sample, write input + delayed*feedback, output mix.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if self.buffer.is_empty() { return input; }
        let len = self.buffer.len();
        let read_pos = if self.delay_samples <= self.write_pos {
            self.write_pos - self.delay_samples
        } else {
            (len + self.write_pos) - self.delay_samples
        } % len;

        let delayed = self.buffer[read_pos];
        let out = input * (1.0 - self.mix) + delayed * self.mix;

        // write to buffer: input + delayed * feedback
        self.buffer[self.write_pos] = input + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % len;

        out * Self::db_to_linear(self.gain_db)
    }

    /// Process an interleaved stereo buffer in-place (assumes f32 samples)
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_buffer_basic() {
        let mut p = DelayProcessor::new_with_sample_rate(48000.0);
        p.set_delay_ms(10.0);
        p.set_feedback(0.5);
        p.set_mix(1.0);
        p.set_gain_db(0.0);

        // feed a single impulse and process a few samples
        let mut out = vec![0.0_f32; 100];
        p.process_sample(1.0);
        for i in 0..99 {
            out[i] = p.process_sample(0.0);
        }
        // ensure some non-zero delayed samples appear
        assert!(out.iter().any(|&s| s != 0.0));
    }
}

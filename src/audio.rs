use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::dsp::DelayProcessor;

pub struct AudioEngine {
    _stream: cpal::Stream,
    processor: Arc<Mutex<DelayProcessor>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No output device found")?;
        
        println!("Output device connected");

        let config = device.default_output_config()?;
        println!("Output config: {:?}", config);

        let sr = config.sample_rate() as f32;
        let processor = Arc::new(Mutex::new(DelayProcessor::new_with_sample_rate(sr)));
        let proc_clone = Arc::clone(&processor);

        let stream = device.build_output_stream::<f32, _, _>(
            &config.config(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if let Ok(mut proc) = proc_clone.lock() {
                    for sample in data.iter_mut() {
                        // placeholder: no external input yet, process zeros through delay
                        *sample = proc.process_sample(0.0);
                    }
                } else {
                    for sample in data.iter_mut() { *sample = 0.0; }
                }
            },
            |err| { eprintln!("Stream error: {}", err); },
            None,
        )?;

        stream.play()?;

        Ok(Self { _stream: stream, processor })
    }

    pub fn set_gain_db(&self, gain_db: f32) {
        if let Ok(mut p) = self.processor.lock() {
            p.set_gain_db(gain_db);
        }
    }

    pub fn set_delay_ms(&self, ms: f32) {
        if let Ok(mut p) = self.processor.lock() {
            p.set_delay_ms(ms);
        }
    }

    pub fn set_feedback(&self, fb: f32) {
        if let Ok(mut p) = self.processor.lock() {
            p.set_feedback(fb);
        }
    }

    pub fn set_mix(&self, mix: f32) {
        if let Ok(mut p) = self.processor.lock() {
            p.set_mix(mix);
        }
    }
}

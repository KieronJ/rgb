use sdl2;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub const SPEC_SAMPLE_RATE: i32 = 44100;
pub const SPEC_CHANNELS: u8 = 2;
pub const SPEC_SAMPLES: u16 = 4096;

pub struct AudioSystem {
    buffer: Vec<i16>,
    device: AudioQueue<i16>,

    paused: bool,
}

impl AudioSystem {
    pub fn new(context: &sdl2::Sdl) -> AudioSystem {
        let audio_subsystem = context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(SPEC_SAMPLE_RATE),
            channels: Some(SPEC_CHANNELS),
            samples: Some(SPEC_SAMPLES),
        };

        let device = audio_subsystem.open_queue(None, &desired_spec).unwrap();

        AudioSystem {
            buffer: Vec::new(),
            device: device,

            paused: true,
        }
    }

    pub fn pause(&mut self) {
        self.device.pause();
        self.buffer.clear();

        self.paused = true;

    }

    pub fn resume(&mut self) {
        self.device.resume();
        self.paused = false;
    }

    pub fn add_samples(&mut self, samples: &[i16]) {
        if self.paused {
            return;
        }

        for sample in samples.iter() {
            self.buffer.push(*sample);
        }

        if self.buffer.len() >= (SPEC_SAMPLES as usize) {
            self.device.queue(&self.buffer[0..SPEC_SAMPLES as usize]);
            self.buffer.clear();
        }
    }
}
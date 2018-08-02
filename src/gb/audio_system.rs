use sdl2;
use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub struct AudioSystem {
    queue: AudioQueue<f32>,
}

impl AudioSystem {
    pub fn new(context: &sdl2::Sdl) -> AudioSystem {
        let audio_subsystem = context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None,
        };

        AudioSystem {
            queue: audio_subsystem.open_queue::<f32, _>(None, &desired_spec).unwrap(),
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.queue.queue(samples);
        self.queue.queue(samples);

        if self.queue.size() >= 2048 {
            self.queue.resume();
        }
    }

    pub fn pause(&mut self) {
        self.queue.pause();
    }

    pub fn play(&mut self) {
        self.queue.resume();
    }
}
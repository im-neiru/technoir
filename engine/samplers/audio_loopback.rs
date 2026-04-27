use wasapi::{
    AudioCaptureClient, AudioClient, Direction, SampleType, StreamMode, WaveFormat, initialize_mta,
};

#[allow(unused)]
pub struct AudioLoopback {
    capture: AudioCaptureClient,
    audio_client: AudioClient,
    buffer: Vec<u8>,
    mono: Vec<f32>,
}

impl AudioLoopback {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        initialize_mta().unwrap();

        let format = WaveFormat::new(32, 32, &SampleType::Float, 44100, 2, None);

        let mut audio_client =
            AudioClient::new_application_loopback_client(std::process::id(), false)
                .expect("Failed to create audio client");

        audio_client
            .initialize_client(
                &format,
                &Direction::Capture,
                &StreamMode::EventsShared {
                    autoconvert: true,
                    buffer_duration_hns: 200_000,
                },
            )
            .unwrap();

        let capture = audio_client.get_audiocaptureclient().unwrap();

        audio_client.start_stream().unwrap();

        Self {
            capture,
            audio_client,
            buffer: vec![0u8; 8192],
            mono: Vec::with_capacity(1024),
        }
    }

    pub fn poll(&mut self) {
        let _packet_size = match self.capture.get_next_packet_size() {
            Ok(Some(v)) if v > 0 => v,
            _ => return,
        };

        let Ok((frames, _info)) = self.capture.read_from_device(&mut self.buffer) else {
            return;
        };

        let samples = unsafe {
            std::slice::from_raw_parts(self.buffer.as_ptr() as *const f32, frames as usize * 2)
        };

        self.mono.clear();

        for i in 0..frames as usize {
            let l = samples[i * 2];
            let r = samples[i * 2 + 1];
            self.mono.push((l + r) * 0.5);
        }
    }

    pub fn samples(&self) -> &[f32] {
        &self.mono
    }
}

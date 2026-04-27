use rustfft::{FftPlanner, num_complex::Complex};
use wasapi::{
    AudioCaptureClient, AudioClient, Direction, SampleType, StreamMode, WaveFormat, initialize_mta,
};

pub struct SpectrumAudioLoopback {
    capture: AudioCaptureClient,
    _audio_client: AudioClient,
    buffer: Vec<u8>,

    left: Vec<f32>,
    right: Vec<f32>,

    left_fft: Vec<f32>,
    right_fft: Vec<f32>,

    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl SpectrumAudioLoopback {
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

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(1024);

        let scratch = vec![Complex::default(); 1024];

        Self {
            capture,
            _audio_client: audio_client,
            buffer: vec![0u8; 8192],

            left: Vec::with_capacity(1024),
            right: Vec::with_capacity(1024),

            left_fft: vec![0.0; 512],
            right_fft: vec![0.0; 512],

            fft,
            scratch,
        }
    }

    pub fn poll(&mut self) {
        let Ok((frames, _info)) = self.capture.read_from_device(&mut self.buffer) else {
            return;
        };

        let samples = unsafe {
            std::slice::from_raw_parts(self.buffer.as_ptr() as *const f32, frames as usize * 2)
        };

        self.left.clear();
        self.right.clear();

        let step = 4;

        let mut i = 0;
        while i < frames as usize && self.left.len() < 1024 {
            let l = samples[i * 2];
            let r = samples[i * 2 + 1];

            self.left.push(l);
            self.right.push(r);

            i += step;
        }

        self.compute_fft();
    }

    fn compute_fft(&mut self) {
        let fft = self.fft.as_ref();
        let scratch = &mut self.scratch;

        Self::apply_fft(fft, scratch, &self.left, &mut self.left_fft);
        Self::apply_fft(fft, scratch, &self.right, &mut self.right_fft);
    }

    fn apply_fft(
        fft: &dyn rustfft::Fft<f32>,
        scratch: &mut [Complex<f32>],
        input: &[f32],
        output: &mut [f32],
    ) {
        let n = scratch.len().min(input.len());

        for i in 0..n {
            scratch[i] = Complex {
                re: input[i],
                im: 0.0,
            };
        }

        for i in scratch.iter_mut().skip(n) {
            *i = Complex { re: 0.0, im: 0.0 };
        }

        fft.process(scratch);

        let half = scratch.len() / 2;

        for i in 0..half {
            let c = scratch[i];
            output[i] = (c.re * c.re + c.im * c.im).sqrt();
        }
    }

    pub fn left_spectrum(&self) -> &[f32] {
        &self.left_fft
    }

    pub fn right_spectrum(&self) -> &[f32] {
        &self.right_fft
    }
}

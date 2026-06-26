use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::Consumer;

struct StreamState {
    volume: f32,
    config: cpal::StreamConfig,
    sample_cons: ringbuf::HeapCons<f32>,
}

impl StreamState {
    pub fn audio_callback(&mut self, out: &mut [f32]) {
        for frame in out.chunks_mut(self.config.channels as usize) {
            let value = self
                .sample_cons
                .try_pop()
                .unwrap_or(cpal::Sample::EQUILIBRIUM)
                * self.volume;
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}

pub fn setup_audio_stream(emu: &mut umesen_core::Emulator) -> Result<cpal::Stream, cpal::Error> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))?;
    let config = device.default_output_config()?;

    let mut state = StreamState {
        config: config.into(),
        volume: 1.,
        sample_cons: emu
            .setup_audio_buffer(config.sample_rate(), std::time::Duration::from_millis(50)),
    };
    let stream = device.build_output_stream(
        config.into(),
        move |out: &mut [f32], _| state.audio_callback(out),
        |err| log::error!("Audio stream error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}

use chirm_encoding::{decoder::ChirmDecoder, encoder::ChirmEncoder};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encode_decode_sample_wav", |b| b.iter(|| encode_decode_sample_wav()));
}

fn encode_decode_sample_wav() -> Result<(),Box<dyn std::error::Error>> {
    let reader = hound::WavReader::open("res/sample.wav")?;
    let spec = reader.spec();
    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .filter_map(Result::ok)
        .collect();

    let sample_rate = spec.sample_rate as u32;
    let channels = spec.channels as usize;

    let mut encoder = ChirmEncoder::create()?;

    let frame_size = (sample_rate / 1000) * 20;
    let mut packets = Vec::new();

    let exact_chunks = samples.chunks_exact(frame_size as usize * channels);
    let remainder = exact_chunks.remainder();

    for chunk in exact_chunks {
        let packet = encoder.encode_sample(chunk)?;
        packets.push(packet);
    }

    if remainder.len() > 0 {
        let mut frame = remainder.to_vec();
        frame.resize(frame_size as usize, 0);
        let packet = encoder.encode_sample(&frame)?;
        packets.push(packet);
    }

    let mut decoder = ChirmDecoder::create()?;

    let mut pcm_buf: Vec<i16> = vec![];

    for packet in packets {
        let sample = decoder.decode_opus_frame(packet)?;
        pcm_buf.extend(sample);
    }
    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
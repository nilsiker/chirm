use crate::{Error, OpusFrame};

pub struct ChirmEncoder {
    encoder: opus::Encoder,
}

impl ChirmEncoder {
    pub fn create() -> Result<Self, Error> {
        Ok(Self {
            encoder: opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip)?,
        })
    }

    pub fn encode_sample(&mut self, chunk: &[i16]) -> Result<OpusFrame, Error> {
        let mut buf = [0; 960];
        let len = self.encoder.encode(chunk, &mut buf)?;
        Ok(OpusFrame(buf[..len].to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_encoder() -> Result<(), Error> {
        ChirmEncoder::create()?;
        Ok(())
    }
}

use crate::{Error, OpusFrame};


pub struct ChirmDecoder {
    decoder: opus::Decoder,
}

impl ChirmDecoder {
    pub fn create() -> Result<Self, Error> {
        Ok(Self {
            decoder: opus::Decoder::new(48000, opus::Channels::Mono)?,
        })
    }

    pub fn decode_opus_frame(&mut self, frame: OpusFrame) -> Result<Vec<i16>, Error> {
        let mut buf = [0; 960];
        let len = self.decoder.decode(&frame.0, &mut buf, true)?;
        Ok(buf[..len].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn create_decoder() -> Result<(), Error> {
        ChirmDecoder::create()?;
        Ok(())
    }
}

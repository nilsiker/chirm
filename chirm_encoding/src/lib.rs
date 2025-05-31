pub mod constants;
pub mod decoder;
pub mod encoder;

pub struct OpusFrame(pub Vec<u8>);

type Error = Box<dyn std::error::Error>;

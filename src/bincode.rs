use std::sync::LazyLock;

use bincode::{Decode, Encode, config::Configuration};
// use thiserror::Error;

// static BINCODE_CONFIG: LazyLock<Configuration<LittleEndian, Fixint>> = LazyLock::new(|| {
//     bincode::config::standard()
//         .with_fixed_int_encoding()
// });

static BINCODE_CONFIG: LazyLock<Configuration> = LazyLock::new(|| bincode::config::standard());

// #[derive(Error, Debug)]
// pub enum BincodeError {
//     #[error("Bincode encoding error: {0}")]
//     Encode(#[from] bincode::error::EncodeError),
//     #[error("Bincode decoding error: {0}")]
//     Decode(#[from] bincode::error::DecodeError),
//     #[error("IO error: {0}")]
//     Io(#[from] std::io::Error),
// }

pub struct Bincode;

impl Bincode {
    pub fn encode<T, W>(data: &T, writer: &mut W) -> usize
    where
        T: Encode,
        W: std::io::Write,
    {
        bincode::encode_into_std_write(data, writer, *BINCODE_CONFIG).unwrap()
    }

    pub fn decode<R, T>(reader: &mut R) -> T
    where
        R: std::io::Read,
        T: Decode<()>,
    {
        bincode::decode_from_std_read(reader, *BINCODE_CONFIG).unwrap()
    }

    pub fn encode_to_vec<T>(data: &T) -> Vec<u8>
    where
        T: Encode,
    {
        bincode::encode_to_vec(data, *BINCODE_CONFIG).unwrap()
    }

    pub fn decode_from_slice<T>(data: &[u8]) -> T
    where
        T: Decode<()>,
    {
        bincode::decode_from_slice(data, *BINCODE_CONFIG).unwrap().0
    }

    pub fn encode_to_slice<T>(data: &T, slice: &mut [u8]) -> usize
    where
        T: Encode,
    {
        bincode::encode_into_slice(data, slice, *BINCODE_CONFIG).unwrap()
    }
}

use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

pub trait SerDe {
    fn marshal<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
        T: Sized + Serialize;

    fn unmarshal<T>(&self, raw: &[u8]) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
        T: Sized + DeserializeOwned;
}

#[derive(Default)]
struct JsonSerDe {}

impl SerDe for JsonSerDe {
    fn marshal<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>
    where
        T: Sized + Serialize,
    {
        Ok(serde_json::to_vec(data)?)
    }

    fn unmarshal<T>(&self, raw: &[u8]) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: Sized + DeserializeOwned,
    {
        Ok(serde_json::from_slice(raw)?)
    }
}

pub fn json() -> impl SerDe {
    JsonSerDe {}
}

pub fn cbor() -> impl SerDe {
    CborSerDe {}
}

struct CborSerDe {}

impl SerDe for CborSerDe {
    fn marshal<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>
    where
        T: Sized + Serialize,
    {
        Ok(serde_cbor::to_vec(data)?)
    }

    fn unmarshal<T>(&self, raw: &[u8]) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: Sized + DeserializeOwned,
    {
        Ok(serde_cbor::from_slice(raw)?)
    }
}

pub(crate) fn marshal<T>(ser: impl SerDe, data: &T) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>
where
    T: Sized + Serialize,
{
    ser.marshal(data)
}

pub(crate) fn unmarshal<T>(de: impl SerDe, raw: &[u8]) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: Sized + DeserializeOwned,
{
    de.unmarshal(raw)
}

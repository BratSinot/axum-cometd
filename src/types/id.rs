use crate::error::ParseError;
use core::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct Id(Arc<[u32; 5]>);

impl Id {
    #[inline(always)]
    pub(crate) fn zero() -> Self {
        Id(Arc::from([0u32; 5]))
    }

    #[inline]
    pub(crate) fn gen() -> Self {
        use rand::Rng;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let lo = (timestamp & u128::from(u32::MAX)) as u32;
        let mid = ((timestamp >> 32) & u128::from(u32::MAX)) as u32;

        let mut id = Arc::<[_; 5]>::from([mid, lo, 0, 0, 0]);
        // SAFETY:
        //  Arc::get_mut().unwrap_unchecked() -- Arc just created, so counter at 1.
        //  get_unchecked_mut() -- Arc::<[_; 5]>.
        unsafe {
            let mut_id = Arc::get_mut(&mut id).unwrap_unchecked();
            let unfilled_id_part = mut_id.get_unchecked_mut(2..);
            rand::thread_rng().fill(unfilled_id_part);
        }

        Self(id)
    }

    #[inline]
    pub(crate) fn parse(str: &str) -> Result<Self, ParseError<'_>> {
        fn hex_str_to_u32(s: &str) -> Result<u32, ParseError<'_>> {
            u32::from_str_radix(s, 16).map_err(|_| ParseError::InvalidValue(s))
        }

        match str.len() {
            40 => {
                let (p0, p1, p2, p3, p4) = unsafe {
                    (
                        str.get_unchecked(0..8),
                        str.get_unchecked(8..16),
                        str.get_unchecked(16..24),
                        str.get_unchecked(24..32),
                        str.get_unchecked(32..40),
                    )
                };

                Ok(Self(Arc::from([
                    hex_str_to_u32(p0)?,
                    hex_str_to_u32(p1)?,
                    hex_str_to_u32(p2)?,
                    hex_str_to_u32(p3)?,
                    hex_str_to_u32(p4)?,
                ])))
            }
            len => Err(ParseError::InvalidLength(len)),
        }
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for u32_chunk in self.0.iter() {
            write!(f, "{u32_chunk:08x}")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Box::<str>::deserialize(deserializer)?;

        Self::parse(&str).map_err(ParseError::into_de_error)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::from_str;

    #[test]
    fn test_leading_zero() {
        let id = Id(Arc::from([
            0x0FFFFFFF,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            u32::MAX,
        ]));

        assert_eq!(id.to_string(), "0fffffffffffffffffffffffffffffffffffffff");

        let parsed_id = from_str::<Id>(r#""0fffffffffffffffffffffffffffffffffffffff""#).unwrap();
        assert_eq!(parsed_id, id);
    }
}

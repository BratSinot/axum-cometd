use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};

/// CometD ClientId.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ClientId([u32; 5]);

impl ClientId {
    #[inline]
    fn rotr(&mut self) {
        let [a0, a1, a2, a3, a4] = &mut self.0;
        let (b0, b1, b2, b3, b4) = (*a0 & 0b1, *a1 & 0b1, *a2 & 0b1, *a3 & 0b1, *a4 & 0b1);

        *a0 >>= 1;
        *a1 >>= 1;
        *a2 >>= 1;
        *a3 >>= 1;
        *a4 >>= 1;

        *a0 |= b4 << 31;
        *a1 |= b0 << 31;
        *a2 |= b1 << 31;
        *a3 |= b2 << 31;
        *a4 |= b3 << 31;
    }
}

impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for u32_chunk in self.0 {
            write!(f, "{u32_chunk:08x?}")?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for ClientId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let str = Box::<str>::deserialize(deserializer)?;
        match str.len() {
            40 => {
                let p0 = &str[0..8];
                let p1 = &str[8..16];
                let p2 = &str[16..24];
                let p3 = &str[24..32];
                let p4 = &str[32..40];

                let hex_str_to_u32 = |s| {
                    u32::from_str_radix(s, 16).map_err(|_| {
                        Error::invalid_value(Unexpected::Str(s), &"valid u32 hex string")
                    })
                };

                Ok(Self([
                    hex_str_to_u32(p0)?,
                    hex_str_to_u32(p1)?,
                    hex_str_to_u32(p2)?,
                    hex_str_to_u32(p3)?,
                    hex_str_to_u32(p4)?,
                ]))
            }
            len => Err(Error::invalid_length(len, &"40")),
        }
    }
}

impl Serialize for ClientId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Debug)]
pub(crate) struct ClientIdGen(ClientId);

impl ClientIdGen {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        use rand::Rng;

        let mut id = [0u32; 5];
        rand::thread_rng().fill(&mut id);

        Self(ClientId(id))
    }

    #[inline]
    pub(crate) fn next(&mut self) -> ClientId {
        let ret = self.0;
        self.0.rotr();
        ret
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::from_str;

    #[test]
    fn test_leading_zero() {
        let mut client_id = ClientId([u32::MAX; 5]);
        client_id.0[0] &= 0x0FFFFFFF;

        assert_eq!(
            client_id.to_string(),
            "0fffffffffffffffffffffffffffffffffffffff"
        );

        let parsed_client_id =
            from_str::<ClientId>(r#""0fffffffffffffffffffffffffffffffffffffff""#).unwrap();
        assert_eq!(parsed_client_id, client_id);
    }
}

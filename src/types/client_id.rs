use serde::{de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub(crate) struct ClientId([u32; 5]);

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
pub(crate) struct ClientIdGen;

impl ClientIdGen {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub(crate) fn next(&self) -> ClientId {
        use rand::Rng;

        let mut id = [0u32; 5];
        rand::thread_rng().fill(&mut id);
        // to prevent leading zero
        id[0] |= 0x10000000;

        ClientId(id)
    }
}

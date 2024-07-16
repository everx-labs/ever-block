use crate::{
    error::BlockError, messages::Message, Deserializable, Serializable, SERDE_OPTS_COMMON_MESSAGE,
    SERDE_OPTS_EMPTY, error, fail, Error, BuilderData, IBitstring, Result, SliceData
};

#[cfg(test)]
#[path = "tests/test_common_message.rs"]
mod tests;

const TAG_STD: usize = 0;
const TAG_MESH: usize = 1;


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommonMessage {
    Std(Message),
    // TODO create message variant for inter network routing
    Mesh(()),
}

impl Default for CommonMessage {
    fn default() -> Self {
        Self::Std(Message::default())
    }
}

impl CommonMessage {
    pub fn get_std(&self) -> Result<&Message> {
        match self {
            Self::Std(msg) => Ok(msg),
            _ => Err(self.unexpected_variant_error("CommonMessage::Std")),
        }
    }

    pub fn withdraw_std(self) -> Result<Message> {
        match self {
            Self::Std(msg) => Ok(msg),
            _ => Err(self.unexpected_variant_error("CommonMessage::Std")),
        }
    }

    pub fn get_std_mut(&mut self) -> Result<&mut Message> {
        match self {
            Self::Std(msg) => Ok(msg),
            _ => Err(self.unexpected_variant_error("CommonMessage::Std")),
        }
    }

    pub fn is_internal(&self) -> bool {
        match self {
            Self::Std(msg) => msg.is_internal(),
            _ => false,
        }
    }

    #[cfg(test)]
    fn default_mesh() -> Self {
        Self::Mesh(())
    }

    pub fn get_type_name(&self) -> String {
        match self {
            CommonMessage::Std(_) => "CommonMessage::Std",
            CommonMessage::Mesh(_) => "CommonMessage::Mesh",
        }
        .to_string()
    }

    fn unexpected_variant_error(&self, expected: &str) -> Error {
        error!(BlockError::UnexpectedStructVariant(
            expected.to_string(),
            self.get_type_name()
        ))
    }
}

impl std::fmt::Display for CommonMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommonMessage::Std(msg) => msg.fmt(f),
            CommonMessage::Mesh(_) => todo!("{:?}", self),
        }
    }
}

impl Serializable for CommonMessage {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        Self::write_with_opts(self, builder, SERDE_OPTS_EMPTY)
    }
    fn write_with_opts(&self, builder: &mut BuilderData, opts: u8) -> Result<()> {
        if opts == SERDE_OPTS_EMPTY {
            match self {
                CommonMessage::Std(msg) => msg.write_to(builder)?,
                _ => Err(self.unexpected_variant_error("CommonMessage::Std"))?,
            }
        }
        if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
            match self {
                CommonMessage::Std(msg) => {
                    builder.append_bits(TAG_STD, 8)?;
                    msg.write_to(builder)?;
                }
                CommonMessage::Mesh(_) => {
                    builder.append_bits(TAG_MESH, 8)?;
                    // TODO serialize mesh message
                }
            }
        }
        Ok(())
    }
}

impl Deserializable for CommonMessage {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let msg = Message::construct_from(cell)?;
        *self = CommonMessage::Std(msg);
        Ok(())
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        if opts == SERDE_OPTS_EMPTY {
            return self.read_from(slice);
        }
        if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
            let tag = slice.get_next_byte()? as usize;
            *self = match tag {
                TAG_STD => CommonMessage::Std(Message::construct_from(slice)?),
                TAG_MESH => CommonMessage::Mesh(()),
                _ => fail!(BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "CommonMessage".to_string()
                }),
            };
        } else {
            fail!(BlockError::UnsupportedSerdeOptions(
                "CommonMessage".to_string(),
                opts as usize
            ));
        }
        Ok(())
    }
}

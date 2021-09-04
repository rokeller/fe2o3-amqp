use std::convert::TryFrom;

use fe2o3_types::performatives::MaxFrameSize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::error::EngineError;

use super::protocol_header::{ProtocolHeader, ProtocolId};

pub struct Transport<T> {
    framed: Framed<T, LengthDelimitedCodec>
}

impl<T: AsyncRead + AsyncWrite + Unpin> Transport<T> {
    pub fn bind(io: T) -> Result<Self, EngineError> {
        let framed = LengthDelimitedCodec::builder()
            .big_endian()
            .length_field_length(4)
            .max_frame_length(usize::from(MaxFrameSize::default())) // change max frame size later in negotiation
            .length_adjustment(-4)
            .new_framed(io);
        Ok(
            Self { framed }
        )
    }

    pub async fn negotiate(io: &mut T, proto_header: ProtocolHeader) -> Result<ProtocolId, EngineError> {
        // negotiation
        let outbound_buf: [u8; 8] = proto_header.clone().into();
        io.write_all(&outbound_buf).await?;

        // wait for incoming header
        let mut inbound_buf = [0u8; 8];
        io.read_exact(&mut inbound_buf).await?;

        // check header
        let incoming_header = ProtocolHeader::try_from(inbound_buf)?;
        if incoming_header != proto_header {
            return Err(EngineError::UnexpectedProtocolHeader(inbound_buf))
        }
        Ok(incoming_header.id)
    }

    pub async fn negotiate_and_bind(mut io: T, proto_header: ProtocolHeader) -> Result<Self, EngineError> {
        // bind transport based on proto_id
        match Self::negotiate(&mut io, proto_header).await? {
            ProtocolId::Amqp => {
                Self::bind(io)
            },
            ProtocolId::Tls => todo!(),
            ProtocolId::Sasl => todo!()
        }
    }

    pub fn set_max_frame_size(&mut self, max_frame_size: usize) -> &mut Self {
        self.framed.codec_mut()
            .set_max_frame_length(max_frame_size);
        self
    }
}


#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio_util::codec::LengthDelimitedCodec;
    use futures::{StreamExt, SinkExt};

    #[tokio::test]
    async fn test_length_delimited_codec() {
        // test write
        let mut writer = vec![];
        let mut framed = LengthDelimitedCodec::builder()
            .big_endian()
            .length_field_length(4)
            .length_adjustment(-4)
            .new_write(&mut writer);

        let payload = Bytes::from("AMQP");
        framed.send(payload).await.unwrap();
        println!("{:?}", writer);

        // test read
        let reader = &writer[..];
        let mut framed = LengthDelimitedCodec::builder()
            .big_endian()
            .length_field_length(4)
            .length_adjustment(-4)
            .new_read(reader);
        let outcome = framed.next().await.unwrap();
        println!("{:?}", outcome)
    }
}
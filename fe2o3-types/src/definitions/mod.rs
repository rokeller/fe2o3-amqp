use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;

use fe2o3_amqp::{
    primitives::{Symbol, UInt},
    value::Value,
};

/// 2.8.1 Role
mod role;
pub use role::Role;

/// 2.8.2 Sender Settle Mode
mod snd_settle_mode;
pub use snd_settle_mode::SenderSettleMode;

/// 2.8.3 Receiver Settle Mode
mod rcv_settle_mode;
pub use rcv_settle_mode::ReceiverSettleMode;

/// 2.8.4 Handle
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Handle(pub UInt);

impl Default for Handle {
    fn default() -> Self {
        Handle(u32::MAX)
    }
}

/// 2.8.5 Seconds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Seconds(pub UInt);

impl Default for Seconds {
    fn default() -> Self {
        Seconds(0)
    }
}

/// 2.8.6 Milliseconds
pub type Milliseconds = UInt;

/// 2.8.7 Delivery Tag
pub type DeliveryTag = ByteBuf;

/// 2.8.8 Delivery Number
pub type DeliveryNumber = SequenceNo;

/// 2.8.9 Transfer Number
pub type TransferNumber = SequenceNo;

/// 2.8.10 Sequence No
pub type SequenceNo = UInt;

/// 2.8.11 Message Format
pub type MessageFormat = UInt;

/// 2.8.12 IETF Language Tag
/// TODO: impl Into
pub type IetfLanguageTag = Symbol;

/// 2.8.13 Fields
pub type Fields = BTreeMap<Symbol, Value>;

/// 2.8.14 Error
mod error;
pub use error::Error;

mod error_cond;
pub use error_cond::ErrorCondition;

/// 2.8.15 AMQP Error
mod amqp_error;
pub use amqp_error::AmqpError;

/// 2.8.16 Connection Error
mod conn_error;
pub use conn_error::ConnectionError;

/// 2.8.17 Session Error
mod session_error;
pub use session_error::SessionError;

/// 2.8.18 Link Error
mod link_error;
pub use link_error::LinkError;

/// 2.8.19 Constant definition
mod constant_def;
pub use constant_def::{MAJOR, MINOR, MIN_MAX_FRAME_SIZE, PORT, REVISION, SECURE_PORT};

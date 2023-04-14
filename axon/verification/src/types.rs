use axon_protocol::types::Bytes;

pub trait Message {
    fn encode(&self) -> Bytes;

    fn as_key(&self) -> &[u8];
}

pub enum VerifyError {
    FoundNoMessage,
    EventNotMatch,
    InvalidReceiptProof,
    InvalidTxProof,
    TxReceiptNotMatch,
}

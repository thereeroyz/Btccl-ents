use crate::relay::Error as RelayError;
use bitcoin::Error as BitcoinError;
use jsonrpc_http_server::jsonrpc_core::Error as JsonRpcError;
use parity_scale_codec::Error as CodecError;
use relayer_core::bitcoin::bitcoincore_rpc::Error as BtcRpcError;
use relayer_core::Error as CoreError;
use runtime::substrate_subxt::Error as XtError;
use runtime::Error as RuntimeError;
use std::net::AddrParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not verify that the oracle is offline")]
    CheckOracleOffline,
    #[error("Suggested status update does not contain block hash")]
    EventNoBlockHash,

    #[error("RuntimeError: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("BtcRpcError: {0}")]
    BtcRpcError(#[from] BtcRpcError),
    #[error("RelayError: {0}")]
    RelayError(#[from] RelayError),
    #[error("SubXtError: {0}")]
    SubXtError(#[from] XtError),
    #[error("CoreError: {0}")]
    CoreError(#[from] CoreError<RelayError>),
    #[error("AddrParseError: {0}")]
    AddrParseError(#[from] AddrParseError),
    #[error("CodecError: {0}")]
    CodecError(#[from] CodecError),
    #[error("BitcoinError: {0}")]
    BitcoinError(#[from] BitcoinError),
    #[error("JsonRpcError: {0}")]
    JsonRpcError(#[from] JsonRpcError),
}

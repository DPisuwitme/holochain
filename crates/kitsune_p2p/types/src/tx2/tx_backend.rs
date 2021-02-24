//! Types and Traits for writing tx2 backends.

use crate::tx2::util::TxUrl;
use crate::*;

use futures::future::BoxFuture;

/// Tx backend read stream type.
pub type InChan = Box<dyn futures::io::AsyncRead + 'static + Send + Unpin>;

/// Tx backend future resolves to InChan instance.
pub type InChanFut = BoxFuture<'static, KitsuneResult<InChan>>;

/// Tx backend future future resolves to InChan instance.
pub type InChanFutFut<'a> = BoxFuture<'a, KitsuneResult<InChanFut>>;

/// Tx backend adapter for incoming InChan instances.
/// This is NOT a futures::Stream to simplify backend implementations.
pub trait InChanRecvAdapt: 'static + Send + Unpin {
    /// Get the next future future that will resolve to an InChan instance.
    fn next(&mut self) -> InChanFutFut<'_>;
}

/// Tx backend write stream type.
pub type OutChan = Box<dyn futures::io::AsyncWrite + 'static + Send + Unpin>;

/// Tx backend future resolves to OutChan type.
pub type OutChanFut = BoxFuture<'static, KitsuneResult<OutChan>>;

/// Tx backend adapter represents an open connection to a remote.
pub trait ConAdapt: 'static + Send + Sync + Unpin {
    /// Get the string address (url) of the remote.
    fn remote_addr(&self) -> KitsuneResult<TxUrl>;

    /// Create a new outgoing channel to the remote.
    fn out_chan(&self, timeout: KitsuneTimeout) -> OutChanFut;

    /// Close this open connection (and all associated Chans).
    fn close(&self) -> BoxFuture<'static, ()>;
}

/// A tx backend Con is both the ability to make outgoing channels,
/// but also to receive incoming channels.
pub type Con = (Arc<dyn ConAdapt>, Box<dyn InChanRecvAdapt>);

/// Tx backend future resolves to a Con instance.
pub type ConFut = BoxFuture<'static, KitsuneResult<Con>>;

/// Tx backend future future resolves to a Con instance.
pub type ConFutFut<'a> = BoxFuture<'a, KitsuneResult<ConFut>>;

/// Tx backend adapter for incoming Con instances.
/// This is NOT a futures::Stream to simplify backend implementations.
pub trait ConRecvAdapt: 'static + Send + Unpin {
    /// Get the next future future that will resolve to a Con instance.
    fn next(&mut self) -> ConFutFut<'_>;
}

/// Tx backend adapter represents a bound local endpoint.
pub trait EndpointAdapt: 'static + Send + Sync + Unpin {
    /// Get the string address (url) of this binding.
    fn local_addr(&self) -> KitsuneResult<TxUrl>;

    /// Create a new outgoing connection to a remote.
    fn connect(&self, url: TxUrl, timeout: KitsuneTimeout) -> ConFut;

    /// Shutdown this endpoint / all connections / all channels.
    fn close(&self) -> BoxFuture<'static, ()>;
}

/// A tx backend Endpoint is both the ability to make outgoing connections,
/// but also to receive incoming connections.
pub type Endpoint = (Arc<dyn EndpointAdapt>, Box<dyn ConRecvAdapt>);

/// Tx backend future resolves to an Endpoint instance.
pub type EndpointFut = BoxFuture<'static, KitsuneResult<Endpoint>>;

/// Tx backend adapter represents the ability to bind local endpoints.
pub trait BackendAdapt: 'static + Send + Sync + Unpin {
    /// Bind a local endpoint, given a url spec.
    fn bind(url: TxUrl, timeout: KitsuneTimeout) -> EndpointFut;
}
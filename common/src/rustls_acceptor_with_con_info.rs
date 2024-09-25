use core::{
    fmt::Debug,
    convert::Infallible,
};
use std::{
    io,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use axum_server::tls_rustls::future::RustlsAcceptorFuture;
use tokio::task::futures::TaskLocalFuture;
use crate::{generate_debug, generate_empty_debug_non_exhaustive};
//--------------------------------------------------------------------------------------------------


//
// Actually we could use axum::extract::connect_info::Connected...
// but due to Rust design it would be a bit complicated implement outer trait
// for outer types (stream) :-)
//
pub trait ExtendableByConnectServiceService {
    fn extend_with_connect_info_from_ssl_stream<RawStream: RawTlStreamSpec>(self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self;
    fn install_connect_info_to(&self, extensions: &mut ConnectionStreamExtensions); // TODO: return Result
}


// Actually this class belongs to 'http' crate, but let's reuse it instead of recreation.
pub type ConnectionStreamExtensions = http::Extensions;

tokio::task_local! {
// tokio_inherit_task_local::inheritable_task_local! {
    static CONNECTION_STREAM_EXTENSION: Arc<ConnectionStreamExtensions>;
}


pub struct ServiceWrapper<S> {
    pub svc: S,
    pub connection_info: Option<Arc<ConnectionInfo>>, // T O D O: it is simple impl, try to do it more generic
}

impl<S> ServiceWrapper<S> {
    pub fn new(svc: S) -> Self {
        Self { svc, connection_info: None }
    }
}

#[cfg(feature = "tonic")]
pub type TonicTlsConnectInfo = tonic::transport::server::TlsConnectInfo<
    tonic::transport::server::TcpConnectInfo>;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConnectionInfo {
    pub peer_certs: Option<PeerCertificates>,
    #[cfg(feature = "tonic")]
    pub tonic_tls_con_info: Option<Arc<TonicTlsConnectInfo>>,
    #[doc(hidden)]
    __non_exhaustive: (),
}


#[derive(Clone)]
pub struct PeerCertificates {
    pub certs: Arc<Vec<rustls_pki_types::CertificateDer<'static>>>,
}
generate_empty_debug_non_exhaustive! { PeerCertificates }
impl PeerCertificates {
    pub fn new(certs: Arc<Vec<rustls_pki_types::CertificateDer<'static>>>) -> Self {
        Self { certs }
    }
}


fn get_connection_stream_extension() -> anyhow::Result<Arc<ConnectionStreamExtensions>> {
    let ext = CONNECTION_STREAM_EXTENSION
        .try_with(|v| v.clone());

    let ext = match ext {
        Ok(ext) => ext,
        Err(ref _err) =>
            // Or we can throw exception and force developer not oto use it
            // if server is not configured properly
            // return Ok(None),

            anyhow::bail!("Seems server is not configured to capture SSL/TLS stream info.\n \
                Make sure ServiceWrapper is used (by ServiceWrapperExt.into_make_service_with_con_info() \
                instead of standard axum into_make_service())
            ")
    };
    Ok(ext)
}

pub fn extract_cert_peers_from_axum_server_task_local()
    -> anyhow::Result<Option<Arc<Vec<rustls_pki_types::CertificateDer<'static>>>>> {

    let ext = get_connection_stream_extension() ?;
    let con_info = ext.get::<ConnectionInfo>();

    // ? What looks better: match or flat_map/map
    let peer_certs = match con_info {
        None => None,
        Some(con_info) => {
            match con_info.peer_certs {
                None => None,
                Some(ref peer_certs) =>
                    Some(peer_certs.certs.clone()),
            }
        }
    };

    Ok(peer_certs)
}

#[cfg(feature = "tonic")]
pub fn extract_tonic_tls_connect_info_from_axum_server_task_local()
    -> anyhow::Result<Option<tonic::transport::server::TlsConnectInfo<
        tonic::transport::server::TcpConnectInfo>>> {

    let ext = get_connection_stream_extension() ?;
    let con_info = ext.get::<ConnectionInfo>();

    // ? What looks better: match or flat_map/map
    Ok(con_info.and_then(|con_info|
        con_info.tonic_tls_con_info.as_ref()
            .map(|tls_con_info| tls_con_info.as_ref().clone())))
}


/// Internal trait to (conditionally by feature) satisfy 'tonic' requirements.
#[cfg(feature = "tonic")]
pub trait RawTlStreamSpec :
    tonic::transport::server::Connected<ConnectInfo = tonic::transport::server::TcpConnectInfo> {
}
#[cfg(feature = "tonic")]
impl<T> RawTlStreamSpec for T
where T: tonic::transport::server::Connected<ConnectInfo = tonic::transport::server::TcpConnectInfo>
{ }

#[cfg(not(feature = "tonic"))]
pub trait RawTlStreamSpec { }
#[cfg(not(feature = "tonic"))]
impl<T> RawTlStreamSpec for T { }


impl<S> ExtendableByConnectServiceService for ServiceWrapper<S> {
    fn extend_with_connect_info_from_ssl_stream<RawStream: RawTlStreamSpec>(
        self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self {

        let peer_certs = get_peer_certs(stream);
        #[cfg(feature = "tonic")]
        let tonic_tls_con_info = create_tonic_tls_connect_info(stream)
            .map(Arc::new);

        let connection_info = ConnectionInfo {
            peer_certs,
            #[cfg(feature = "tonic")]
            tonic_tls_con_info,
            __non_exhaustive: (),
        };

        Self {
            svc: self.svc,
            connection_info: Some(Arc::new(connection_info)),
        }
    }

    fn install_connect_info_to(&self, extensions: &mut ConnectionStreamExtensions) {
        match self.connection_info.as_ref() {
            None => {}
            Some(con_info) => {
                let con_info: ConnectionInfo = con_info.as_ref().clone();
                extensions.insert(con_info);
            }
        }
    }
}

fn get_peer_certs<RawStream>(stream: &tokio_rustls::server::TlsStream<RawStream>)
    -> Option<PeerCertificates> {
    let (_raw_stream, common_state) = stream.get_ref();
    let peer_certs = common_state.peer_certificates();
    let peer_certs = match peer_certs {
        None => None,
        Some(certs) => {
            let certs = certs.iter()
                .map(|cert| cert.clone())
                .collect::<Vec<_>>();

            if certs.is_empty() {
                None
            } else {
                Some(PeerCertificates::new(Arc::new(certs)))
            }
        }
    };
    peer_certs
}


#[cfg(feature = "tonic")]
#[allow(dead_code)]
fn create_tonic_tls_connect_info<
    RawStream: tonic::transport::server::Connected<ConnectInfo = tonic::transport::server::TcpConnectInfo>
> (stream: &tokio_rustls::server::TlsStream<RawStream>)
    -> Option<TonicTlsConnectInfo>
{
    use tonic::transport::server::Connected;
    use tokio_rustls::server::TlsStream;

    let con_info: TonicTlsConnectInfo = <TlsStream<RawStream>
        as Connected>::connect_info(stream);

    Some(con_info)
}


// #[derive(Debug,Clone)] is not used because pin_project_lite::pin_project does not support it :-(.
//
impl<S: Clone> Clone for ServiceWrapper<S> {
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
            connection_info: self.connection_info.as_ref()
                .map(|con_info|Arc::clone(&con_info)),
        }
    }
}
generate_debug! { ServiceWrapper<S: Debug>, svc }


#[derive(Debug, Clone)]
pub struct ServiceWrapperIntoMakeService<S> where S: Debug + Clone {
    pub svc: ServiceWrapper<S>,
}


impl ServiceWrapper<axum::Router<()>> {

    pub fn with_state(self, state: ()) -> ServiceWrapper<axum::Router<()>> {
        // call `Router::with_state` such that everything is turned into `Route` eagerly
        // rather than doing that per request
        let svc = self.svc.with_state(state);
        ServiceWrapper::<axum::Router<()>> {
            svc,
            connection_info: self.connection_info,
        }
    }

    pub fn into_make_service(self) -> ServiceWrapperIntoMakeService<axum::Router<()>> {
        // call `Router::with_state` such that everything is turned into `Route` eagerly
        // rather than doing that per request
        ServiceWrapperIntoMakeService { svc: self.with_state(()) }
    }
}

impl<S, T> tower_service::Service<T> for ServiceWrapperIntoMakeService<S>
where
    S: Debug + Clone,
{
    type Response = ServiceWrapper<S>;
    type Error = Infallible;
    type Future = ServiceWrapperIntoMakeServiceFuture<S>;

    #[inline]
    //noinspection DuplicatedCode
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _target: T) -> Self::Future {
        ServiceWrapperIntoMakeServiceFuture::new(self.svc.clone())
    }
}


pin_project_lite::pin_project! {
    pub struct ServiceWrapperIntoMakeServiceFuture<S> {
        #[pin] // Do I need pin there?
        svc: ServiceWrapper<S>,
    }
}

impl<S> ServiceWrapperIntoMakeServiceFuture<S> {
    #[inline]
    pub fn new(svc: ServiceWrapper<S>) -> Self {
        Self { svc }
    }
}
generate_empty_debug_non_exhaustive! { ServiceWrapperIntoMakeServiceFuture<S> }


// T O D O: try to remove Clone requirement later, but in easy way
impl<S: Debug + Clone> Future for ServiceWrapperIntoMakeServiceFuture<S>
where
    std::future::Ready<Result<S, Infallible>>: Future,
{
    type Output = Result<ServiceWrapper<S>, Infallible>;
    #[inline]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(self.svc.clone()))
    }
}


impl tower_service::Service<axum::serve::IncomingStream<'_>> for ServiceWrapper<axum::Router> {
    type Response = Self;
    type Error = Infallible;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    #[inline]
    //noinspection DuplicatedCode
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: axum::serve::IncomingStream<'_>) -> Self::Future {
        // call `Router::with_state` such that everything is turned into `Route` eagerly
        // rather than doing that per request
        std::future::ready(Ok(self.clone().with_state(())))
    }
}

impl<B> tower_service::Service<axum::extract::Request<B>> for ServiceWrapper<axum::Router>
where
    B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
    B::Error: Into<axum::BoxError>,
{
    type Response = axum::response::Response;
    type Error = Infallible;
    type Future = ServiceAxumRoutreWrapperCallFuture;
    // Box can be used if underlying/axum API is changeable
    // type Future = Pin<Box<dyn Future<Output=Result<axum::response::Response, Infallible>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <axum::Router as tower_service::Service<axum::extract::Request<B>>>
            ::poll_ready(&mut self.svc, _ctx)
    }

    fn call(&mut self, req: axum::extract::Request<B>) -> Self::Future {
        // let req = req.map(axum::body::Body::new);
        // self.svc.call_with_state(req, ())

        let mut stream_ext: ConnectionStreamExtensions = ConnectionStreamExtensions::new();
        <ServiceWrapper<axum::Router> as ExtendableByConnectServiceService>
            ::install_connect_info_to(self, &mut stream_ext);

        // delegate call
        let axum_router_fut = self.svc.call(req);

        let fut_wrapper= CONNECTION_STREAM_EXTENSION
            .scope(Arc::new(stream_ext), axum_router_fut);

        // Box can be used if underlying/axum API is changeable
        // Box::pin(fut_wrapper) // T O D O: try to avoid Box

        ServiceAxumRoutreWrapperCallFuture { delegate_fut: fut_wrapper }
    }
}

// Separate type is used to make ServiceWrapper::Future less changeable
// if/when we change task_local by some something else.
// In general, we could use
//   ServiceWrapper::Future = TaskLocalFuture<ConnectionStreamExtensions, axum::routing::future::RouteFuture<Infallible>>
pin_project_lite::pin_project! {
    pub struct ServiceAxumRoutreWrapperCallFuture {
        #[pin]
        delegate_fut: TaskLocalFuture<Arc<ConnectionStreamExtensions>, axum::routing::future::RouteFuture<Infallible>>,
    }
}
generate_debug! { ServiceAxumRoutreWrapperCallFuture, delegate_fut }

impl Future for ServiceAxumRoutreWrapperCallFuture {
    type Output = Result<axum::response::Response, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().delegate_fut.poll(cx)
    }
}



#[derive(Clone)]
pub struct ServiceWrapperRustlsAcceptor<A: Clone = axum_server::accept::DefaultAcceptor> {
    delegate: axum_server::tls_rustls::RustlsAcceptor<A>,
}
generate_empty_debug_non_exhaustive! { ServiceWrapperRustlsAcceptor<A: Clone> }


impl ServiceWrapperRustlsAcceptor {
    /// axum_server::tls_rustls::RustlsAcceptor API
    pub fn new(config: axum_server::tls_rustls::RustlsConfig) -> Self {
        Self {
            delegate: axum_server::tls_rustls::RustlsAcceptor::new(config),
        }
    }
    /// axum_server::tls_rustls::RustlsAcceptor API
    pub fn handshake_timeout(self, val: core::time::Duration) -> Self {
        Self {
            delegate: self.delegate.handshake_timeout(val),
        }
    }
}

impl<A: Clone> ServiceWrapperRustlsAcceptor<A> {
    /// axum_server::tls_rustls::RustlsAcceptor API
    pub fn acceptor<Acceptor: Clone>(self, acceptor: Acceptor) -> ServiceWrapperRustlsAcceptor<Acceptor> {
        ServiceWrapperRustlsAcceptor::<Acceptor> {
            delegate: self.delegate.acceptor(acceptor)
        }
    }
}



impl<A, I, S> axum_server::accept::Accept<I, S> for ServiceWrapperRustlsAcceptor<A>
where
    A: Clone + axum_server::accept::Accept<I, S>,
    A::Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    A::Service: ExtendableByConnectServiceService,
    <A as axum_server::accept::Accept<I, S>>::Service: ExtendableByConnectServiceService,
    <A as axum_server::accept::Accept<I, S>>::Stream: RawTlStreamSpec
{
    type Stream = tokio_rustls::server::TlsStream<A::Stream>;
    type Service = A::Service;
    type Future = ServiceWrapperRustlsAcceptorFuture<A::Future, A::Stream, A::Service>;
    // type Future = Map<RustlsAcceptorFuture2_2<A::Future, A::Stream, A::Service>, MapperStruct>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let accept_fut = self.delegate.accept(stream, service);
        let fut: ServiceWrapperRustlsAcceptorFuture<A::Future, A::Stream, A::Service> =
            ServiceWrapperRustlsAcceptorFuture::new(accept_fut);
        fut
    }
}


// Seems pin_project_lite::pin_project does not support new-type approach
// (we need to use classical struct).
//
// pub struct RustlsAcceptorFuture2_2<F, I, S>(RustlsAcceptorFuture<F, I, S>);


pin_project_lite::pin_project! {
    pub struct ServiceWrapperRustlsAcceptorFuture<F, I, S> {
        #[pin]
        delegate_fut: RustlsAcceptorFuture<F, I, S>
    }
}
generate_empty_debug_non_exhaustive! { ServiceWrapperRustlsAcceptorFuture<F, I, S> }


impl<F, I, S> ServiceWrapperRustlsAcceptorFuture<F, I, S> {
    pub fn new(rustls_acceptor_future: RustlsAcceptorFuture<F, I, S>) -> Self {
        Self { delegate_fut: rustls_acceptor_future }
    }
}


impl<F, I, S> Future for ServiceWrapperRustlsAcceptorFuture<F, I, S>
where
    F: Future<Output = io::Result<(I, S)>>,
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    I: RawTlStreamSpec,
    S: ExtendableByConnectServiceService, // <I>,
{
    type Output = io::Result<(tokio_rustls::server::TlsStream<I>, S)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll_res = self.project().delegate_fut.poll(cx);

        match poll_res {
            Poll::Ready(Ok((stream, service))) => {
                let service = service.extend_with_connect_info_from_ssl_stream(&stream);
                Poll::Ready(Ok((stream, service)))
            }
            Poll::Ready(other_ready) =>
                Poll::Ready(other_ready),
            Poll::Pending =>
                Poll::Pending,
        }
    }
}


#[extension_trait::extension_trait]
pub impl ServiceWrapperExt for axum::Router<()> {
    fn into_make_service_with_con_info(self) -> ServiceWrapperIntoMakeService<axum::Router<()>> {
        ServiceWrapper::<axum::Router<()>>::new(self).into_make_service()
    }
}

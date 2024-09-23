use core::{
    fmt::{self, Debug},
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
use crate::{generate_empty_debug_non_exhaustive_delegate};
//--------------------------------------------------------------------------------------------------


//
// Actually we could use axum::extract::connect_info::Connected...
// but due to Rust design it would be a bit complicated implement outer trait
// for outer types (stream) :-)
//
pub trait ExtendableByConnectServiceService {
    fn extend_with_connect_info_from_ssl_stream<RawStream>(self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self;
    fn install_connect_info_to(&self, extensions: &mut ConnectionStreamExtensions); // TODO: return Result
}

/// Actually this class belongs to 'http', but let's reuse it.
pub type ConnectionStreamExtensions = http::Extensions;

tokio::task_local! {
// tokio_inherit_task_local::inheritable_task_local! {
    pub static CONNECTION_STREAM_EXTENSION: ConnectionStreamExtensions;
}

pub struct ServiceWrapper<S> {
    pub svc: S,
    pub connection_info: Option<Arc<ConnectionInfo>>, // TODO: it is simple impl, try to do it more generic
}

impl<S> ServiceWrapper<S> {
    pub fn new(svc: S) -> Self {
        Self { svc, connection_info: None }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_certs: Option<PeerCertificates>,
}

#[derive(Clone)]
pub struct PeerCertificates {
    pub certs: Vec<rustls_pki_types::CertificateDer<'static>>,
}
generate_empty_debug_non_exhaustive_delegate! { PeerCertificates }

impl<S> ExtendableByConnectServiceService for ServiceWrapper<S> {
    fn extend_with_connect_info_from_ssl_stream<RawStream>(
        self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self {

        let peer_certs = get_peer_certs(stream);
        let connection_info = ConnectionInfo { peer_certs };

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
                Some(PeerCertificates { certs })
            }
        }
    };
    peer_certs
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
impl<S: Debug> Debug for ServiceWrapper<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceWrapper")
            .field("svc", &self.svc)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct MyIntoMakeService<S> where S: Debug + Clone {
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

    pub fn into_make_service(self) -> MyIntoMakeService<axum::Router<()>> {
        // call `Router::with_state` such that everything is turned into `Route` eagerly
        // rather than doing that per request
        MyIntoMakeService { svc: self.with_state(()) }
    }
}

impl<S, T> tower_service::Service<T> for MyIntoMakeService<S>
where
    S: Debug + Clone,
{
    type Response = ServiceWrapper<S>;
    type Error = Infallible;
    type Future = MyIntoMakeServiceFuture2<S>;

    #[inline]
    //noinspection DuplicatedCode
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _target: T) -> Self::Future {
        MyIntoMakeServiceFuture2::new(self.svc.clone())
    }
}


pin_project_lite::pin_project! {
    pub struct MyIntoMakeServiceFuture2<S> {
        #[pin] // Do I need pin there?
        svc: ServiceWrapper<S>,
    }
}

impl<S> MyIntoMakeServiceFuture2<S> {
    #[inline]
    pub fn new(svc: ServiceWrapper<S>) -> Self {
        Self { svc }
    }
}
generate_empty_debug_non_exhaustive_delegate! { MyIntoMakeServiceFuture2, S }

// T O D O: try to remove Clone requirement later, but in easy way
impl<S: Debug + Clone> Future for MyIntoMakeServiceFuture2<S>
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
    type Future = Pin<Box<dyn Future<Output=Result<axum::response::Response, Infallible>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <axum::Router as tower_service::Service<axum::extract::Request<B>>>
            ::poll_ready(&mut self.svc, _ctx)
    }

    fn call(&mut self, req: axum::extract::Request<B>) -> Self::Future {
        // let req = req.map(axum::body::Body::new);
        // self.svc.call_with_state(req, ())

        let mut stream_ext: ConnectionStreamExtensions = ConnectionStreamExtensions::new();
        <ServiceWrapper<axum::Router> as ExtendableByConnectServiceService>::install_connect_info_to(self, &mut stream_ext);

        // delegate call
        let axum_router_fut = self.svc.call(req);

        let fut_wrapper= CONNECTION_STREAM_EXTENSION
            .scope(stream_ext, axum_router_fut);

        // let abc: Pin<Box<dyn Future<Output=Result<axum::response::Response, Infallible>> + Send>> =
        //     Box::pin(fut_wrapper);
        // abc
        Box::pin(fut_wrapper) // TODO: try to avoid Box
    }
}



#[derive(Clone)]
pub struct MyRustlsAcceptor2<A: Clone = axum_server::accept::DefaultAcceptor> {
    delegate: axum_server::tls_rustls::RustlsAcceptor<A>,
}

impl MyRustlsAcceptor2 {
    // axum_server::tls_rustls::RustlsAcceptor API
    pub fn new(config: axum_server::tls_rustls::RustlsConfig) -> Self {
        Self {
            delegate: axum_server::tls_rustls::RustlsAcceptor::new(config),
        }
    }
    // axum_server::tls_rustls::RustlsAcceptor API
    pub fn handshake_timeout(self, val: core::time::Duration) -> Self {
        Self {
            delegate: self.delegate.handshake_timeout(val),
        }
    }
}

impl<A: Clone> MyRustlsAcceptor2<A> {
    // axum_server::tls_rustls::RustlsAcceptor API
    pub fn acceptor<Acceptor: Clone>(self, acceptor: Acceptor) -> MyRustlsAcceptor2<Acceptor> {
        MyRustlsAcceptor2::<Acceptor> {
            delegate: self.delegate.acceptor(acceptor)
        }
    }
}



impl<A, I, S> axum_server::accept::Accept<I, S> for MyRustlsAcceptor2<A>
where
    A: Clone + axum_server::accept::Accept<I, S>,
    A::Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    A::Service: ExtendableByConnectServiceService,
    <A as axum_server::accept::Accept<I, S>>::Service: ExtendableByConnectServiceService,
{
    type Stream = tokio_rustls::server::TlsStream<A::Stream>;
    type Service = A::Service;
    type Future = RustlsAcceptorFuture2_2<A::Future, A::Stream, A::Service>;
    // type Future = Map<RustlsAcceptorFuture2_2<A::Future, A::Stream, A::Service>, MapperStruct>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let accept_fut = self.delegate.accept(stream, service);
        let fut: RustlsAcceptorFuture2_2<A::Future, A::Stream, A::Service> =
            RustlsAcceptorFuture2_2::new(accept_fut);
        fut
    }
}

// generate_empty_debug_non_exhaustive_delegate! { MyRustlsAcceptor2, A } // TODO: try to use it
impl<A: Clone> Debug for MyRustlsAcceptor2<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(MyRustlsAcceptor2)).finish()
    }
}

// Seems pin_project_lite::pin_project does not support new-type approach
// (we need to use classical struct).
//
// pub struct RustlsAcceptorFuture2_2<F, I, S>(RustlsAcceptorFuture<F, I, S>);


pin_project_lite::pin_project! {
    pub struct RustlsAcceptorFuture2_2<F, I, S> {
        #[pin]
        delegate_fut: RustlsAcceptorFuture<F, I, S>
    }
}

impl<F, I, S> RustlsAcceptorFuture2_2<F, I, S> {
    pub fn new(rustls_acceptor_future: RustlsAcceptorFuture<F, I, S>) -> Self {
        Self { delegate_fut: rustls_acceptor_future }
    }
}

generate_empty_debug_non_exhaustive_delegate! { RustlsAcceptorFuture2_2, F, I, S }


impl<F, I, S> Future for RustlsAcceptorFuture2_2<F, I, S>
where
    F: Future<Output = io::Result<(I, S)>>,
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
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
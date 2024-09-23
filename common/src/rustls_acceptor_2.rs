use core::fmt::{self, Debug};
use std::io;
use std::convert::Infallible;
use std::future::Future;
// use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
// use crate::client_cert_auth::ONE_BLA_BLA_2;
// use crate::server::BoxFuture;
// use axum_server::service::MakeService;
// use http_body::Body;
// use tower_service::Service;
//--------------------------------------------------------------------------------------------------

//
//
// Actually we could use axum::extract::connect_info::Connected...
// but due to Rust design it would be a bit complicated implement outer trait
// for outer types (stream) :-)
//
pub trait ExtendableByConnectServiceService {
    fn extend_with_connect_info_from<RawStream>(self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self;
    fn install_connect_info_to(&self, extensions: &mut ConnectionStreamExtensions); // TODO: return Result
}

/// Actually this class belongs to 'http', but let's reuse it.
pub type ConnectionStreamExtensions = http::Extensions;

tokio::task_local! {
// tokio_inherit_task_local::inheritable_task_local! {
    pub static CONNECTION_STREAM_EXTENSION: ConnectionStreamExtensions;
}

pub struct ServiceWrapper<S> /*where S: Clone + Debug*/ {
    pub svc: S,
    pub connection_info: Option<Arc<ConnectionInfo>>, // TODO: it is simple impl, try to do it more generic
}

impl<S> ServiceWrapper<S> {
    pub fn new(svc: S) -> Self {
        Self {
            svc,
            connection_info: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_certs: Option<PeerCertificates>,
}

#[derive(Clone, Debug)] // TODO: implement faking Debug
pub struct PeerCertificates {
    pub certs: Vec<rustls_pki_types::CertificateDer<'static>>,
}

impl<S> ExtendableByConnectServiceService for ServiceWrapper<S> {
    fn extend_with_connect_info_from<RawStream>(self, stream: &tokio_rustls::server::TlsStream<RawStream>) -> Self {
        let peer_certs = get_peer_certs(stream);
        let connection_info = ConnectionInfo {
            peer_certs,
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
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: T) -> Self::Future {
        MyIntoMakeServiceFuture2::new(self.svc.clone())
    }
}


pin_project_lite::pin_project! {
    pub struct MyIntoMakeServiceFuture<S> {
        #[pin]
        future: std::future::Ready<Result<S, Infallible>>,
    }
}

impl<S> MyIntoMakeServiceFuture<S> {
    pub fn new(future: std::future::Ready<Result<S, Infallible>>)
        -> Self {
        Self { future }
    }
}
impl<S> Debug for MyIntoMakeServiceFuture<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyIntoMakeServiceFuture").finish_non_exhaustive()
    }
}
impl<S> Future for MyIntoMakeServiceFuture<S>
where
    std::future::Ready<Result<S, Infallible>>: Future,
{
    type Output = <std::future::Ready<Result<S, Infallible>>
        as Future>::Output;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

pin_project_lite::pin_project! {
    pub struct MyIntoMakeServiceFuture2<S> {
        // #[pin]
        svc: ServiceWrapper<S>,
    }
}

impl<S: Debug + Clone> MyIntoMakeServiceFuture2<S> {
    pub fn new(svc: ServiceWrapper<S>) -> Self {
        Self { svc }
    }
}
impl<S: Debug> Debug for MyIntoMakeServiceFuture2<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyIntoMakeServiceFuture").finish_non_exhaustive()
    }
}
impl<S: Debug + Clone> Future for MyIntoMakeServiceFuture2<S> // T O D O: try remove Clone requirement later
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
        // Poll::Ready(Ok(()))
        <axum::Router as tower_service::Service<axum::extract::Request<B>>>
            ::poll_ready(&mut self.svc, _ctx)
    }

    #[inline]
    fn call(&mut self, req: axum::extract::Request<B>) -> Self::Future {
        // let req = req.map(axum::body::Body::new);
        // self.svc.call_with_state(req, ())

        let mut stream_ext: ConnectionStreamExtensions = ConnectionStreamExtensions::new();
        <ServiceWrapper<axum::Router> as ExtendableByConnectServiceService>::install_connect_info_to(self, &mut stream_ext);

        let axum_router_fut = self.svc.call(req);

        let fut_wrapper= CONNECTION_STREAM_EXTENSION
            .scope(stream_ext, axum_router_fut);

        let abc: Pin<Box<dyn Future<Output=Result<axum::response::Response, Infallible>> + Send>> =
            Box::pin(fut_wrapper);
        abc
    }
}


/*
// trait TestTrait345 {}
// impl<T> TestTrait345 for ServiceWrapper<T> {}

impl<T, B, Request> axum_server::service::SendService<Request> for ServiceWrapper<T>
where
    // ServiceWrapper<T>: TestTrait345,
    T: Debug,
    T: tower_service::Service<Request, Response = http::response::Response<B>> + Send + Clone + 'static,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T::Future: Send + 'static,
    B: http_body::Body + Send + 'static,
    B::Data: Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Service = ServiceWrapper<T>;

    type Body = B;
    type BodyData = B::Data;
    type BodyError = B::Error;

    type Error = T::Error;
    type Future = T::Future;

    fn into_service(self) -> Self::Service {
        self
    }
}
*/

/*
impl<T, S, B, E, F, Target, Request> axum_server::service::MakeService<Target, Request>
    for MyIntoMakeService<ServiceWrapper<T>>
where
    T: Debug + Clone,
    MyIntoMakeService<ServiceWrapper<T>>: tower_service::Service<Target, Response = S, Error = E, Future = F>,
    S: tower_service::Service<Request, Response = http::Response<B>> + Send + Clone + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
    B: http_body::Body + Send + 'static,
    B::Data: Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
    F: Future<Output = Result<S, E>>,
{
    type Service = S;

    type Body = B;
    type BodyData = B::Data;
    type BodyError = B::Error;

    type Error = S::Error;

    type Future = S::Future;

    type MakeError = E;
    type MakeFuture = F;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::MakeError>> {
        self.poll_ready(cx)
    }

    fn make_service(&mut self, target: Target) -> Self::MakeFuture {
        // self.call(target)
        <Self as tower_service::Service<Target, Response=S, Error=E, Future=F>>
            ::call(self, target)
        // t o d o!("9897979")
    }
}
*/


#[derive(Clone)]
pub struct MyRustlsAcceptor2<A: Clone = axum_server::accept::DefaultAcceptor> {
    delegate: axum_server::tls_rustls::RustlsAcceptor<A>,
}

impl MyRustlsAcceptor2 {
    pub fn new(config: axum_server::tls_rustls::RustlsConfig) -> Self {
        Self {
            delegate: axum_server::tls_rustls::RustlsAcceptor::new(config),
        }
    }
    pub fn handshake_timeout(self, val: core::time::Duration) -> Self {
        Self {
            delegate: self.delegate.handshake_timeout(val),
        }
    }
}

impl<A: Clone> MyRustlsAcceptor2<A> {
    /// Overwrite inner acceptor.
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
    // S: ExtendableByConnectServiceService<I>,
    A::Service: ExtendableByConnectServiceService,
    // <A as axum_server::accept::Accept<I, S>>::Service: ExtendableByConnectServiceService<<A as axum_server::accept::Accept<I, S>>::Stream>,
    <A as axum_server::accept::Accept<I, S>>::Service: ExtendableByConnectServiceService,
{
    type Stream = tokio_rustls::server::TlsStream<A::Stream>;
    type Service = A::Service;
    // type Future = axum_server::tls_rustls::future::RustlsAcceptorFuture<A::Future, A::Stream, A::Service>;
    type Future = RustlsAcceptorFuture2<A::Future, A::Stream, A::Service>;
    // type Future = MyRustlsAcceptorFuture<A::Future, A::Stream, A::Service>;
    // type Future = BoxFuture<io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        /*
        let inner_future = self.inner.accept(stream, service);
        let config = self.config.clone();

        MyRustlsAcceptorFuture::new(inner_future, config, self.handshake_timeout)
        */
        // MyRustlsAcceptorFuture::<A::Future, A::Stream, A::Service> {
        //     delegate: self.delegate.accept(stream, service),
        // }

        let _delegate = self.delegate.clone();
        // Box::pin( async move { delegate.accept(stream, service).await })
        // Box::pin( delegate.accept(stream, service) )
        // delegate.accept(stream, service)


        let inner_future = self.delegate.inner.accept(stream, service);
        let config = self.delegate.config.clone();

        let fut: RustlsAcceptorFuture2<A::Future, A::Stream, A::Service> =
            RustlsAcceptorFuture2::new(
                inner_future, config, self.delegate.handshake_timeout);
        fut
        // Box::pin(fut)
        // t o d o!("bvnbvbnvn")
    }
}

impl<A: Clone> fmt::Debug for MyRustlsAcceptor2<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyRustlsAcceptor").finish()
    }
}


/*
pin_project! {
    /// Future type for [`RustlsAcceptor`](crate::tls_rustls::RustlsAcceptor).
    pub struct RustlsAcceptorFuture<F, I, S> {
        #[pin]
        inner: AcceptFuture<F, I, S>,
        config: Option<RustlsConfig>,
    }
}
*/



pin_project_lite::pin_project! {
    /// Future type for [`RustlsAcceptor`](crate::tls_rustls::RustlsAcceptor).
    pub struct RustlsAcceptorFuture2<F, I, S> {
        #[pin]
        inner: AcceptFuture<F, I, S>,
        config: Option<axum_server::tls_rustls::RustlsConfig>,
    }
}

impl<F, I, S> RustlsAcceptorFuture2<F, I, S> {
    pub fn new(future: F,
               config: axum_server::tls_rustls::RustlsConfig,
               handshake_timeout: core::time::Duration,
    ) -> Self {
        let inner = AcceptFuture::Inner {
            future,
            handshake_timeout,
        };
        let config = Some(config);

        Self { inner, config }
    }
}

impl<F, I, S> Debug for RustlsAcceptorFuture2<F, I, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RustlsAcceptorFuture").finish()
    }
}

pin_project_lite::pin_project! {
    #[project = AcceptFutureProj]
    enum AcceptFuture<F, I, S> {
        Inner {
            #[pin]
            future: F,
            handshake_timeout: core::time::Duration,
        },
        Accept {
            #[pin]
            future: tokio::time::Timeout<tokio_rustls::Accept<I>>,
            service: Option<S>,
        },
    }
}

impl<F, I, S> Future for RustlsAcceptorFuture2<F, I, S>
where
    F: Future<Output = io::Result<(I, S)>>,
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    S: ExtendableByConnectServiceService, // <I>,
{
    type Output = io::Result<(tokio_rustls::server::TlsStream<I>, S)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        use std::io;

        let mut this = self.project();

        // ONE_BLA_BLA_2.sync_scope(987, ||{

        loop {
            match this.inner.as_mut().project() {
                AcceptFutureProj::Inner {
                    future,
                    handshake_timeout,
                } => {
                    match future.poll(cx) {
                        Poll::Ready(Ok((stream, service))) => {
                            let server_config = this.config
                                .take()
                                .expect("config is not set. this is a bug in axum-server, please report")
                                .get_inner();

                            let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
                            let future = acceptor.accept(stream);

                            let service = Some(service);
                            let handshake_timeout = *handshake_timeout;

                            this.inner.set(AcceptFuture::Accept {
                                future: tokio::time::timeout(handshake_timeout, future),
                                service,
                            });
                        }
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => return Poll::Pending,
                    }
                }
                AcceptFutureProj::Accept { future, service } => match future.poll(cx) {
                    Poll::Ready(Ok(Ok(stream))) => {
                        let service = service.take().expect("future polled after ready");
                        let service = service.extend_with_connect_info_from(&stream);

                        return Poll::Ready(Ok((stream, service)));
                    }
                    Poll::Ready(Ok(Err(e))) => return Poll::Ready(Err(e)),
                    Poll::Ready(Err(timeout)) => {
                        return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, timeout)))
                    }
                    Poll::Pending => return Poll::Pending,
                },
            }
        }
        // })
    }
}



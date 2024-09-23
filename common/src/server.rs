// use core::fmt;
use core::time::Duration;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
// use std::task::{Context, Poll};
// use axum_server::accept::DefaultAcceptor;
use log::info;
// use tokio::io::{AsyncRead, AsyncWrite};
use tokio::signal;
// use tracing_subscriber::filter::FilterExt;
use crate::cfg::{ServerConf, SslConfValue};
use crate::client_cert_auth::{
    // ONE_BLA_BLA,
    // ONE_BLA_BLA_2,
    server_rustls_with_ssl_cert_client_auth_config,
};
use crate::net::ConnectionType;
use crate::rustls_acceptor_2::{MyRustlsAcceptor2, ServiceWrapper};
//--------------------------------------------------------------------------------------------------


pub async fn start_axum_server<Conf: ServerConf>(server_conf: Conf, app_router: axum::routing::Router) -> anyhow::Result<()> {

    let connection_type = server_conf.connection_type();
    let port = server_conf.server_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let handle = axum_server::Handle::new();
    tokio::spawn(axum_server_shutdown_signal(
        handle.clone(), server_conf.shutdown_timeout()));

    match connection_type {
        ConnectionType::Plain => {

            // Using axum-server third-party crate
            info!("Web server started on plain port [{port}]");
            axum_server::bind(addr)
                .handle(handle)
                .serve(app_router.into_make_service())
                // .serve(ServiceWrapper { svc: app_router }.into_make_service())
                // .serve(app_router.into_make_service_with_connect_info::<AAConInfo>())
                .await ?;

            // Using axum core
            /*
            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
            info!("Web server started on port [{port}]");
            axum::serve(listener, app_router)
                // T O D O: how to use shutdown timeout with this axum core?
                .with_graceful_shutdown(axum_core_shutdown_signal()) ???
                .await ?;
            */
        }

        ConnectionType::Ssl => {

            let rust_tls_config =
                if server_conf.client_auth_ssl_ca_cert().is_some() {
                    server_rustls_with_ssl_cert_client_auth_config(&server_conf).await ?
                } else {
                    server_rustls_config(&server_conf).await ?
                };

            info!("Web server started on ssl port [{port}]");
            axum_server_bind_rustls_custom(addr, rust_tls_config)
                .handle(handle)
                // .serve(app_router.into_make_service())
                .serve(ServiceWrapper::new(app_router).into_make_service())
                .await ?;
        }

        ConnectionType::Auto =>
            anyhow::bail!("Server connection type auto detection is not supported"),
    }

    Ok(())
}


/*
#[derive(Clone)]
pub struct AAConInfo {
    addr: Option<std::net::SocketAddr>,
}
// impl Default for AAConInfo {
//     fn default() -> Self {
//         Self {
//             addr: None,
//         }
//     }
// }

impl<Stream> axum::extract::connect_info::Connected<tokio_rustls::server::TlsStream<Stream>> for AAConInfo {
    fn connect_info(_target: tokio_rustls::server::TlsStream<Stream>) -> Self {
        t o d o!("axum::extract::connect_info::Connected<tokio_rustls::server::TlsStream<Stream>>")
    }
}
impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_>> for AAConInfo {
    fn connect_info(_target: axum::serve::IncomingStream<'_>) -> Self {
        t o d o!("axum::extract::connect_info::Connected<IncomingStream<'_>>")
        // AAConInfo::default()
    }
}

impl axum::extract::connect_info::Connected<std::net::SocketAddr> for AAConInfo {
    fn connect_info(target: std::net::SocketAddr) -> Self {
        // AAConInfo::default()
        // t o d o!("connect_info(target: std::net::SocketAddr)")
        AAConInfo {
            addr: Some(target),
        }
    }
}
*/

/*
pub fn axum_server_bind_rustls_22(addr: SocketAddr, config: axum_server::tls_rustls::RustlsConfig)
    // -> axum_server::Server<axum_server::tls_rustls::RustlsAcceptor> {
    // -> axum_server::Server<axum_server::tls_rustls::RustlsAcceptor<DefaultAcceptor22>> {
    -> axum_server::Server<MyRustlsAcceptor<DefaultAcceptor22>> {

    // use axum_server::tls_rustls::{RustlsConfig, RustlsAcceptor};
    use axum_server::Server;

    let acceptor: MyRustlsAcceptor<DefaultAcceptor22> = MyRustlsAcceptor::new(config)
        .acceptor::<DefaultAcceptor22>(DefaultAcceptor22);

    // origin
    // let aa: axum_server::Server<axum_server::tls_rustls::RustlsAcceptor> =
    //     Server::bind(addr).acceptor(acceptor)

    let aa: Server<MyRustlsAcceptor<DefaultAcceptor22>> =
        Server::bind(addr).acceptor(acceptor);

    aa
    // aa.serve()
    // aa.http_builder().serve_connection_with_upgrades()

    // t o d o!("dsdsd")
    //Server::<DefaultAcceptor22>::bind(addr).acceptor(acceptor)
}
*/

pub fn axum_server_bind_rustls_custom(
    addr: SocketAddr, config: axum_server::tls_rustls::RustlsConfig)
    -> axum_server::Server<MyRustlsAcceptor2<DefaultAcceptor22>> {

    // use axum_server::tls_rustls::{RustlsConfig, RustlsAcceptor};
    use axum_server::Server;

    let acceptor: MyRustlsAcceptor2<DefaultAcceptor22> = MyRustlsAcceptor2::new(config)
        .acceptor::<DefaultAcceptor22>(DefaultAcceptor22);

    let aa: Server<MyRustlsAcceptor2<DefaultAcceptor22>> =
        Server::bind(addr).acceptor(acceptor);
    aa
}


/// A no-op acceptor.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultAcceptor22;

impl DefaultAcceptor22 {
    /// Create a new default acceptor.
    pub fn new() -> Self {
        Self
    }
}

// pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;


impl<I, S> axum_server::accept::Accept<I, S> for DefaultAcceptor22
    where
        I: Send + Sync + 'static,
        S: Send + Sync + 'static,
    {
    type Stream = I;
    type Service = S;
    // type Future = core::future::Ready<io::Result<(Self::Stream, Self::Service)>>;
    type Future = BoxFuture<io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {

        /*
        Box::pin(ONE_BLA_BLA_2.scope(789, async move {
            let vv = ONE_BLA_BLA_2.get();
            println!("### ONE_BLA_BLA: {vv}");
            Ok((stream, service)) // TcpStream, too early
        })) // .boxed()
        */

        Box::pin(std::future::ready(Ok((stream, service))))
    }
}


/*
#[derive(Clone)]
pub struct MyRustlsAcceptor<A: Clone = axum_server::accept::DefaultAcceptor> {
    delegate: axum_server::tls_rustls::RustlsAcceptor<A>,
}

impl MyRustlsAcceptor {
    pub fn new(config: axum_server::tls_rustls::RustlsConfig) -> Self {
        Self {
            delegate: axum_server::tls_rustls::RustlsAcceptor::new(config),
        }
    }
    pub fn handshake_timeout(self, val: Duration) -> Self {
        Self {
            delegate: self.delegate.handshake_timeout(val),
        }
    }
}

impl<A: Clone> MyRustlsAcceptor<A> {
    /// Overwrite inner acceptor.
    pub fn acceptor<Acceptor: Clone>(self, acceptor: Acceptor) -> MyRustlsAcceptor<Acceptor> {
        MyRustlsAcceptor::<Acceptor> {
            delegate: self.delegate.acceptor(acceptor)
        }
    }
}


impl<A, I, S> axum_server::accept::Accept<I, S> for MyRustlsAcceptor<A>
where
    A: Clone + axum_server::accept::Accept<I, S>,
    A::Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    type Stream = tokio_rustls::server::TlsStream<A::Stream>;
    type Service = A::Service;
    // type Future = axum_server::tls_rustls::future::RustlsAcceptorFuture<A::Future, A::Stream, A::Service>;
    type Future = RustlsAcceptorFuture33<A::Future, A::Stream, A::Service>;
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

        let fut: RustlsAcceptorFuture33<A::Future, A::Stream, A::Service> =
            RustlsAcceptorFuture33::new(
                inner_future, config, self.delegate.handshake_timeout);
        fut
        // Box::pin(fut)
        // t o d o!("bvnbvbnvn")
    }
}

impl<A: Clone> fmt::Debug for MyRustlsAcceptor<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyRustlsAcceptor").finish()
    }
}
*/

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

/*
// pin_project_lite::pin_project! {
pub struct MyRustlsAcceptorFuture<F, I, S> {
    // #[pin]
    delegate: axum_server::tls_rustls::future::RustlsAcceptorFuture<F, I, S>,
    // config: Option<RustlsConfig>,
}
// }

impl<F, I, S> Future for MyRustlsAcceptorFuture<F, I, S>
where
    F: Future<Output = io::Result<(I, S)>>,
    I: AsyncRead + AsyncWrite + Unpin,
{
    type Output = <axum_server::tls_rustls::future::RustlsAcceptorFuture<F, I, S> as Future>::Output;
    // type Output = BoxFuture<io::Result<(I, S)>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Box::pin(self.delegate).po
        // self.delegate.poll(cx)
        // let delegate = self.delegate;
        // let mut pinned = std::pin::pin!(delegate);
        // pinned.as_mut().poll(cx)

        // let mut pinned = std::pin::pin!(self.delegate);
        // pinned.as_mut().poll(cx)

        // match Pin::new(&mut self.delegate).poll(cx) {
        // match self.delegate.poll(cx) {
        // let mut pinned = std::pin::pin!(self.delegate);
        // match pinned.as_mut().poll(cx) {
        //     // Poll::Ready(x) => Poll::Ready(x.map_err(|_| Error::msg("can't recv"))),
        //     Poll::Ready(x) => Poll::Ready(x),
        //     Poll::Pending => Poll::Pending,
        // }
    }
}
*/



pub async fn server_rustls_config<Conf: ServerConf>(server_conf: &Conf)
    -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {

    use axum_server::tls_rustls::RustlsConfig;

    let server_name = server_conf.server_name();
    let server_name = server_name.as_str();

    if let (Some(SslConfValue::Path(server_key)), Some(SslConfValue::Path(server_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        Ok(RustlsConfig::from_pem_file(server_cert, server_key).await ?)

    } else if let (Some(SslConfValue::Value(server_key)), Some(SslConfValue::Value(server_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        Ok(RustlsConfig::from_pem(
            Vec::from(server_cert.as_bytes()),
            Vec::from(server_key.as_bytes()),
        ).await ?)
    } else {
        anyhow::bail!("Both {server_name} cert/key should have the same type")
    }
}


// See
// * axum core: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
// * axum_server: ...
pub async fn axum_server_shutdown_signal(
    handle: axum_server::Handle, max_shutdown_duration: Duration) {

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("graceful_shutdown (ctrl_c)");
            handle.graceful_shutdown(Some(max_shutdown_duration))
        },
        _ = terminate => {
            info!("graceful_shutdown (terminate)");
            handle.graceful_shutdown(Some(max_shutdown_duration))
        },
    }
}


//---------------------------------
// TEMP

/*
// use crate::tls_rustls::RustlsConfig;
// use pin_project_lite::pin_project;
// use std::io::{Error, ErrorKind};
// use axum::serve::IncomingStream;
// use std::time::Duration;
// use tokio::time::{timeout, Timeout};
// use tokio_rustls::{server::TlsStream, Accept, TlsAcceptor};
// use tokio_rustls::{server::TlsStream};
// use tonic::IntoRequest;
// use tonic::transport::server::Connected;

pin_project_lite::pin_project! {
    /// Future type for [`RustlsAcceptor`](crate::tls_rustls::RustlsAcceptor).
    pub struct RustlsAcceptorFuture33<F, I, S> {
        #[pin]
        inner: AcceptFuture<F, I, S>,
        config: Option<axum_server::tls_rustls::RustlsConfig>,
    }
}

impl<F, I, S> RustlsAcceptorFuture33<F, I, S> {
    pub(crate) fn new(future: F, config: axum_server::tls_rustls::RustlsConfig, handshake_timeout: Duration) -> Self {
        let inner = AcceptFuture::Inner {
            future,
            handshake_timeout,
        };
        let config = Some(config);

        Self { inner, config }
    }
}

impl<F, I, S> fmt::Debug for RustlsAcceptorFuture33<F, I, S> {
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
            handshake_timeout: Duration,
        },
        Accept {
            #[pin]
            future: tokio::time::Timeout<tokio_rustls::Accept<I>>,
            service: Option<S>,
        },
    }
}

impl<F, I, S> Future for RustlsAcceptorFuture33<F, I, S>
where
    F: Future<Output = io::Result<(I, S)>>,
    I: AsyncRead + AsyncWrite + Unpin,
{
    type Output = io::Result<(tokio_rustls::server::TlsStream<I>, S)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        use std::io;

        let mut this = self.project();

        ONE_BLA_BLA_2.sync_scope(987, ||{

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
        })
    }
}
*/

/*
fn temp_333() {

    // let aa: tokio::net::tcp::TcpStream = unimplemented!();
    // watcher = {axum_server::handle::Watcher}
    // stream = {tokio_rustls::server::TlsStream<tokio::net::tcp::stream::TcpStream>}
    // let aa: tokio_rustls::server::TlsStream<tokio::net::tcp::TcpStream> = unimplemented!();
    let _aa: tokio_rustls::server::TlsStream<tokio::net::TcpStream> = unimplemented!();

    // let (stream, con) = _aa.into_inner();
    // let _certs = con.peer_certificates();


}
*/

/*
#[derive(Debug)]
struct MyAxumRouterIntoMakeService<S> {
    pub route: axum::Router<S>,
}
// impl<S> IntoMakeService<axum::Router<S>> for MyAxumRouterIntoMakeService<S> {
//
// }
impl<S, T> tower_service::Service<T> for MyAxumRouterIntoMakeService<S>
where
    S: Clone,
{
    // type Response = S;
    type Response = axum::Router<S>;
    type Error = core::convert::Infallible;
    type Future = MyAxumRouterIntoMakeServiceFuture2<S>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: T) -> Self::Future {
        // MyAxumRouterIntoMakeServiceFuture::new(core::future::ready(Ok(self.route.clone())))
        // MyAxumRouterIntoMakeServiceFuture2::new(self.route.with_state(()).clone())
        MyAxumRouterIntoMakeServiceFuture2::new(self.route.clone())
        // t o d o!("oueouoiuioexghfghf")
    }
}



#[allow(unused_macros)]
macro_rules! opaque_future {
    ($(#[$m:meta])* pub type $name:ident = $actual:ty;) => {
        opaque_future! {
            $(#[$m])*
            pub type $name<> = $actual;
        }
    };

    ($(#[$m:meta])* pub type $name:ident<$($param:ident),*> = $actual:ty;) => {
        pin_project_lite::pin_project! {
            $(#[$m])*
            pub struct $name<$($param),*> {
                #[pin] future: $actual,
            }
        }

        impl<$($param),*> $name<$($param),*> {
            pub(crate) fn new(future: $actual) -> Self {
                Self { future }
            }
        }

        impl<$($param),*> std::fmt::Debug for $name<$($param),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl<$($param),*> std::future::Future for $name<$($param),*>
        where
            $actual: std::future::Future,
        {
            type Output = <$actual as std::future::Future>::Output;

            #[inline]
            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                self.project().future.poll(cx)
            }
        }
    };
}
*/

/*
opaque_future! {
    /// Response future for [`IntoMakeService`].
    pub type MyAxumRouterIntoMakeServiceFuture<S> =
        std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>;
}
*/


// / Response future for [`IntoMakeService`].
// pub struct MyAxumRouterIntoMakeServiceFuture<S> {
//     future: std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>,
// }



/*
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    #[doc(hidden)]
    #[allow(dead_code)]
    #[allow(single_use_lifetimes)]
    #[allow(clippy::unknown_clippy_lints)]
    #[allow(clippy::mut_mut)]
    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::type_repetition_in_bounds)]
    pub(crate) struct Projection<'__pin, S>
    where
        MyAxumRouterIntoMakeServiceFuture<S>: '__pin,
    {
        future: ::pin_project_lite::__private::Pin<
            &'__pin mut (std::future::Ready<
                Result<axum::Router<S>, core::convert::Infallible>,
            >),
        >,
    }
    #[doc(hidden)]
    #[allow(dead_code)]
    #[allow(single_use_lifetimes)]
    #[allow(clippy::unknown_clippy_lints)]
    #[allow(clippy::mut_mut)]
    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::type_repetition_in_bounds)]
    pub(crate) struct ProjectionRef<'__pin, S>
    where
        MyAxumRouterIntoMakeServiceFuture<S>: '__pin,
    {
        future: ::pin_project_lite::__private::Pin<
            &'__pin (std::future::Ready<
                Result<axum::Router<S>, core::convert::Infallible>,
            >),
        >,
    }
    impl<S> MyAxumRouterIntoMakeServiceFuture<S> {
        #[doc(hidden)]
        #[inline]
        pub(crate) fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> Projection<'__pin, S> {
            unsafe {
                let Self { future } = self.get_unchecked_mut();
                Projection {
                    future: ::pin_project_lite::__private::Pin::new_unchecked(future),
                }
            }
        }
        #[doc(hidden)]
        #[inline]
        pub(crate) fn project_ref<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin Self>,
        ) -> ProjectionRef<'__pin, S> {
            unsafe {
                let Self { future } = self.get_ref();
                ProjectionRef {
                    future: ::pin_project_lite::__private::Pin::new_unchecked(future),
                }
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct __Origin<'__pin, S> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        future: std::future::Ready<
            Result<axum::Router<S>, core::convert::Infallible>,
        >,
    }
    impl<'__pin, S> ::pin_project_lite::__private::Unpin
    for MyAxumRouterIntoMakeServiceFuture<S>
    where
        __Origin<'__pin, S>: ::pin_project_lite::__private::Unpin,
    {}
    trait MustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project_lite::__private::Drop> MustNotImplDrop for T {}
    impl<S> MustNotImplDrop for MyAxumRouterIntoMakeServiceFuture<S> {}
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<S>(this: &MyAxumRouterIntoMakeServiceFuture<S>) {
        let _ = &this.future;
    }
};
*/

/*
pin_project_lite::pin_project! {
    // $(#[$m])*
    pub struct MyAxumRouterIntoMakeServiceFuture<S> {
        #[pin]
        future: std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>,
    }
}

impl<S> MyAxumRouterIntoMakeServiceFuture<S> {
    pub fn new(future: std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>)
        -> Self {
        Self { future }
    }
}
impl<S> fmt::Debug for MyAxumRouterIntoMakeServiceFuture<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyAxumRouterIntoMakeServiceFuture").finish_non_exhaustive()
    }
}
impl<S> Future for MyAxumRouterIntoMakeServiceFuture<S>
where
    std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>: Future,
{
    type Output = <std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>
        as Future>::Output;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

pin_project_lite::pin_project! {
    // $(#[$m])*
    pub struct MyAxumRouterIntoMakeServiceFuture2<S> {
        // #[pin]
        // future: std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>,
        route: axum::Router<S>,
    }
}

impl<S> MyAxumRouterIntoMakeServiceFuture2<S> {
    pub fn new(route: axum::Router<S>)
        -> Self {
        Self { route }
    }
}
impl<S> fmt::Debug for MyAxumRouterIntoMakeServiceFuture2<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyAxumRouterIntoMakeServiceFuture").finish_non_exhaustive()
    }
}
impl<S> Future for MyAxumRouterIntoMakeServiceFuture2<S>
where
    std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>: Future,
{
    type Output = Result<axum::Router<S>, core::convert::Infallible>;
    // type Output = <std::future::Ready<Result<axum::Router<S>, core::convert::Infallible>>
    //     as Future>::Output;
    #[inline]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // // let route = self.route.clone();
        // // let r: std::future::Ready::<Result<axum::Router<S>, core::convert::Infallible>> =
        // //     std::future::ready(Ok(self.route.clone()));
        let route = self.route.clone();
        let r3: Result<axum::Router<S>, core::convert::Infallible> = Ok(route);
        Poll::Ready(r3)
    }
}
*/

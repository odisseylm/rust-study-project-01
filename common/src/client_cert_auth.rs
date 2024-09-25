use std::{
    sync::Arc,
};
use anyhow::anyhow;
use itertools::{Either, Itertools};
use log::{error, warn};
use rustls::{
    DigitallySignedStruct, DistinguishedName,
    client::danger::HandshakeSignatureValid,
    server::danger::ClientCertVerified,
};
use rustls_pki_types::CertificateDer;
use x509_parser::nom::AsBytes;
use crate::{
    cfg::ServerConf,
    client_auth_cert_info::{ClientAuthCertInfo, extract_client_auth_cert_info_from_cert},
};
use crate::rustls_acceptor_with_con_info::PeerCertificates;
//--------------------------------------------------------------------------------------------------



pub async fn server_rustls_with_ssl_cert_client_auth_config<Conf: ServerConf>(server_conf: &Conf)
    -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {

    use std::sync::Arc;
    use axum_server::tls_rustls::RustlsConfig;

    let cert = server_conf.server_ssl_cert()
        .ok_or_else(||anyhow!("No server SSL cert (for SSL mode)")) ?
        .as_secure_string() ?;
    let key = server_conf.server_ssl_key()
        .ok_or_else(||anyhow!("No server SSL key (for SSL mode)")) ?
        .as_secure_string() ?;
    let client_auth_ca_cert = server_conf.client_auth_ssl_ca_cert()
        .ok_or_else(||anyhow!("No server SSL key (for SSL mode with client cert auth support)")) ?
        .as_secure_string() ?;

    let cert = Vec::<u8>::from(cert.as_ref().as_bytes());
    let key = Vec::<u8>::from(key.as_ref().as_bytes());
    let client_auth_ca_cert: Vec<u8> = Vec::<u8>::from(client_auth_ca_cert.as_ref().as_bytes());

    let serv_conf = config_from_pem(cert, key, client_auth_ca_cert) ?;
    let rust_tls_config = RustlsConfig::from_config(Arc::new(serv_conf));
    Ok(rust_tls_config)
}

//--------------------------------------------------------------------------------------------------
//
// Code copied from axum-server since I don't know how to add custom
// auth client cert verifier.
//
fn config_from_pem(cert: Vec<u8>, key: Vec<u8>, client_auth_ca_cert: Vec<u8>) -> std::io::Result<rustls::ServerConfig> {

    use rustls_pemfile::Item;

    let cert = rustls_pemfile::certs(&mut cert.as_ref())
        .map(|it| it.map(|it| it.to_vec()))
        .collect::<Result<Vec<_>, _>>()?;

    // Check the entire PEM file for the key in case it is not first section
    let mut key_vec: Vec<Vec<u8>> = rustls_pemfile::read_all(&mut key.as_ref())
        .filter_map(|i| match i.ok()? {
            Item::Sec1Key(key) => Some(key.secret_sec1_der().to_vec()),
            Item::Pkcs1Key(key) => Some(key.secret_pkcs1_der().to_vec()),
            Item::Pkcs8Key(key) => Some(key.secret_pkcs8_der().to_vec()),
            _ => None,
        })
        .collect();

    // Make sure file contains only one key
    if key_vec.len() != 1 {
        return Err(io_other("private key format not supported"));
    }

    let key: Vec<u8> = key_vec.pop()
        .ok_or_else(||io_other("No cert key")) ?;

    config_from_der(cert, key, client_auth_ca_cert)
}

fn io_other<E: Into<Box<dyn std::error::Error + Send + Sync>>>(error: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, error)
}


// Code copied and ADOPTED from axum-server
// since I don't know how to add custom auth client cert verifier.
//
// fn config_from_der(cert: Vec<Vec<u8>>, key: Vec<u8>, client_auth_ca_cert: &SslConfValue) -> std::io::Result<rustls::ServerConfig> {
fn config_from_der(cert: Vec<Vec<u8>>, key: Vec<u8>, client_auth_ca_cert: Vec<u8>)
    -> std::io::Result<rustls::ServerConfig> {

    let cert = cert.into_iter().map(CertificateDer::from).collect();
    let key = rustls_pki_types::PrivateKeyDer::try_from(key).map_err(io_other)?;

    use rustls::RootCertStore;
    use rustls::server::WebPkiClientVerifier;
    use std::sync::Arc;

    let mut root_s: RootCertStore = RootCertStore::empty();
    let ca_bytes = client_auth_ca_cert;
    let mut certs: &mut dyn std::io::BufRead = &mut ca_bytes.as_slice();

    for cert in rustls_pemfile::certs(&mut certs).collect::<Result<Vec<_>, _>>()? {
        root_s
            .add(cert)
            // .map_err(|_| TlsError::CertificateParseError)
            .map_err(|err|io_other(err))
            // .map_err(|_err|io_other("Error of adding client cert to roots."))
            ?;
    }

    let verifier = WebPkiClientVerifier::builder(Arc::new(root_s))
        .allow_unauthenticated() // T O D O: make configurable
        .build()
        // .map_err(|err| io_other("Error of creating WebPkiClientVerifier."));
        .map_err(|err| io_other(err))
        ?;

    let mut config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(MyClientCertVerifier { delegate: verifier }))
        .with_single_cert(cert, key)
        .map_err(io_other)?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(config)
}


pub fn get_grpc_current_client_auth_cert<T>(req: &tonic::Request<T>) -> anyhow::Result<Option<ClientAuthCertInfo>> {
    let ext = req.extensions();

    let tonic_certs = req.peer_certs();
    if let Some(tonic_certs) = tonic_certs {
        return get_current_client_auth_cert(tonic_certs.as_ref());
    }

    ext.get::<PeerCertificates>()
        .map(|peer_certs|get_current_client_auth_cert(&peer_certs.certs))
        .unwrap_or(Ok(None))
}

pub fn get_http_current_client_auth_cert_from_req<T>(req: &http::Request<T>) -> anyhow::Result<Option<ClientAuthCertInfo>> {
    let ext = req.extensions();

    if cfg!(feature = "tonic") {
        let tonic_certs = ext.get::<tonic::transport::server::TlsConnectInfo<tonic::transport::server::TcpConnectInfo>>()
            .and_then(|i| i.peer_certs())
            .map(|peer_certs| get_current_client_auth_cert(&peer_certs.as_ref()))
            .unwrap_or(Ok(None)) ?;
        if let Some(ref _certs) = tonic_certs {
            return Ok(tonic_certs);
        }
    }

    ext.get::<PeerCertificates>()
        .map(|peer_certs|get_current_client_auth_cert(&peer_certs.certs))
        .unwrap_or(Ok(None))
}

pub fn get_http_current_client_auth_cert_from_req_parts(req: &http::request::Parts) -> anyhow::Result<Option<ClientAuthCertInfo>> {
    let ext = &req.extensions;
    ext.get::<PeerCertificates>()
        .map(|peer_certs|get_current_client_auth_cert(&peer_certs.certs))
        .unwrap_or(Ok(None))
}

pub fn get_current_client_auth_cert(peer_certs: &Vec<CertificateDer<'static>>) -> anyhow::Result<Option<ClientAuthCertInfo>> {

    let (parsed_certs, mut failed): (Vec<ClientAuthCertInfo>, Vec<_>) = peer_certs.iter()
        .map(|cert| extract_client_auth_cert_info_from_cert(cert.as_bytes()))
        .partition_map(|el| match el {
            Ok(val) => Either::Left(val),
            Err(err) => Either::Right(err),
        });

    if let Some(err) = failed.pop() {
        if parsed_certs.is_empty() {
            return Err(anyhow!("Client certificate process error (all certs failed) ({err:?})."));
        }
    }

    let mut all_client_auth_certs = parsed_certs.into_iter()
        .filter(|cert| cert.is_client_auth_key_usage)
        .collect::<Vec<_>>();

    let all_client_auth_certs_count = all_client_auth_certs.len();
    if all_client_auth_certs_count > 1 {
        // Probably some additional filtering should be used if we have such situation.
        let cert_names = all_client_auth_certs.iter()
            .map(|cert|&cert.common_name)
            .join(", ");
        warn!("There are several ({all_client_auth_certs_count}) client auth certs [{cert_names}]. Last one will be used.");
    }

    Ok(all_client_auth_certs.pop())
}

#[derive(Debug)]
struct MyClientCertVerifier {
    delegate: Arc<dyn rustls::server::danger::ClientCertVerifier>,
}

impl rustls::server::danger::ClientCertVerifier for MyClientCertVerifier {
    fn offer_client_auth(&self) -> bool {
        self.delegate.offer_client_auth()
    }

    fn client_auth_mandatory(&self) -> bool {
        self.delegate.client_auth_mandatory()
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        self.delegate.root_hint_subjects()
    }


    fn verify_client_cert(&self,
                          end_entity: &CertificateDer<'_>,
                          intermediates: &[CertificateDer<'_>],
                          now: rustls_pki_types::UnixTime)
        -> Result<ClientCertVerified, rustls::Error> {

        let res = self.delegate.verify_client_cert(end_entity, intermediates, now);
        match res {
            Ok(ref _client_cert_verified) => {
                /*
                info!("Client cert verification succeeded");
                let der_bytes = end_entity.as_ref();

                let cert_info = extract_client_auth_cert_info_from_cert(der_bytes);
                match cert_info {
                    Ok(client_auth_cert_info) => {
                        info!("cert_info: {client_auth_cert_info:?}");
                        // T O D O: put client_auth_cert_info to request context
                        // ONE_BLA_BLA = LocalKey::new() 567;
                        // ONE_BLA_BLA.with(|_arg|456);
                        // ONE_BLA_BLA.scope(1, async move {
                        //     println!("task local value: {}", ONE_BLA_BLA. get());
                        // }).await;

                        // tokio::task::futures::TaskLocalFuture

                        // ONE_BLA_BLA.get();
                        // let bla_bla_2 = ONE_BLA_BLA_2.get();
                        // println!("### bla_bla_2: {bla_bla_2}");

                        // ONE_BLA_BLA.sync_scope(1, move ||{
                        //         println!("task local value: {}", ONE_BLA_BLA. get());
                        //     });

                    }
                    Err(err) => {
                        error!("Client verification succeeded, but extracting cert info failed ({err:?})");
                        // return Err(rustls::Error::Other(rustls::OtherError(Arc::new(err)))) // T O D O: Why?
                        return Err(rustls::Error::General(err.to_string()))
                    }
                }
                */
            }
            Err(ref err) => {
                error!("Client cert verification failed: {err:?}");
            }
        }
        res
    }

    fn verify_tls12_signature(&self,
                              message: &[u8],
                              cert: &CertificateDer<'_>,
                              dss: &DigitallySignedStruct)
                              -> Result<HandshakeSignatureValid, rustls::Error> {
        self.delegate.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(&self,
                              message: &[u8],
                              cert: &CertificateDer<'_>,
                              dss: &DigitallySignedStruct)
                              -> Result<HandshakeSignatureValid, rustls::Error> {
        self.delegate.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.delegate.supported_verify_schemes()
    }
}


//--------------------------------------------------------------------------------------------------

// use tonic::{
//     // Request,
//     transport::server::Connected,
// };
// use x509_parser::nom::AsBytes;
// use crate::client_auth_cert_info::ClientAuthCertInfo;
// use crate::rustls_acceptor_2::{ConnectionInfo, ConnectionStreamExtensions, PeerCertificates};

/*
// A `Stream` that yields connections
struct MyConnector {}

// Return metadata about the connection as `MyConnectInfo`
impl Connected for MyConnector {
    type ConnectInfo = MyConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        MyConnectInfo {}
    }
}

#[derive(Clone)]
struct MyConnectInfo {
    // Metadata about your connection
}
*/
/*
// The connect info can be accessed through request extensions:
let connect_info: &MyConnectInfo = request
    .extensions()
    .get::<MyConnectInfo>()
    .expect("bug in tonic");
*/

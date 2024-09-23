use std::{
    sync::Arc,
};
use anyhow::anyhow;
use log::{error};
use rustls::{
    DigitallySignedStruct, DistinguishedName,
    client::danger::HandshakeSignatureValid,
    server::danger::ClientCertVerified,
};
use rustls_pki_types::CertificateDer;
use x509_parser::nom::AsBytes;
use crate::{
    cfg::ServerConf, //client_auth_cert_info,
    client_auth_cert_info::{ClientAuthCertInfo, extract_client_auth_cert_info_from_cert},
    rustls_acceptor_with_con_info::ConnectionInfo,
};
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

    let mut serv_conf = config_from_pem(cert, key, client_auth_ca_cert) ?;
    serv_conf.enable_secret_extraction = true; // TODO: Do we need it?
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
    //
    // let client_auth_cert = client_auth_ca_cert.as_secure_string() ?;
    // let client_auth_cert = match client_auth_cert {
    //     Cow::Borrowed(secure_str_ref) =>
    //         Vec::from(secure_str_ref.as_bytes()),
    //     Cow::Owned(secure_str) =>
    //         secure_str.into_bytes(),
    // };
    // let client_auth_cert = client_auth_cert.as_bytes();

    let ca_bytes = client_auth_ca_cert;
    // let ca = ca_bytes.into_iter().map(rustls::pki_types::CertificateDer::from).collect();
    // let ca = rustls::pki_types::CertificateDer::from(ca_bytes);

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


// tokio::task_local! {
//     pub static ONE_BLA_BLA: u32;
// }
// tokio_inherit_task_local::inheritable_task_local! {
//     pub static ONE_BLA_BLA_2: u32;
// }

pub fn get_current_client_auth_cert() -> anyhow::Result<Option<ClientAuthCertInfo>> {
    // TODO: temp, move it to 'enrich request' interceptor
    //       and take there ConnectionInfo from request extensions (not from task-local var)
    let ext = crate::rustls_acceptor_with_con_info::CONNECTION_STREAM_EXTENSION
        .try_with(|v| v.clone());

    let ext = match ext {
        Ok(ext) => ext,
        Err(ref _err) =>
            // Or we can throw exception and force developer not oto use it
            // if server is not configured properly
            // return Ok(None),

            anyhow::bail!("Seems server is not configured to capture SSL/TLS stream info.")
    };

    // TODO: Use also tonic approach

    let con_info = ext.get::<ConnectionInfo>();
    println!("### con_info: ${con_info:?}");

    match con_info {
        None => Ok(None),
        Some(con_info) => {
            match con_info.peer_certs {
                None => Ok(None),
                Some(ref peer_certs) => {
                    // TODO: use filter/validate/find authClient certificate
                    let auth_client_cert = peer_certs.certs.first()
                        // Or we can return None... but I think if we are there
                        // it is unexpected situation, and it is error/bug
                        .ok_or_else(||anyhow!("No auth cert")) ?;
                    let client_auth_cert_info = extract_client_auth_cert_info_from_cert(
                        auth_client_cert.as_bytes()) ?;
                    Ok(Some(client_auth_cert_info))
                }
            }
        }
    }
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

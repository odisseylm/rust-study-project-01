use anyhow::anyhow;
use log::debug;
use x509_parser::{
    der_parser::asn1_rs::Oid,
    prelude::X509Name,
    der_parser::asn1_rs::Any,
};
//--------------------------------------------------------------------------------------------------


// It contains only currently needed client auth cert attributes.
// Feel free to extend it.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ClientAuthCertInfo {
    // The common name attribute type specifies an identifier of an object.
    // Oid CN/2.5.4.3  https://www.alvestrand.no/objectid/2.5.4.3.html
    pub common_name: String,

    // The Organization Name attribute type specifies an organizatio.
    // Oid  O/2.5.4.10
    pub organization: String,

    /// Email Address attribute for use in signatures.
    /// Oid 1.2.840.113549.1.9.1  OID_PKCS9_EMAIL
    //
    // SmallVec is used there as experiment.
    pub pkcs9_emails: smallvec::SmallVec<[String; 5]>,

    /// Oid 2.5.29.17 - Subject Alternative Name
    /// https://www.alvestrand.no/objectid/2.5.29.17.html
    //
    // SmallVec is used there as experiment.
    pub alt_name_ext_emails: smallvec::SmallVec<[String; 5]>,

    // ExtendedKeyUsage extension, attribute 'clientAuth'
    // https://www.alvestrand.no/objectid/2.5.29.37.html
    pub is_client_auth_key_usage: bool,

    #[doc(hidden)]
    __non_exhaustive: (),
}


pub fn extract_client_auth_cert_info_from_cert(cert_bytes: &[u8])
    -> anyhow::Result<ClientAuthCertInfo> {

    use x509_parser::der_parser::asn1_rs::Oid;
    use x509_parser::der_parser::asn1_rs::oid;
    use x509_parser::extensions::GeneralName;

    let (_bytes, ref cert) = x509_parser::parse_x509_certificate(cert_bytes) ?;

    let issuer = &cert.issuer;
    debug!("Auth client cert issuer: {issuer}");

    let issuer_uid = &cert.issuer_uid;
    debug!("Auth client cert issuer UID: {issuer_uid:?}");

    let subject = &cert.subject;

    const CN_OID: Oid<'static> = oid!(2.5.4.3);
    let common_name = get_single_attr_value(subject, &CN_OID) ?;

    const O_OID: Oid<'static> = oid!(2.5.4.10);
    let organization = get_single_attr_value(subject, &O_OID) ?;

    let pkcs9_email_attrs = subject.iter_email().collect::<Vec<_>>();
    let pkcs9_emails = pkcs9_email_attrs.into_iter()
        .map(|p| attribute_value_to_string(p.attr_value(), p.attr_type()))
        .collect::<anyhow::Result<smallvec::SmallVec<[String; 5]>>>() ?;

    let alt_names = &cert.subject_alternative_name() ?;
    let alt_name_ext_emails =
        if let Some(alt_names) = alt_names.as_ref() {
            alt_names.value.general_names.iter()
                .filter_map(|general_name|{
                    if let GeneralName::RFC822Name(email) = general_name {
                        Some(email.to_string())
                    } else {
                        None
                    }
                })
                .collect::<smallvec::SmallVec<[String; 5]>>()
        } else {
            smallvec::SmallVec::<[String; 5]>::new()
        };

    let ex_key_usage_ext = &cert.extended_key_usage() ?;
    let is_client_auth_key_usage = ex_key_usage_ext.as_ref().map(|ex_key_usage_ext| ex_key_usage_ext.value.client_auth)
        .unwrap_or(false);

    Ok(ClientAuthCertInfo {
        common_name,
        organization,
        pkcs9_emails,
        alt_name_ext_emails,
        is_client_auth_key_usage,
        __non_exhaustive: (),
    })
}


fn get_single_attr_value(subject: &X509Name, oid: &Oid) -> anyhow::Result<String> {

    let attrs = subject.iter_by_oid(&oid).collect::<Vec<_>>();
    let attrs_count = attrs.len();

    if attrs_count != 1 {
        anyhow::bail!("There are {attrs_count} [{}] subject attrs, expected one.",
            get_oid_debug_name(oid));
    }
    let cn_attr = attrs.first()
        .ok_or_else(||anyhow!("There are no [{}] subject attrs.", get_oid_debug_name(oid))) ?;

    let value = attribute_value_to_string(cn_attr.attr_value(), &oid) ?;
    Ok(value)
}


fn get_oid_debug_name(oid: &Oid) -> String {
    use x509_parser::prelude::{oid2abbrev, oid_registry};

    let oid_as_numbers_str = oid.to_string();
    let oid_name = oid2abbrev(oid, oid_registry());
    match oid_name {
        Ok(str) => format!("{oid_as_numbers_str}/{str}"),
        Err(_err) => oid_as_numbers_str,
    }
}



// Code copied from x509-parser crate.
//
fn attribute_value_to_string(attr: &Any, _attr_type: &Oid)
    -> anyhow::Result<String> { // X509Error

    use x509_parser::{error::X509Error, der_parser::asn1_rs::{BmpString, Tag}};

    // T O D O: replace this with helper function, when it is added to asn1-rs
    match attr.tag() {
        Tag::NumericString
        | Tag::VisibleString
        | Tag::PrintableString
        | Tag::GeneralString
        | Tag::ObjectDescriptor
        | Tag::GraphicString
        | Tag::T61String
        | Tag::VideotexString
        | Tag::Utf8String
        | Tag::Ia5String => {
            let s = core::str::from_utf8(attr.data).map_err(|_| X509Error::InvalidAttributes)?;
            Ok(s.to_owned())
        }
        Tag::BmpString => {
            // T O D O: remove this when a new release of asn1-rs removes the need to consume attr in try_from
            let any = attr.clone();
            let s = BmpString::try_from(any).map_err(|_| X509Error::InvalidAttributes)?;
            Ok(s.string())
        }
        _ => {
            // type is not a string, get slice and convert it to base64
            // Ok(HEXUPPER.encode(attr.as_bytes()))  // At that moment we do not need it.
            //
            Ok("...".to_owned())
        }
    }
}


/*
 https://javadoc.sic.tech/iaik_jce/old/iaik/x509/extensions/ExtendedKeyUsage.html

 The following extended key usage purposes are defined by RFC 3280:

    serverAuth (1.3.6.1.5.5.7.3.1) -- TLS Web server authentication
    clientAuth (1.3.6.1.5.5.7.3.2) -- TLS Web client authentication
    codeSigning (1.3.6.1.5.5.7.3.3) -- Code signing
    emailProtection (1.3.6.1.5.5.7.3.4) -- E-mail protection
    timeStamping (1.3.6.1.5.5.7.3.8) -- Timestamping
    ocspSigning (1.3.6.1.5.5.7.3.9) -- OCSPstamping

The following purposes have been included in a predecessor draft of RFC 3280 and therefore continue to be registrated by this implementation:

    ipsecEndSystem (1.3.6.1.5.5.7.3.5) -- IP security end system
    ipsecTunnel (1.3.6.1.5.5.7.3.6) -- IP security tunnel termination
    ipsecUser (1.3.6.1.5.5.7.3.7) -- IP security user
*/

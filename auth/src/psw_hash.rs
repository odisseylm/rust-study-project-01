use std::borrow::Cow;
use log::{error, warn};
use mvv_common::{backtrace::{backtrace, BacktraceCell}, generate_display_delegate, string::StaticRefOrString};
use crate::{PasswordComparator, SecureString};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, Copy, Clone)]
#[derive(strum::Display, strum::EnumString)]
pub enum SaltRndGenType {
    Default,
    Hc128Rng,
    ChaCha,
    RandThreadRng,
}


#[derive(Debug)]
pub struct PswHashConfig {
    pub algorithm: String, // algorithm name is case-sensitive
    pub version: Option<password_hash::Decimal>,
    pub salt_rnd_gen_type: Option<SaltRndGenType>,
    pub salt: Option<password_hash::SaltString>,
}


#[derive(
    Debug,
    thiserror::Error,
    mvv_error_macro::ThisErrorFromWithBacktrace,
    mvv_error_macro::ThisErrorBacktraceSource,
)]
pub enum PswHashError {
    #[error("PasswordHashError {{ {0} }}")]
    PasswordHashError(#[source] #[from_with_bt] password_hash::Error, BacktraceCell),
    #[error("PasswordHashError, no algorithm [0]")]
    NoAlgorithm(StaticRefOrString, BacktraceCell),
    #[error("ConfigError {{ {0} }}")]
    ConfigError(StaticRefOrString, BacktraceCell),
    #[error("NotInitialized {{ {0} }}")]
    NotInitialized(StaticRefOrString, BacktraceCell),
    #[error("PasswordHashFormatError")]
    PasswordHashFormatError(BacktraceCell),

    // Use it to verify that password is incorrect.
    #[error("InvalidPassword")]
    InvalidPassword, // Actually it is not real error and backtrace is not needed.
}


impl PswHashConfig {
    pub fn load_from_env(prefix: &str) -> Result<Self, PswHashError> {
        use password_hash::Decimal;
        use core::str::FromStr;

        let alg_name_env_name = format!("{prefix}PSW_HASH_ALG");
        let alg_ver_env_name = format!("{prefix}PSW_HASH_ALG_VER");
        let salt_env_name = format!("{prefix}PSW_HASH_SALT");
        let salt_rnd_gen_type_env_name = format!("{prefix}PSW_HASH_SALT_GEN");

        fn no_env_var_err(err: mvv_common::env::EnvVarError) -> PswHashError {
            PswHashError::ConfigError(
                format!("No/broken env var [{}]", err.var_name).into(), backtrace())
        }

        let alg_name = mvv_common::env::required_env_var_2(&alg_name_env_name)
            .map_err(no_env_var_err) ?;
        let alg_ver_opt = mvv_common::env::env_var_2(&alg_ver_env_name)
            .map_err(no_env_var_err) ?;
        let salt_b64 = mvv_common::env::required_env_var_2(&salt_env_name)
            .map_err(no_env_var_err) ?;

        let salt_rnd_gen_type: Option<String> = mvv_common::env::env_var_2(&salt_rnd_gen_type_env_name)
            .map_err(no_env_var_err) ?;
        let salt_rnd_gen_type: Option<SaltRndGenType> = match salt_rnd_gen_type {
            None => None,
            Some(ref s) => {
                Some(SaltRndGenType::try_from(s.as_str())
                         .map_err(|_|PswHashError::ConfigError(
                             format!("No/broken env var [{salt_rnd_gen_type_env_name}]").into(),
                             backtrace(),
                         ))
                ?)
            }
        };

        let alg_ver_opt = match alg_ver_opt {
            None => None,
            Some(ref alg_ver) => {
                let alg_ver =
                    if alg_ver.starts_with("0x") {
                        let alg_ver = alg_ver.strip_prefix("0x").unwrap_or(alg_ver);
                        Decimal::from_str_radix(alg_ver, 16)
                    } else {
                        Decimal::from_str(alg_ver)
                    };

                let alg_ver = alg_ver.map_err(|_|PswHashError::ConfigError(
                    format!("Env var [{alg_ver_env_name}] has incorrect format.").into(), backtrace())) ?;
                Some(alg_ver)
            }
        };

        Ok(PswHashConfig {
            algorithm: alg_name.to_owned(),
            version: alg_ver_opt,
            salt_rnd_gen_type,
            salt: Some(password_hash::SaltString::from_b64(&salt_b64) ?),
        })
    }
}



// It is just aliases. You can use original ones from argon, pbkdf2, scrypt if you want,
#[allow(dead_code)]
pub mod algorithm {
    use password_hash::{Decimal, Ident};

    // ----------------------------------------------------------------------------
    //                                   Argon
    //
    pub const ARGON2D_ALG: Ident<'static> = argon2::ARGON2D_IDENT;
    pub const ARGON2I_ALG: Ident<'static> = argon2::ARGON2I_IDENT;
    pub const ARGON2ID_ALG: Ident<'static> = argon2::ARGON2ID_IDENT;
    //
    #[allow(non_upper_case_globals)]
    pub const ARGON2_VER_V0x10: Decimal = argon2::Version::V0x10 as u32;
    #[allow(non_upper_case_globals)]
    pub const ARGON2_VER_V0x13: Decimal = argon2::Version::V0x13 as u32;

    // ----------------------------------------------------------------------------
    //                                   pbkdf2
    //
    // See sources pbkdf2-0.12.2/src/simple.rs
    pub const PBKDF2_SHA256_ALG: Ident<'static> = pbkdf2::Algorithm::PBKDF2_SHA256_IDENT; // Ident::new_unwrap("pbkdf2-sha256"); // PBKDF2_SHA256_IDENT;
    pub const PBKDF2_SHA512_ALG: Ident<'static> = pbkdf2::Algorithm::PBKDF2_SHA512_IDENT; // Ident::new_unwrap("pbkdf2-sha256"); // PBKDF2_SHA256_IDENT;
    //
    // PBKDF2 has no versions (at least now)

    // ----------------------------------------------------------------------------
    //                                   Scrypt
    //
    pub const SCRYPT_ALG: Ident<'static> = scrypt::ALG_ID;
    //
    // Scrypt has no versions (at least now)
}


fn is_argon_alg(alg: &str) -> bool {
    alg.starts_with("argon")
}
fn is_pbkdf2_alg(alg: &str) -> bool {
    alg.starts_with("pbkdf2")
}
fn is_scrypt_alg(alg: &str) -> bool {
    alg.starts_with("scrypt")
}


/// !!! Unstable design !!!
///
/// Due to design of 'password_hash' crate we cannot just return structure with password_hash objects
/// (at least without usage Pin and heap allocation)
#[derive(Debug, PartialEq, Eq)]
pub struct PswHash(pub String);
generate_display_delegate! { PswHash }


/// Due to design of 'password_hash' crate we cannot just return structure with password_hash objects
/// (at least without usage Pin and heap allocation)
pub fn hash_psw_str(cfg: &PswHashConfig, plain_psw: &SecureString)
    -> Result<String, PswHashError> {
    hash_psw_str_using_rnd_gen(cfg, plain_psw, SaltRndGenType::Default)
}

/// Due to design of 'password_hash' crate we cannot just return structure with password_hash objects
/// (at least without usage Pin and heap allocation)
pub fn hash_psw_str_using_rnd_gen(
    cfg: &PswHashConfig, plain_psw: &SecureString, salt_rnd_gen_type: SaltRndGenType)
    -> Result<String, PswHashError> {

    let salt = generate_salt_using_rnd_gen(salt_rnd_gen_type) ?;
    let hash = hash_psw_with_salt(cfg, plain_psw, salt.as_salt()) ?;
    Ok(hash.to_string())
}


pub fn hash_psw_with_salt<'a>(cfg: &'a PswHashConfig, plain_psw: &'a SecureString, salt: password_hash::Salt<'a>)
    -> Result<password_hash::PasswordHash<'a>, PswHashError> {
    use password_hash::{ Ident, PasswordHasher };

    let alg_name = cfg.algorithm.as_str();
    let psw_bytes = plain_psw.as_bytes();
    let alg_param = Some(Ident::new(cfg.algorithm.as_str()) ?);
    let ver = cfg.version;

    match alg_name { // algorithm name is case-sensitive
        alg if is_argon_alg(alg) => {
            let psw_hash = argon2::Argon2::default()
                .hash_password_customized(
                    psw_bytes, alg_param, ver, argon2::Params::default(), salt) ?;
            Ok(psw_hash)
        }
        alg if is_pbkdf2_alg(alg) => {
            let psw_hash = pbkdf2::Pbkdf2.hash_password_customized(
                psw_bytes, alg_param, ver, pbkdf2::Params::default(), salt) ?;
            Ok(psw_hash)
        }
        alg if is_scrypt_alg(alg) => {
            let psw_hash = scrypt::Scrypt.hash_password_customized(
                psw_bytes, alg_param, ver, scrypt::Params::default(), salt) ?;
            Ok(psw_hash)
        }
        _ =>
            Err(PswHashError::NoAlgorithm(alg_name.to_string().into(), backtrace())),
    }
}


pub fn verify_psw_by_hash_str(
    plain_psw_bytes: &[u8],
    psw_hash: &str,
) -> Result<(), PswHashError> {
    let psw_hash = password_hash::PasswordHash::try_from(psw_hash)
        .map_err(|_| PswHashError::PasswordHashFormatError(backtrace())) ?;
    verify_psw(plain_psw_bytes, &psw_hash)
}


pub fn verify_psw(
    plain_psw_bytes: &[u8],
    psw_hash: &password_hash::PasswordHash<'_>,
) -> Result<(), PswHashError> {

    use password_hash::PasswordVerifier;

    let alg_name = psw_hash.algorithm.as_str();
    let psw_bytes = plain_psw_bytes; // plain_psw.as_bytes();

    let verify_res = match alg_name { // algorithm name is case-sensitive
        alg if is_argon_alg(alg) => {
            argon2::Argon2::default()
                .verify_password(psw_bytes, psw_hash)
        }
        alg if is_pbkdf2_alg(alg) => {
            pbkdf2::Pbkdf2.verify_password(psw_bytes, psw_hash)
        }
        alg if is_scrypt_alg(alg) => {
            scrypt::Scrypt.verify_password(psw_bytes, psw_hash)
        }
        _ =>
            return Err(PswHashError::NoAlgorithm(alg_name.to_string().into(), backtrace())),
    };

    match verify_res {
        Ok(_) => Ok(()),
        Err(password_hash::Error::Password) =>
            Err(PswHashError::InvalidPassword),
        Err(err) =>
            Err(PswHashError::PasswordHashError(err, backtrace())),
    }
}


#[inline]
pub fn generate_salt() -> Result<password_hash::SaltString, PswHashError> {
    generate_salt_using_rnd_gen(SaltRndGenType::Default)
}


pub fn generate_salt_using_rnd_gen(crypto_rnd_gen: SaltRndGenType) -> Result<password_hash::SaltString, PswHashError> {
    use rand::SeedableRng;
    use password_hash::SaltString;

    let salt = match crypto_rnd_gen {
        SaltRndGenType::Default =>
            SaltString::generate(rand_hc::Hc128Rng::from_entropy()),
        SaltRndGenType::Hc128Rng =>
            SaltString::generate(rand_hc::Hc128Rng::from_entropy()),
        SaltRndGenType::ChaCha =>
            SaltString::generate(rand_chacha::ChaCha20Rng::from_entropy()),
        SaltRndGenType::RandThreadRng =>
            SaltString::generate(rand::thread_rng()),
    };

    Ok(salt)
}

#[derive(Debug, Clone)]
pub struct Pepper {
    #[allow(dead_code)]
    value: SecureString,
}


pub struct PswHashComparator {
    // https://stackoverflow.com/questions/16891729/best-practices-salting-peppering-passwords
    #[allow(dead_code)]
    pepper: Option<Pepper>,
}
impl PswHashComparator {
    pub fn new() -> Self {
        PswHashComparator { pepper: None }
    }
    fn apply_pepper<'a>(&self, user_password: &'a str) -> Cow<'a, str> {
        // T O DO : use pepper if you think it is really needed
        // but read please this https://stackoverflow.com/questions/16891729/best-practices-salting-peppering-passwords
        Cow::Borrowed(user_password)
    }
}
impl PasswordComparator for PswHashComparator {
    fn passwords_equal(&self, user_psw_or_psw_hash: &str, credentials_password: &str) -> bool {

        let credentials_password = self.apply_pepper(credentials_password);
        let credentials_password = credentials_password.as_ref();

        let verify_res = verify_psw_by_hash_str(credentials_password.as_bytes(), user_psw_or_psw_hash);
        match verify_res {
            Ok( () ) => true,
            Err(err) => match err {
                PswHashError::InvalidPassword =>
                    false,
                err => {
                    warn!("Error password hash validation: {err:?}");
                    false
                }
            }
        }
    }
}

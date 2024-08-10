use log::{error, warn};
use mvv_common::{
    backtrace::{backtrace, BacktraceCell},
    string::StaticRefOrString,
};
use crate::{PasswordComparator, SecureString};
//--------------------------------------------------------------------------------------------------


#[derive(Debug)]
pub struct PswHashConfig {
    pub algorithm: String, // algorithm name is case-sensitive
    pub version: Option<password_hash::Decimal>,
    pub salt: password_hash::SaltString,
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

        fn no_env_var_err(err: mvv_common::env::EnvVarError) -> PswHashError {
            PswHashError::ConfigError(
                format!("No/broken env var [{}]", err.var_name).into(), backtrace())
        }

        let alg_name = mvv_common::env::required_env_var_2(&alg_name_env_name)
            .map_err(no_env_var_err) ?;
        let alg_ver = mvv_common::env::env_var_2(&alg_ver_env_name)
            .map_err(no_env_var_err) ?;
        let salt_b64 = mvv_common::env::required_env_var_2(&salt_env_name)
            .map_err(no_env_var_err) ?;

        let alg_ver = match alg_ver {
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
            version: alg_ver,
            salt: password_hash::SaltString::from_b64(&salt_b64) ?,
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

pub fn hash_password<'a>(cfg: &'a PswHashConfig, plain_psw: &'a SecureString)
    -> Result<password_hash::PasswordHash<'a>, PswHashError> {
    use password_hash::{ Ident, PasswordHasher };

    let alg_name = cfg.algorithm.as_str();
    let psw_bytes = plain_psw.as_bytes();
    let alg_param = Some(Ident::new(cfg.algorithm.as_str()) ?);
    let ver = cfg.version;
    let salt = cfg.salt.as_salt();

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


pub fn verify_password<'a>(
    // plain_psw: &'a SecureString,
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


pub fn generate_salt() -> Result<password_hash::SaltString, PswHashError> {
    use rand::SeedableRng;

    let rng = rand_hc::Hc128Rng::from_entropy();
    // let rng = rand_chacha::ChaCha20Rng::seed_from_u64(0u64);
    // let rng = rand_chacha::ChaCha20Rng::from_entropy();
    // let mut rng = rand::thread_rng();

    Ok(password_hash::SaltString::generate(rng))
}



pub struct PswHashComparator;
impl PasswordComparator for PswHashComparator {
    fn passwords_equal(user_password: &str, credentials_password: &str) -> bool {
        let psw_hash = password_hash::PasswordHash::new(user_password);
        match psw_hash {
            Err(ref err) => {
                error!("Error of parsing password hash. {err:?}");
                false
            }
            Ok(psw_hash) => {
                let verify_res = verify_password(
                    credentials_password.as_bytes(), &psw_hash);
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
    }
}

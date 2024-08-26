use password_hash::{Ident, SaltString};
use tokio_test::{assert_err, assert_ok};
// use mvv_auth::psw_hash::{generate_salt, PswHashAndSalt, SaltRndGenType, verify_password};
use mvv_auth::psw_hash::{generate_salt, hash_psw_str, hash_psw_with_salt, PswHashConfig, SaltRndGenType, verify_psw, verify_psw_by_hash_str};
use mvv_auth::psw_hash::algorithm::{ARGON2_VER_V0x10, ARGON2D_ALG};
use mvv_auth::SecureString;
use mvv_common::test::{TestDisplayStringOps, TestOptionUnwrap, TestResultUnwrap};
//--------------------------------------------------------------------------------------------------



#[test]
fn test_hasher_1() {
    use password_hash::{ PasswordHasher };

    let hasher = argon2::Argon2::new( // argon2::Argon2::default()
        argon2::Algorithm::Argon2i, argon2::Version::V0x10, argon2::Params::default());

    let salt_str = SaltString::encode_b64(b"12345678901234567890").test_unwrap();
    // password_hash::Salt::from()

    let psw_hash = hasher.hash_password(
        b"qwerty",
        // "salt".as_bytes(),
        salt_str.as_salt(),
    ).test_unwrap();

    println!("### psw_hash: {psw_hash}");
    assert_eq!(
        psw_hash.to_test_string(),
        "$argon2i$v=16$m=19456,t=2,p=1$MTIzNDU2Nzg5MDEyMzQ1Njc4OTA$4l4WGDDW0Y9YZp72yyYABS2t3Wo5H40tjQtgGi0wdzw",
    );

    // hasher.hash_password()

    // assert!(false, "Test error to see console output");
}


#[test]
fn test_hasher_2() {
    use password_hash::{ PasswordHasher };

    let hasher = argon2::Argon2::default();
    // let hasher = argon2::Argon2::new( // argon2::Argon2::default()
    //     argon2::Algorithm::Argon2i, argon2::Version::V0x10, argon2::Params::default());

    let salt_str = SaltString::encode_b64(b"12345678901234567890").test_unwrap();
    // password_hash::Salt::from()

    let psw_hash = hasher.hash_password_customized(
        b"qwerty",
        // Some(Ident::new("Argon2i").test_unwrap()),
        Some(Ident::new("argon2i").test_unwrap()), // algorithm name is case-sensitive
        // Some(password_hash::Decimal::from_str("0x10").test_unwrap()),
        // Some(password_hash::Decimal::from_str_radix("0x10", 16).test_unwrap()),
        Some(password_hash::Decimal::from_str_radix("10", 16).test_unwrap()),
        argon2::Params::default(),
        // "salt".as_bytes(),
        salt_str.as_salt(),
    ).test_unwrap();

    println!("### psw_hash: {psw_hash}");
    assert_eq!(
        psw_hash.to_test_string(),
        "$argon2i$v=16$m=19456,t=2,p=1$MTIzNDU2Nzg5MDEyMzQ1Njc4OTA$4l4WGDDW0Y9YZp72yyYABS2t3Wo5H40tjQtgGi0wdzw",
    );

    // assert!(false, "Test error to see console output");
}

#[test]
fn test_hasher_3() {
    use password_hash::{ PasswordHasher };

    let hasher = argon2::Argon2::default();
    // let hasher = argon2::Argon2::new( // argon2::Argon2::default()
    //     argon2::Algorithm::Argon2i, argon2::Version::V0x10, argon2::Params::default());

    let salt_str = SaltString::encode_b64(b"12345678901234567890").test_unwrap();
    // password_hash::Salt::from()

    let psw_hash = hasher.hash_password_customized(
        b"qwerty",
        Some(argon2::ARGON2I_IDENT), // algorithm name is case-sensitive
        Some(argon2::Version::V0x10.into()),
        argon2::Params::default(),
        salt_str.as_salt(),
    ).test_unwrap();

    println!("### psw_hash: {psw_hash}");
    assert_eq!(
        psw_hash.to_test_string(),
        "$argon2i$v=16$m=19456,t=2,p=1$MTIzNDU2Nzg5MDEyMzQ1Njc4OTA$4l4WGDDW0Y9YZp72yyYABS2t3Wo5H40tjQtgGi0wdzw",
    );

    // assert!(false, "Test error to see console output");
}


#[test]
fn test_generate_salt_investigation() {
    use rand::SeedableRng;
    // use password_hash::rand_core::CryptoRngCore;

    // let salt_str = "BAavSXRIKbL+vyGb9uVZkg";
    // let salt = SaltString::from_b64(salt_str).test_unwrap();

    // let rng = rand_hc::Hc128Rng::from_entropy();
    let rng = rand_chacha::ChaCha20Rng::seed_from_u64(0u64);
    // let mut rng = rand::thread_rng();

    let salt = SaltString::generate(rng);

    println!("salt: {}", salt.as_str());
    // assert!(false, "To see output");
}


#[test]
fn test_generate_salt() {
    let salt = generate_salt().test_unwrap();
    println!("salt: {}", salt.as_str());
    // assert!(false, "To see output");
}


#[test]
fn test_create_psw_hash_and_verify_with_cfg_salt() {
    use mvv_auth::psw_hash::{
        algorithm::{ ARGON2D_ALG, ARGON2_VER_V0x10 },
        hash_psw_with_salt, PswHashConfig,
    };

    let salt_str_1 = "BAavSXRIKbL+vyGb9uVZkg";
    let salt_1 = SaltString::from_b64(salt_str_1).test_unwrap();

    let salt_str_2 = "YKjwEaoF/etx5+yyqAlwuQ";
    let salt_2 = SaltString::from_b64(salt_str_2).test_unwrap();

    let cfg_1 = PswHashConfig {
        algorithm: ARGON2D_ALG.to_string(),
        version: Some(ARGON2_VER_V0x10),
        salt_rnd_gen_type: None,
        salt: Some(salt_1),
    };

    let cfg_2 = PswHashConfig {
        algorithm: ARGON2D_ALG.to_string(),
        version: Some(ARGON2_VER_V0x10),
        salt_rnd_gen_type: None,
        salt: Some(salt_2),
    };

    let plain_psw_1 = SecureString::from_string("qwerty1".to_test_string());
    let psw_hash_1 = hash_psw_with_salt(&cfg_1, &plain_psw_1, cfg_1.salt.as_ref().test_unwrap().as_salt()).test_unwrap();
    println!("psw_hash_1: {psw_hash_1}");

    let psw_hash_1_2 = hash_psw_with_salt(&cfg_2, &plain_psw_1, cfg_2.salt.as_ref().test_unwrap().as_salt()).test_unwrap();
    println!("psw_hash_1_2: {psw_hash_1_2}");

    assert_ne!(psw_hash_1, psw_hash_1_2);


    let plain_psw_2 = SecureString::from_string("qwerty2".to_test_string());
    let psw_hash_2 = hash_psw_with_salt(&cfg_1, &plain_psw_2, cfg_1.salt.as_ref().test_unwrap().as_salt()).test_unwrap();
    println!("psw_hash_2: {psw_hash_2}");

    assert_ok!(verify_psw(plain_psw_1.as_bytes(), &psw_hash_1));
    assert_ok!(verify_psw(plain_psw_1.as_bytes(), &psw_hash_1_2));
    assert_ok!(verify_psw(plain_psw_2.as_bytes(), &psw_hash_2));

    assert_err!(verify_psw(plain_psw_1.as_bytes(), &psw_hash_2));
    assert_err!(verify_psw(&plain_psw_2.as_bytes(), &psw_hash_1));

    // assert!(false, "To see output");
}


#[test]
fn test_create_psw_hash_and_verify_with_generated_salt() {
    use mvv_auth::psw_hash::{
        algorithm::{ ARGON2D_ALG, ARGON2_VER_V0x10 },
        hash_psw_str, PswHashConfig,
    };

    let cfg = PswHashConfig {
        algorithm: ARGON2D_ALG.to_string(),
        version: Some(ARGON2_VER_V0x10),
        salt_rnd_gen_type: Some(SaltRndGenType::Default),
        salt: None,
    };

    let plain_psw_1 = SecureString::from_string("qwerty1".to_test_string());
    let psw_hash_and_salt_1 = hash_psw_str(&cfg, &plain_psw_1).test_unwrap();
    println!("psw_hash_1: {psw_hash_and_salt_1}");

    let psw_hash_and_salt_1_2 = hash_psw_str(&cfg, &plain_psw_1).test_unwrap();
    let psw_hash_and_salt_1_2 = psw_hash_and_salt_1_2;
    println!("psw_hash_1_2: {psw_hash_and_salt_1_2}");

    assert_ne!(psw_hash_and_salt_1, psw_hash_and_salt_1_2);


    let plain_psw_2 = SecureString::from_string("qwerty2".to_test_string());
    let psw_hash_and_salt_2 = hash_psw_str(&cfg, &plain_psw_2).test_unwrap();
    let psw_hash_and_salt_2 = psw_hash_and_salt_2;
    println!("psw_hash_2: {psw_hash_and_salt_2}");

    assert_ok!(verify_psw_by_hash_str(plain_psw_1.as_bytes(), &psw_hash_and_salt_1));
    assert_ok!(verify_psw_by_hash_str(plain_psw_1.as_bytes(), &psw_hash_and_salt_1_2));
    assert_ok!(verify_psw_by_hash_str(plain_psw_2.as_bytes(), &psw_hash_and_salt_2));

    assert_err!(verify_psw_by_hash_str(plain_psw_1.as_bytes(), &psw_hash_and_salt_2));
    assert_err!(verify_psw_by_hash_str(&plain_psw_2.as_bytes(), &psw_hash_and_salt_1));

    // assert!(false, "To see output");
}


#[test]
fn test_parsing_from_dif_encodes() {
    use mvv_auth::psw_hash::{
        algorithm::{ ARGON2D_ALG, ARGON2_VER_V0x10 },
        PswHashConfig,
    };

    let salt_str = "BAavSXRIKbL+vyGb9uVZkg";
    let salt_str = SaltString::from_b64(salt_str).test_unwrap();

    let cfg = PswHashConfig {
        algorithm: ARGON2D_ALG.to_string(),
        version: Some(ARGON2_VER_V0x10),
        salt_rnd_gen_type: Some(SaltRndGenType::Default),
        salt: Some(salt_str),
    };

    let plain_psw = SecureString::from_string("qwerty1".to_test_string());
    let psw_hash = hash_psw_with_salt(&cfg, &plain_psw, cfg.salt.as_ref().test_unwrap().as_salt()).test_unwrap();
    println!("psw_hash: {psw_hash}");
    let psw_hash_str = psw_hash.to_string();
    let ph: password_hash::PasswordHash = psw_hash_str.as_str().try_into().test_unwrap();
    println!("psw_hash: {ph}");

    // assert!(false, "To see output");
}


#[test]
fn generate_psw_hashes() {

    let cfg = PswHashConfig {
        algorithm: ARGON2D_ALG.to_string(),
        version: Some(ARGON2_VER_V0x10),
        salt_rnd_gen_type: Some(SaltRndGenType::Default),
        // salt: Some(salt_str),
        salt: None,
    };

    let psw = "psw4";
    let hash = hash_psw_str(&cfg, &SecureString::from_string(psw.to_owned())).test_unwrap();
    println!("psw [{psw}]: [{hash}]");

    // assert!(false, "To see output");
}

// psw [qwerty]: [$argon2d$v=16$m=19456,t=2,p=1$66zW697+kjcdOfLDOs6GGA$76kr0gFz1xQ6o2bpTqwoetOn0RoJ7QrSxRoMwLkA0xg]
// psw [qwerty]: [$argon2d$v=16$m=19456,t=2,p=1$1jd9ljbST1E/CsGOiBOsOg$4NUBW5Rf7Cm/E/+YtoQw2Vg3270pO5EWufqo3qmS7oE]
// psw [qwerty]: [$argon2d$v=16$m=19456,t=2,p=1$EUXO4jWOfnouAFF+CipWgQ$TleOX6tIVbCkynIGptjzRTcFVSbS2K3tZcmPckwJVt8]
// psw [psw1]: [$argon2d$v=16$m=19456,t=2,p=1$6suBJobcNzc8KE/leXzHKQ$etGYL8dLVSIFlX7TyclQczqfmx4hPm64nV3819H4vEo]
// psw [psw2]: [$argon2d$v=16$m=19456,t=2,p=1$lJ4IasWc2DTlqUobU0IWGw$ef49CpVe7Id4Sg2P4Ly7UdYkwlih5H8CsPy0ITzwLiM]
// psw [psw3]: [$argon2d$v=16$m=19456,t=2,p=1$GjKFhhuGE1wDSpPwPCVIJQ$wOZVOOvDFYTyhjCtRXYr3X0i6IFvseKQNtjHYrgNCQA]
// psw [psw4]: [$argon2d$v=16$m=19456,t=2,p=1$n3UFN93PKuf2OrK2xyXs+w$0pt4vmIHK3bWycKXJW6IWkQ/P9WWikpCrol5gcvll1k]

# sd-jwt onchain verifier

## Features

### Expiration Check

For sdjwt that has an expiration, instead of using the standard epoch time,
for cosmwasm, we use the key `crate::types::CW_EXPIRATION` (`cw_exp`) as opposed to `exp`,
which expects the value to be the serialised value of `cw_util::Expiration`.
If the route requires expiration check, the caller must include, in their `Criterion` - `Criterion::Expires(true)`

## Keys generation

### The keys encoding should be ASN1

- Keys generation for Ed25519 (we only support this key type for now)

```sh
openssl genpkey -algorithm ED25519 -out private.pem
openssl pkey -in private.pem -pubout -out public.pem
```

- For testing purposes you might want to import `jsonwebtoken` and `josekit` crates

- Below you'll find some examples of parsing keys, using aforementioned crates, for further interactions

```rust
/// Is used to get an sdjwt issuer instance with some ed25519 predefined private key, read from a file
pub fn issuer() -> SDJWTIssuer {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let encodingkey = EncodingKey::from_ed_pem(&encoding_key_pem).unwrap();
    SDJWTIssuer::new(encodingkey, Some("EdDSA".to_string()))
}

/// Is used to get an jwk public key instance from some ed25519 predefined private key, read from a file
pub fn issuer_jwk() -> josekit::jwk::Jwk {
    let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    key_path = key_path.join("fixtures/test_ed25519_private.pem");
    let encoding_key_pem = fs::read(key_path).unwrap();
    let key_pair = josekit::jwk::alg::ed::EdKeyPair::from_pem(encoding_key_pem).unwrap();
    println!("key_pair: {:#?}", key_pair);
    key_pair.to_jwk_public_key()
}
```

# TLS Example

## Generate Certificates and Keys

1. Install [mkcert](https://github.com/FiloSottile/mkcert)
2. Generate keys: `mkcert foobar.com`
3. Generate `p12` file: `openssl pkcs12 -export -out foobar.com.p12 -inkey foobar.com-key.pem -in foobar.com.pem -password pass:foobar`

## Run

1. Start TLS Server: `RUST_LOG=info cargo run --example tls-server`
2. Start TLS Client: `RUST_LOG=info cargo run --example tls-client`

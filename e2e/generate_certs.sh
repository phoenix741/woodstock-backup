#!/bin/sh
set -e

# Output directory
OUT_DIR=${1:-./certs}
mkdir -p "$OUT_DIR"

echo "Generating certificates in $OUT_DIR..."

# --- 1. Root CA ---
if [ ! -f "$OUT_DIR/rootCA.key" ]; then
    echo "Generating Root CA..."
    openssl genrsa -out "$OUT_DIR/rootCA.key" 4096
    openssl req -x509 -new -nodes -key "$OUT_DIR/rootCA.key" -sha256 -days 3650 \
        -out "$OUT_DIR/rootCA.pem" \
        -subj "/C=FR/ST=Paris/O=Woodstock Backup/CN=Woodstock Root CA"
else
    echo "Root CA already exists."
fi

# Function to generate signed certs
generate_cert() {
    NAME=$1
    SAN=$2
    CN=${3:-$NAME}
    echo "Generating cert for $NAME (CN=$CN)..."

    # Private Key
    if [ ! -f "$OUT_DIR/$NAME.key" ]; then
        openssl genrsa -out "$OUT_DIR/$NAME.key" 2048
    fi

    # CSR
    SUBJ="/C=FR/ST=Paris/O=Woodstock Backup/CN=$CN"
    if [ -n "$SAN" ]; then
        openssl req -new -key "$OUT_DIR/$NAME.key" -out "$OUT_DIR/$NAME.csr" -subj "$SUBJ" \
            -addext "subjectAltName = $SAN"
    else
        openssl req -new -key "$OUT_DIR/$NAME.key" -out "$OUT_DIR/$NAME.csr" -subj "$SUBJ"
    fi

    # Sign with CA (Use -copy_extensions copy to preserve SAN)
    # Note: openssl x509 -copy_extensions copy requires openssl 1.1.1+
    if [ -n "$SAN" ]; then
        openssl x509 -req -in "$OUT_DIR/$NAME.csr" \
            -CA "$OUT_DIR/rootCA.pem" -CAkey "$OUT_DIR/rootCA.key" -CAcreateserial \
            -out "$OUT_DIR/$NAME.pem" -days 3650 -sha256 \
            -copy_extensions copy
    else
        openssl x509 -req -in "$OUT_DIR/$NAME.csr" \
            -CA "$OUT_DIR/rootCA.pem" -CAkey "$OUT_DIR/rootCA.key" -CAcreateserial \
            -out "$OUT_DIR/$NAME.pem" -days 3650 -sha256
    fi
    
    rm "$OUT_DIR/$NAME.csr"
}

# --- 2. Server Certs ---
# The server loads: https.pem/key (for serving HTTPS) and <hostname>_server?
# server-client-api uses 'https.pem' and 'https.key' for the TLS config.
generate_cert "https" "DNS:server-client-api,DNS:localhost,IP:127.0.0.1"
generate_cert "server-client-api" "DNS:server-client-api" 

# --- 3. Client Certs ---
# Client A
# Client expects: <hostname>_server.key (Identity)
generate_cert "client-a_server" "DNS:client-a" "client-a"
# Also generates <hostname>_https.key just in case
generate_cert "client-a_https" "DNS:client-a" "client-a"

# Client B
generate_cert "client-b_server" "DNS:client-b" "client-b"
generate_cert "client-b_https" "DNS:client-b" "client-b"

echo "Certificates generated."
chown -R 1000:1000 "$OUT_DIR"
ls -l "$OUT_DIR"

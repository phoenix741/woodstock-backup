# Server Authentication to Client

## Overview

When the server attempts to back up a client, the server needs to authenticate the client device. This page describes the
authentication protocol used by the Woodstock backup server to authenticate the client device.

This security protocol is designed to authenticate the server with each client device that needs to be backed up, ensuring secure and trusted communication between them. It combines mutual TLS (mTLS) for secure communication and JSON Web Tokens (JWT) with RSA encryption (RS256 algorithm) for robust authentication. The protocol ensures that both the server and client can verify each other's identities and that the data transmitted is encrypted and protected against tampering.

## Components of the Protocol

### 1. RSA Key Pairs and Certificates

#### JWT Key Pair

On the first start of `api_server`, the server generates its own RSA key pair of 2048 bits stored
in the certificate directory (`public_key.pem`, `private_key.pem`). Generation is idempotent: an
existing pair is reused as-is.

The goal of this key pair is to sign the JWT token sent to the client. The public key is sent to the client to allow the client to verify the JWT token.

#### Device Certificate

On the first start of `api_server`, the server generates an authority certificate used to sign all devices certificates (stored in `rootCA.pem` and `rootCA.key`). `client_api_server` generates its own `https.pem`/`https.key` on its first start.

Per-device certificates are generated lazily, the first time the agent bundle is requested
(`GET /api/hosts/{name}/client`) — not when the host is added to `hosts.yml`. Until that request is
made, a backup of that host cannot start. The following certificates are then created:

- `${host}_ca.pem` and `${host}_ca.key`: The certificate authority of the device.
- `${host}_client.pem` and `${host}_client.key`: The server certificate signed with `rootCA` certificate.
- `${host}_server.pem` and `${host}_server.key`: The device certificate signed with `${host}_ca` certificate.
- `${host}_https.pem` and `${host}_https.key`: The device certificate used to register with the gateway, signed with `rootCA`.

Each certificate must assert the extended key usage matching the side of the
connection it authenticates. Getting this wrong makes rustls reject the
handshake with `UnsupportedCertificate`, which surfaces much later as a backup
that never starts:

| Certificate | Presented by | Toward | Extended key usage |
|---|---|---|---|
| `${host}_client` | server | the agent's gRPC port 3657 | `clientAuth` |
| `${host}_server` | agent | the worker connecting in | `serverAuth` |
| `${host}_https` | agent | the gateway on port 8443 | `clientAuth` |
| `https` | server | agents registering on 8443 | `serverAuth` |

All these certificates are stored in the certificate directory.

When the agent is downloaded on the Woodstock backup server UI, all necessary certificates are downloaded to the client:

- `rootCA.pem`: The root certificate authority used to verify the certificate of the server.
- `${host}_server.pem`: The certificate of the device.
- `${host}_server.key`: The private key of the device.
- `public_key.pem`: The public key of the server used to verify the JWT token.

### 2. Mutual TLS (mTLS)

The Woodstock backup server uses mTLS to authenticate the device. mTLS is a two-way authentication protocol where both the client and server authenticate each other using certificates.

The device uses the `${host}_server.pem` and `${host}_server.key` to authenticate itself to the server and the server will use the `${host}_ca.pem` to verify the device certificate, and in the other direction, the server uses the `${host}_client.pem` and `${host}_client.key` to authenticate itself to the device and the device will use the `rootCA.pem` to verify the server certificate.

## Authentication Flow

### Step 1: Establish Mutual TLS Connection

Before any application-level data is exchanged, the client and server set up a secure connection using mTLS.

- Device Actions:
  - Loads its own certificate and private key (`${host}_server.pem` and `${host}_server.key`).
  - Validates the server's certificate using the trusted CA certificate `rootCA.pem`.
  
- Server Actions:
  - Loads its own certificate and private key (`${host}_client.pem` and `${host}_client.key`).
  - Validates the client's certificate using the trusted CA certificate `${host}_ca.pem`.

Result: Both parties have verified each other's identities at the transport layer, and all subsequent communication is
encrypted.

### Step 2: Server Generates Authentication Token

The Woodstock backup server needs to authenticate at the application level using a JWT. A password is stored for each
device and is stored in a configuration file.

The Woodstock Backup server will generate a JWT token with the following claims:

- `iss`: The issuer of the token, in this case, `woodstock.shadoware.org`.
- `aud`: The audience of the token, in this case, `${host}`.
- `sub`: The subject of the token, in this case, `${host}`.
- `exp`: The expiration time of the token, in this case, 1 minute after the token creation.
- `hash`: The hash (SHA-256) of the password of the device.

The JWT token is signed with the server private key (`private_key.pem`).

### Step 3: Woodstock Backup Server Sends Authentication Request

The client sends an authentication request to the server over the established mTLS connection.

### Step 4: Device Verifies Authentication Token

Upon receiving the authentication request, the device performs several checks:

- Signature Verification: Uses the client's public RSA key to verify the JWT signature.
- Claims Validation: Checks standard claims (iss, aud, sub, exp, iat) for validity.
- Verifies the password hash matches the expected hash for that client.

### Step 5: Device Generates Session Token

After successful authentication, the server generates a session token for the Woodstock Backup Server.

A new HS256 JWT is created with claims including a unique session ID (session_id) and standard claims (a secret known
only by the device is used).

The session token should be sent in the metadata of each request by the Woodstock Backup Server.

### Step 6: Server Sends Session Token to Device

The Woodstock Backup Server sends the session token back to the client for each request.

The Device will verify the session token and process the request only if the session is valid.

### Step 7: Session Termination

Sessions are valid for a predetermined time or until explicitly terminated.

- Expiration: Session tokens include an expiration time after which they are no longer valid (the maximum backup duration).
- Manual Termination: The server can terminate the session, requiring re-authentication.
- Timeout: If no requests are received within a certain time, the session is terminated.

## Diagram: Authentication Sequence

@startuml
participant Client
participant Server

Client -> Server: Establish mTLS Connection
Server --> Client: mTLS Connection Established

Client -> Server: Send Authentication Request
Server -> Server: Generate JWT Token
Server -> Client: Send JWT Token

Client -> Client: Verify JWT Signature
Client -> Client: Validate JWT Claims
Client -> Client: Verify Password Hash

Client -> Server: Send Session Token (HS256 JWT)
Server -> Server: Verify Session Token
Server -> Client: Session Token Valid

Client -> Server: Send Authenticated Request (Include session_id in metadata)
Server -> Server: Process Request if Session is Valid

Client -> Server: Session Termination (Expiration/Manual/Timeout)
@enduml

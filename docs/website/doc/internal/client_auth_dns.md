# Client Authentication to Server

## Overview

When the client is connected to a network (device wakeup, network change, etc.), the client connect to the Woodstock
backup server (if resolution_mode is set to direct) to register itself. This page describes the
authentication protocol used by the Woodstock backup server to authenticate the client device.

## Authentication Protocol Flow

The authentication protocol between the Woodstock backup client and server uses mutual TLS (mTLS) certificate-based
authentication. This ensures that both the client and server can trust each other through cryptographic verification.

### Certificate Setup

Before the authentication can take place, the following certificates need to be in place:

1. **Root CA Certificate** (`rootCA.pem`): The Certificate Authority certificate used to sign both client and server certificates
2. **Server Certificates**:
   - Private key: `https.key`
   - Public certificate: `https.pem`
3. **Client HTTPS Certificates**:
   - Private key: `<hostname>_https.key`
   - Public certificate: `<hostname>_https.pem`

These certificates should be stored in the client's configuration directory.

### Client-Side Process

When a client is set to use the direct resolution mode, the following process occurs:

1. The client loads its certificates from the configuration directory:
   - Root CA certificate
   - HTTPS private key and public certificate

2. The client creates an HTTPS client with TLS authentication:
   - Sets up the TLS identity using the HTTPS certificates
   - Adds the Root CA certificate to trust

3. Periodically (every 60 seconds by default), the client performs the following actions:
   - Collects information about network interfaces
   - Sends this information to the server via HTTPS POST request
   - Authentication occurs automatically through the TLS handshake

4. The client sends a POST request to:

   ```http
   ¨POST {server}/api/hosts/{hostname}/client
   ```

   With a JSON payload containing:
   - List of network interfaces and IP addresses
   - Port number where the client is listening
   - Client version

### Server-Side Process

The server-side authentication is handled in the following way:

1. The server configures HTTPS with client certificate verification:
   - Loads the Root CA certificate
   - Sets up its own server certificate

2. When a client attempts to connect, the server receives the client's certificate during the TLS handshake.

3. The server verifies the certificate:
   - Extracts the Common Name (CN) from the certificate
   - Validates it against the registered hosts in authorized hosts
   - If the host is not found or invalid, throws an unauthorized exception

4. While the registration process of the host name, the server
   - Verifies that the certificate's CN matches the hostname in the URL path
   - If they don't match, throws a forbidden exception
   - If they match, registers the client's network information

## Security Considerations

- The mutual TLS authentication ensures that both parties can verify each other's identity.
- The Common Name (CN) in the certificate must match the hostname being registered.
- The server keeps track of registered hosts and verifies them before accepting connections.
- The periodic refresh ensures the server has up-to-date information about the client's network status.

## Error Handling

- If a client's certificate is not valid or not recognized, the connection is rejected.
- If a client attempts to register a different hostname than its own certificate, the request is forbidden.
- Network connectivity issues are handled through retries at regular intervals (every 60 seconds by default).

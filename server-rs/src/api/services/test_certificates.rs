//! Test pour vérifier la génération de certificats avec signature CA

#[cfg(test)]
mod tests {
    use super::super::certificate::CertificateService;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio;
    use woodstock::config::{Configuration, ConfigurationPath};

    #[tokio::test]
    async fn test_ca_certificate_generation() -> Result<(), Box<dyn std::error::Error>> {
        // Créer un répertoire temporaire pour les tests
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Créer une configuration de test
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Créer le service de certificats
        let cert_service = CertificateService::new(config);

        // Générer le certificat CA
        cert_service.generate_certificate().await?;

        // Vérifier que les fichiers ont été créés
        let ca_cert_path = temp_path.join("rootCA.pem");
        let ca_key_path = temp_path.join("rootCA.key");

        assert!(ca_cert_path.exists(), "Le certificat CA n'a pas été créé");
        assert!(ca_key_path.exists(), "La clé CA n'a pas été créée");

        // Vérifier que les fichiers ne sont pas vides
        let cert_content = tokio::fs::read_to_string(&ca_cert_path).await?;
        let key_content = tokio::fs::read_to_string(&ca_key_path).await?;

        assert!(!cert_content.is_empty(), "Le certificat CA est vide");
        assert!(!key_content.is_empty(), "La clé CA est vide");
        assert!(
            cert_content.contains("BEGIN CERTIFICATE"),
            "Format PEM invalide pour le certificat"
        );
        assert!(
            key_content.contains("BEGIN PRIVATE KEY"),
            "Format PEM invalide pour la clé"
        );

        println!("✅ Test CA certificate generation: PASSED");

        Ok(())
    }

    #[tokio::test]
    async fn test_https_certificate_signing() -> Result<(), Box<dyn std::error::Error>> {
        // Créer un répertoire temporaire pour les tests
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Créer une configuration de test
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Créer le service de certificats
        let cert_service = CertificateService::new(config);

        // Générer le certificat HTTPS (qui devrait d'abord créer le CA)
        cert_service.generate_https_certificate().await?;

        // Vérifier que tous les fichiers ont été créés
        let ca_cert_path = temp_path.join("rootCA.pem");
        let ca_key_path = temp_path.join("rootCA.key");
        let https_cert_path = temp_path.join("https.pem");
        let https_key_path = temp_path.join("https.key");

        assert!(ca_cert_path.exists(), "Le certificat CA n'a pas été créé");
        assert!(ca_key_path.exists(), "La clé CA n'a pas été créée");
        assert!(
            https_cert_path.exists(),
            "Le certificat HTTPS n'a pas été créé"
        );
        assert!(https_key_path.exists(), "La clé HTTPS n'a pas été créée");

        // Vérifier le contenu
        let https_cert_content = tokio::fs::read_to_string(&https_cert_path).await?;
        let https_key_content = tokio::fs::read_to_string(&https_key_path).await?;

        assert!(
            !https_cert_content.is_empty(),
            "Le certificat HTTPS est vide"
        );
        assert!(!https_key_content.is_empty(), "La clé HTTPS est vide");
        assert!(
            https_cert_content.contains("BEGIN CERTIFICATE"),
            "Format PEM invalide pour le certificat HTTPS"
        );
        assert!(
            https_key_content.contains("BEGIN PRIVATE KEY"),
            "Format PEM invalide pour la clé HTTPS"
        );

        println!("✅ Test HTTPS certificate signing: PASSED");

        Ok(())
    }

    #[tokio::test]
    async fn test_host_certificates_generation() -> Result<(), Box<dyn std::error::Error>> {
        // Créer un répertoire temporaire pour les tests
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Créer une configuration de test
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Créer le service de certificats
        let cert_service = CertificateService::new(config);

        // Générer tous les certificats pour un host
        let hostname = "test-host";
        cert_service.generate_host_certificate(hostname).await?;

        // Vérifier que tous les fichiers ont été créés
        let expected_files = vec![
            "rootCA.pem",
            "rootCA.key",
            "test-host_client.pem",
            "test-host_client.key",
            "test-host_ca.pem",
            "test-host_ca.key",
            "test-host_server.pem",
            "test-host_server.key",
            "test-host_https.pem",
            "test-host_https.key",
        ];

        for file in expected_files {
            let file_path = temp_path.join(file);
            assert!(file_path.exists(), "Le fichier {} n'a pas été créé", file);

            let content = tokio::fs::read_to_string(&file_path).await?;
            assert!(!content.is_empty(), "Le fichier {} est vide", file);

            if file.ends_with(".pem") {
                assert!(
                    content.contains("BEGIN CERTIFICATE"),
                    "Format PEM invalide pour {}",
                    file
                );
            } else if file.ends_with(".key") {
                assert!(
                    content.contains("BEGIN PRIVATE KEY"),
                    "Format PEM invalide pour {}",
                    file
                );
            }
        }

        println!("✅ Test host certificates generation: PASSED");

        Ok(())
    }

    /// `{host}_https` is the identity the agent presents to the mTLS gateway
    /// when it registers, so it must assert ClientAuth. It used to assert only
    /// ServerAuth, and rustls rejected the handshake with
    /// `UnsupportedCertificate` — the agent could never register and no backup
    /// could start.
    #[tokio::test]
    async fn test_host_https_certificate_allows_client_auth(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        let cert_service = CertificateService::new(config);
        let hostname = "test-host";
        cert_service.generate_host_certificate(hostname).await?;

        let host_https = temp_path.join(format!("{hostname}_https.pem"));
        let eku = read_extended_key_usage(&host_https).await?;
        assert!(
            eku.client_auth,
            "{hostname}_https doit permettre l'authentification cliente"
        );

        // The gateway's own certificate stays a pure server certificate.
        cert_service.generate_https_certificate().await?;
        let gateway_https = temp_path.join("https.pem");
        let gateway_eku = read_extended_key_usage(&gateway_https).await?;
        assert!(
            gateway_eku.server_auth && !gateway_eku.client_auth,
            "https.pem doit rester un certificat serveur uniquement"
        );

        Ok(())
    }

    /// Each of the four host certificates must assert the usage matching the
    /// side of the mTLS connection it authenticates. The Rust port inverted two
    /// of them relative to the TypeScript original, which broke agent
    /// registration and every backup.
    #[tokio::test]
    async fn test_host_certificates_key_usages() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        let cert_service = CertificateService::new(config);
        let hostname = "test-host";
        cert_service.generate_host_certificate(hostname).await?;

        // (file, must assert client auth, must assert server auth)
        let expectations = [
            // The worker presents this when it dials the agent.
            ("test-host_client.pem", true, false),
            // The agent presents this on its gRPC port 3657.
            ("test-host_server.pem", false, true),
            // The agent presents this to the gateway on port 8443.
            ("test-host_https.pem", true, false),
        ];

        for (file, want_client, want_server) in expectations {
            let eku = read_extended_key_usage(&temp_path.join(file)).await?;
            assert_eq!(
                eku.client_auth, want_client,
                "{file}: clientAuth attendu = {want_client}"
            );
            if want_server {
                assert!(eku.server_auth, "{file}: serverAuth attendu");
            }
        }

        Ok(())
    }

    /// A certificate issued before the fix must be reissued rather than reused:
    /// an agent holding it can never register.
    #[tokio::test]
    async fn test_stale_host_https_certificate_is_reissued(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();

        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_path.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        let cert_service = CertificateService::new(config);
        let hostname = "test-host";
        cert_service.generate_host_certificate(hostname).await?;

        // Stand in for a pre-fix certificate: the gateway's own ServerAuth-only
        // certificate, which is exactly what this file used to contain.
        cert_service.generate_https_certificate().await?;
        let host_https = temp_path.join(format!("{hostname}_https.pem"));
        tokio::fs::copy(temp_path.join("https.pem"), &host_https).await?;
        tokio::fs::copy(
            temp_path.join("https.key"),
            temp_path.join(format!("{hostname}_https.key")),
        )
        .await?;
        assert!(!read_extended_key_usage(&host_https).await?.client_auth);

        cert_service.generate_host_certificate(hostname).await?;

        assert!(
            read_extended_key_usage(&host_https).await?.client_auth,
            "un certificat sans ClientAuth doit être réémis"
        );

        Ok(())
    }

    /// Whether the certificate at `path` asserts ClientAuth and ServerAuth.
    ///
    /// Returns owned flags rather than the parsed `ExtendedKeyUsage`, which
    /// borrows from the DER buffer decoded here.
    async fn read_extended_key_usage(
        path: &std::path::Path,
    ) -> Result<Eku, Box<dyn std::error::Error>> {
        use rustls::pki_types::{pem::PemObject, CertificateDer};

        let pem = tokio::fs::read_to_string(path).await?;
        let der = CertificateDer::from_pem_slice(pem.as_bytes())?;
        let (_, x509) = x509_parser::parse_x509_certificate(&der)?;
        let eku = x509
            .extended_key_usage()?
            .ok_or("le certificat ne déclare aucun extended key usage")?;

        Ok(Eku {
            client_auth: eku.value.client_auth,
            server_auth: eku.value.server_auth,
        })
    }

    struct Eku {
        client_auth: bool,
        server_auth: bool,
    }
}

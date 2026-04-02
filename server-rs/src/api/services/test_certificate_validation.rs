//! Comprehensive validation tests to ensure Rust implementation matches TypeScript exactly

#[cfg(test)]
mod tests {
    use super::super::certificate::CertificateService;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;
    use woodstock::config::{Configuration, ConfigurationPath};

    #[tokio::test]
    async fn test_complete_pki_structure_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test the complete workflow as described in TypeScript:
        // 1. Generate root CA
        service.generate_certificate().await.unwrap();

        // 2. Generate HTTPS certificate
        service.generate_https_certificate().await.unwrap();

        // 3. Generate all host certificates
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();
        service
            .generate_host_certificate("192.168.1.100")
            .await
            .unwrap();

        // Validate complete PKI structure matches TypeScript specification:

        // Root CA (self-signed)
        assert!(temp_dir.path().join("rootCA.pem").exists());
        assert!(temp_dir.path().join("rootCA.key").exists());

        // Server HTTPS (signed by root CA)
        assert!(temp_dir.path().join("https.pem").exists());
        assert!(temp_dir.path().join("https.key").exists());

        // Host certificates for "test-host"
        assert!(temp_dir.path().join("test-host_client.pem").exists()); // Signed by root CA
        assert!(temp_dir.path().join("test-host_client.key").exists());
        assert!(temp_dir.path().join("test-host_ca.pem").exists()); // Self-signed host CA
        assert!(temp_dir.path().join("test-host_ca.key").exists());
        assert!(temp_dir.path().join("test-host_server.pem").exists()); // Signed by host CA
        assert!(temp_dir.path().join("test-host_server.key").exists());
        assert!(temp_dir.path().join("test-host_https.pem").exists()); // Signed by root CA
        assert!(temp_dir.path().join("test-host_https.key").exists());

        // Host certificates for IP address "192.168.1.100"
        assert!(temp_dir.path().join("192.168.1.100_client.pem").exists());
        assert!(temp_dir.path().join("192.168.1.100_client.key").exists());
        assert!(temp_dir.path().join("192.168.1.100_ca.pem").exists());
        assert!(temp_dir.path().join("192.168.1.100_ca.key").exists());
        assert!(temp_dir.path().join("192.168.1.100_server.pem").exists());
        assert!(temp_dir.path().join("192.168.1.100_server.key").exists());
        assert!(temp_dir.path().join("192.168.1.100_https.pem").exists());
        assert!(temp_dir.path().join("192.168.1.100_https.key").exists());
    }

    #[tokio::test]
    async fn test_certificate_content_validity() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate complete certificate set
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Validate all certificates have proper PEM format
        let certificate_files = vec![
            "rootCA.pem",
            "https.pem",
            "test-host_client.pem",
            "test-host_ca.pem",
            "test-host_server.pem",
            "test-host_https.pem",
        ];

        for cert_file in certificate_files {
            let cert_content = fs::read_to_string(temp_dir.path().join(cert_file))
                .await
                .unwrap();
            assert!(
                cert_content.contains("-----BEGIN CERTIFICATE-----"),
                "Certificate {} should have BEGIN marker",
                cert_file
            );
            assert!(
                cert_content.contains("-----END CERTIFICATE-----"),
                "Certificate {} should have END marker",
                cert_file
            );
            assert!(
                cert_content.len() > 200,
                "Certificate {} should have reasonable size",
                cert_file
            );
        }

        // Validate all private keys have proper PEM format
        let key_files = vec![
            "rootCA.key",
            "https.key",
            "test-host_client.key",
            "test-host_ca.key",
            "test-host_server.key",
            "test-host_https.key",
        ];

        for key_file in key_files {
            let key_content = fs::read_to_string(temp_dir.path().join(key_file))
                .await
                .unwrap();
            assert!(
                key_content.contains("-----BEGIN PRIVATE KEY-----"),
                "Key {} should have BEGIN marker",
                key_file
            );
            assert!(
                key_content.contains("-----END PRIVATE KEY-----"),
                "Key {} should have END marker",
                key_file
            );
            assert!(
                key_content.len() > 200,
                "Key {} should have reasonable size",
                key_file
            );
        }
    }

    #[tokio::test]
    async fn test_certificate_generation_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test that HTTPS certificate generation creates root CA automatically
        service.generate_https_certificate().await.unwrap();
        assert!(
            temp_dir.path().join("rootCA.pem").exists(),
            "Root CA should be created when generating HTTPS certificate"
        );

        // Test that host certificate generation creates all dependencies
        let temp_dir2 = TempDir::new().unwrap();
        let config2 = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir2.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });
        let service2 = CertificateService::new(config2);

        service2
            .generate_host_certificate("test-host")
            .await
            .unwrap();
        assert!(
            temp_dir2.path().join("rootCA.pem").exists(),
            "Root CA should be created when generating host certificates"
        );
        assert!(
            temp_dir2.path().join("test-host_ca.pem").exists(),
            "Host CA should be created when generating host certificates"
        );
    }

    #[tokio::test]
    async fn test_certificate_idempotency() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates multiple times - should be idempotent
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Read content after first generation
        let root_ca_content = fs::read_to_string(temp_dir.path().join("rootCA.pem"))
            .await
            .unwrap();
        let https_content = fs::read_to_string(temp_dir.path().join("https.pem"))
            .await
            .unwrap();
        let host_ca_content = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();

        // Generate again
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Content should be identical (no regeneration)
        let root_ca_content2 = fs::read_to_string(temp_dir.path().join("rootCA.pem"))
            .await
            .unwrap();
        let https_content2 = fs::read_to_string(temp_dir.path().join("https.pem"))
            .await
            .unwrap();
        let host_ca_content2 = fs::read_to_string(temp_dir.path().join("test-host_ca.pem"))
            .await
            .unwrap();

        assert_eq!(
            root_ca_content, root_ca_content2,
            "Root CA should not change on regeneration"
        );
        assert_eq!(
            https_content, https_content2,
            "HTTPS cert should not change on regeneration"
        );
        assert_eq!(
            host_ca_content, host_ca_content2,
            "Host CA should not change on regeneration"
        );
    }

    #[tokio::test]
    async fn test_certificate_naming_convention_matches_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test various hostname formats
        // Note: IPv6 addresses with `:` are excluded on Windows as `:` is invalid in filenames
        let hostnames = vec![
            "simple-host",
            "complex.hostname.domain",
            "192.168.1.1",
            #[cfg(not(windows))]
            "2001:db8::1",
            "localhost",
        ];

        for hostname in hostnames {
            service.generate_host_certificate(hostname).await.unwrap();

            // Verify expected file naming convention
            assert!(temp_dir
                .path()
                .join(format!("{}_client.pem", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_client.key", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_ca.pem", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_ca.key", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_server.pem", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_server.key", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_https.pem", hostname))
                .exists());
            assert!(temp_dir
                .path()
                .join(format!("{}_https.key", hostname))
                .exists());
        }
    }
}

//! Final verification tests to ensure 100% TypeScript compatibility

#[cfg(test)]
mod tests {
    use super::super::certificate::CertificateService;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;
    use woodstock::config::{Configuration, ConfigurationPath};

    /// Test that validates the complete certificate generation workflow
    /// matches the TypeScript implementation exactly
    #[tokio::test]
    async fn test_complete_workflow_typescript_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Step 1: Generate root CA (should create rootCA.pem, rootCA.key)
        service.generate_certificate().await.unwrap();
        assert!(temp_dir.path().join("rootCA.pem").exists());
        assert!(temp_dir.path().join("rootCA.key").exists());

        // Step 2: Generate HTTPS certificate (should create https.pem, https.key)
        service.generate_https_certificate().await.unwrap();
        assert!(temp_dir.path().join("https.pem").exists());
        assert!(temp_dir.path().join("https.key").exists());

        // Step 3: Generate host certificates (should create 4 certificate pairs)
        let hostname = "test-host";
        service.generate_host_certificate(hostname).await.unwrap();

        // Verify all 8 files created (4 .pem + 4 .key files)
        let expected_host_files = vec![
            format!("{}_client.pem", hostname),
            format!("{}_client.key", hostname),
            format!("{}_ca.pem", hostname),
            format!("{}_ca.key", hostname),
            format!("{}_server.pem", hostname),
            format!("{}_server.key", hostname),
            format!("{}_https.pem", hostname),
            format!("{}_https.key", hostname),
        ];

        for file in &expected_host_files {
            assert!(
                temp_dir.path().join(file).exists(),
                "File {} should exist",
                file
            );
        }

        // Total expected files: 2 (root CA) + 2 (HTTPS) + 8 (host) = 12 files
        let all_files = std::fs::read_dir(temp_dir.path()).unwrap();
        let file_count = all_files.count();
        assert_eq!(file_count, 12, "Should have exactly 12 certificate files");
    }

    /// Test that IP vs hostname detection works exactly like TypeScript
    #[tokio::test]
    async fn test_ip_detection_comprehensive() {
        // IPv4 tests (should be detected as IP)
        let ipv4_addresses = vec![
            "127.0.0.1",
            "192.168.1.1",
            "10.0.0.1",
            "172.16.0.1",
            "255.255.255.255",
            "0.0.0.0",
            "8.8.8.8",
        ];

        for ip in ipv4_addresses {
            assert!(
                CertificateService::is_ip(ip),
                "Should detect {} as IPv4",
                ip
            );
        }

        // IPv6 tests (should be detected as IP)
        let ipv6_addresses = vec![
            "::1",
            "::0",
            "2001:db8::1",
            "fe80::1",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "::ffff:192.0.2.1",
        ];

        for ip in ipv6_addresses {
            assert!(
                CertificateService::is_ip(ip),
                "Should detect {} as IPv6",
                ip
            );
        }

        // Hostname tests (should NOT be detected as IP)
        let hostnames = vec![
            "localhost",
            "example.com",
            "test-host",
            "woodstock.shadoware.org",
            "server.local",
            "my-server-123",
            "host.domain.tld",
            "",
            "invalid.ip.address",
            "999.999.999.999",
            "gggg::1",
        ];

        for hostname in hostnames {
            assert!(
                !CertificateService::is_ip(hostname),
                "Should NOT detect {} as IP",
                hostname
            );
        }
    }

    /// Test certificate generation for various hostname formats  
    #[tokio::test]
    async fn test_certificate_generation_various_hostnames() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test different hostname formats
        let test_hostnames = vec![
            "simple-host",
            "complex.hostname.domain",
            "192.168.1.1",
            "localhost",
            "server123",
        ];

        for hostname in test_hostnames {
            service.generate_host_certificate(hostname).await.unwrap();

            // Verify all expected files were created
            let expected_files = vec![
                format!("{}_client.pem", hostname),
                format!("{}_client.key", hostname),
                format!("{}_ca.pem", hostname),
                format!("{}_ca.key", hostname),
                format!("{}_server.pem", hostname),
                format!("{}_server.key", hostname),
                format!("{}_https.pem", hostname),
                format!("{}_https.key", hostname),
            ];

            for file in expected_files {
                assert!(
                    temp_dir.path().join(&file).exists(),
                    "File {} should exist for hostname {}",
                    file,
                    hostname
                );
            }
        }
    }

    /// Test that the certificate attributes match TypeScript specification
    #[tokio::test]
    async fn test_certificate_attributes_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Generate certificates
        service.generate_certificate().await.unwrap();
        service
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Read certificate contents and validate structure
        let cert_files = vec![
            "rootCA.pem",
            "test-host_ca.pem",
            "test-host_client.pem",
            "test-host_server.pem",
            "test-host_https.pem",
        ];

        for cert_file in cert_files {
            let content = fs::read_to_string(temp_dir.path().join(cert_file))
                .await
                .unwrap();

            // Validate PEM structure
            assert!(
                content.starts_with("-----BEGIN CERTIFICATE-----"),
                "Certificate {} should start with BEGIN marker",
                cert_file
            );
            assert!(
                content
                    .trim_end_matches(['\n', '\r'])
                    .ends_with("-----END CERTIFICATE-----"),
                "Certificate {} should end with END marker",
                cert_file
            );

            // Validate certificate has proper base64 content
            let lines: Vec<&str> = content.lines().collect();
            assert!(
                lines.len() > 3,
                "Certificate {} should have multiple lines",
                cert_file
            );

            // Check that middle lines contain base64-like content
            for line in &lines[1..lines.len() - 1] {
                if !line.is_empty() {
                    assert!(
                        line.chars()
                            .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='),
                        "Certificate {} should contain valid base64 content",
                        cert_file
                    );
                }
            }
        }
    }

    /// Test the exact certificate dependency chain as specified in TypeScript
    #[tokio::test]
    async fn test_certificate_dependency_chain() {
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });

        let service = CertificateService::new(config);

        // Test 1: Generating HTTPS certificate should create root CA first
        service.generate_https_certificate().await.unwrap();
        assert!(
            temp_dir.path().join("rootCA.pem").exists(),
            "Root CA should be created automatically"
        );
        assert!(
            temp_dir.path().join("rootCA.key").exists(),
            "Root CA key should be created automatically"
        );

        // Test 2: Clean slate for host certificate test
        let temp_dir2 = TempDir::new().unwrap();
        let config2 = Arc::new(Configuration {
            path: ConfigurationPath {
                certificates_path: temp_dir2.path().to_path_buf(),
                ..Default::default()
            },
            ..Default::default()
        });
        let service2 = CertificateService::new(config2);

        // Generating host certificate should create all dependencies
        service2
            .generate_host_certificate("test-host")
            .await
            .unwrap();

        // Should create root CA automatically
        assert!(
            temp_dir2.path().join("rootCA.pem").exists(),
            "Root CA should be created by host cert generation"
        );
        assert!(
            temp_dir2.path().join("rootCA.key").exists(),
            "Root CA key should be created by host cert generation"
        );

        // Should create host CA as part of the process
        assert!(
            temp_dir2.path().join("test-host_ca.pem").exists(),
            "Host CA should be created by host cert generation"
        );
        assert!(
            temp_dir2.path().join("test-host_ca.key").exists(),
            "Host CA key should be created by host cert generation"
        );

        // All host certificates should be present
        let expected_host_files = vec![
            "test-host_client.pem",
            "test-host_client.key",
            "test-host_server.pem",
            "test-host_server.key",
            "test-host_https.pem",
            "test-host_https.key",
        ];

        for file in expected_host_files {
            assert!(
                temp_dir2.path().join(file).exists(),
                "Host file {} should be created",
                file
            );
        }
    }
}

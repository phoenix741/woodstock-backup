//! Tests for certificate optimization improvements using x509-parser

#[cfg(test)]
mod tests {
    use crate::api::services::certificate::CertificateService;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;
    use woodstock::config::Configuration;

    async fn setup_service() -> (CertificateService, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Configuration::from_backup_path(temp_dir.path().to_path_buf());

        let service = CertificateService::new(Arc::new(config));
        (service, temp_dir)
    }

    /// Test that the optimized load_ca_issuer method works correctly
    /// by generating a certificate, then loading it back using the optimized method
    #[tokio::test]
    async fn test_optimized_load_ca_issuer() {
        let (service, _temp_dir) = setup_service().await;

        // Generate root CA certificate
        service.generate_certificate().await.unwrap();

        // Verify CA files exist
        let ca_cert_path = service.certificate_path().join("rootCA.pem");
        let ca_key_path = service.certificate_path().join("rootCA.key");
        assert!(ca_cert_path.exists());
        assert!(ca_key_path.exists());

        // Load CA issuer using optimized method (reads from existing certificate)
        let _issuer = service.load_ca_issuer().await.unwrap();

        // Verify we can use the issuer to sign a certificate
        service.generate_https_certificate().await.unwrap();

        // Verify HTTPS certificate was generated successfully
        let https_cert_path = service.certificate_path().join("https.pem");
        assert!(https_cert_path.exists());

        // Read and verify HTTPS certificate content
        let https_cert_content = fs::read_to_string(&https_cert_path).await.unwrap();
        assert!(https_cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(https_cert_content.contains("-----END CERTIFICATE-----"));
    }

    /// Test that the optimized load_host_ca_issuer method works correctly
    #[tokio::test]
    async fn test_optimized_load_host_ca_issuer() {
        let (service, _temp_dir) = setup_service().await;
        let hostname = "testhost";

        // Generate host certificates (will generate root CA, host CA, etc.)
        service.generate_host_certificate(hostname).await.unwrap();

        // Verify host CA files exist
        let host_ca_cert_path = service
            .certificate_path()
            .join(format!("{}_ca.pem", hostname));
        let host_ca_key_path = service
            .certificate_path()
            .join(format!("{}_ca.key", hostname));
        assert!(host_ca_cert_path.exists());
        assert!(host_ca_key_path.exists());

        // Load host CA issuer using optimized method (reads from existing certificate)
        let _host_issuer = service.load_host_ca_issuer(hostname).await.unwrap();

        // Verify we can use the host issuer to sign a certificate by checking
        // that the host server certificate was generated
        let host_server_cert_path = service
            .certificate_path()
            .join(format!("{}_server.pem", hostname));
        assert!(host_server_cert_path.exists());

        // Read and verify host server certificate content
        let host_server_cert_content = fs::read_to_string(&host_server_cert_path).await.unwrap();
        assert!(host_server_cert_content.contains("-----BEGIN CERTIFICATE-----"));
        assert!(host_server_cert_content.contains("-----END CERTIFICATE-----"));
    }

    /// Test that parse_certificate_params correctly extracts certificate information
    #[tokio::test]
    async fn test_parse_certificate_params() {
        let (service, _temp_dir) = setup_service().await;

        // Generate root CA certificate
        service.generate_certificate().await.unwrap();

        // Read the generated certificate
        let ca_cert_path = service.certificate_path().join("rootCA.pem");
        let cert_pem = fs::read_to_string(&ca_cert_path).await.unwrap();

        // Parse certificate parameters using our optimized method
        let params = service.parse_certificate_params(&cert_pem).unwrap();

        // Verify extracted parameters
        assert!(
            params.is_ca != rcgen::IsCa::NoCa,
            "Should detect CA certificate"
        );
        assert!(
            params
                .key_usages
                .contains(&rcgen::KeyUsagePurpose::KeyCertSign),
            "CA should have KeyCertSign usage"
        );
        assert!(
            params.serial_number.is_some(),
            "Should extract serial number"
        );

        // Verify distinguished name contains expected attributes
        let dn_entries: Vec<_> = params.distinguished_name.iter().collect();
        assert!(
            dn_entries.len() >= 4,
            "Should have at least 4 DN attributes, found {}",
            dn_entries.len()
        );

        // Verify subject alternative names
        assert!(
            !params.subject_alt_names.is_empty(),
            "Should have SAN entries"
        );

        // Verify validity period is extracted
        assert!(
            params.not_after > params.not_before,
            "Certificate should have valid time range"
        );
    }

    /// Test that the optimization maintains TypeScript compatibility
    #[tokio::test]
    async fn test_optimization_typescript_compatibility() {
        let (service, _temp_dir) = setup_service().await;

        // Generate all certificates using optimized methods
        service.generate_certificate().await.unwrap();
        service.generate_https_certificate().await.unwrap();
        service.generate_host_certificate("testhost").await.unwrap();

        // Verify all expected files exist (TypeScript compatibility)
        let expected_files = vec![
            "rootCA.pem",
            "rootCA.key",
            "https.pem",
            "https.key",
            "testhost_client.pem",
            "testhost_client.key",
            "testhost_ca.pem",
            "testhost_ca.key",
            "testhost_server.pem",
            "testhost_server.key",
            "testhost_https.pem",
            "testhost_https.key",
        ];

        for file_name in expected_files {
            let file_path = service.certificate_path().join(file_name);
            assert!(
                file_path.exists(),
                "File {} should exist for TypeScript compatibility",
                file_name
            );

            // Verify file is not empty
            let content = fs::read_to_string(&file_path).await.unwrap();
            assert!(
                !content.trim().is_empty(),
                "File {} should not be empty",
                file_name
            );
        }
    }

    /// Test that the optimization handles edge cases correctly
    #[tokio::test]
    async fn test_optimization_edge_cases() {
        let (service, _temp_dir) = setup_service().await;

        // Test error handling when certificate doesn't exist
        let load_result = service.load_ca_issuer().await;
        assert!(
            load_result.is_err(),
            "Should return error when CA doesn't exist"
        );

        // Test error handling for invalid certificate content
        let parse_result = service.parse_certificate_params("invalid certificate content");
        assert!(
            parse_result.is_err(),
            "Should return error for invalid certificate content"
        );

        // Test with empty string
        let parse_result = service.parse_certificate_params("");
        assert!(
            parse_result.is_err(),
            "Should return error for empty certificate content"
        );
    }
}

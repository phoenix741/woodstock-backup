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
}

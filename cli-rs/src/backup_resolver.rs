//! Utilitaire de résolution d'identifiants de backup (UUID ou numéro séquentiel).

use eyre::Result;
use uuid::Uuid;
use woodstock::config::Backups;

/// Résout un identifiant de backup (UUID ou numéro séquentiel) en `(Uuid, number)`.
///
/// Accepte aussi bien une chaîne UUID v7 qu'un entier (numéro d'affichage).
///
/// # Errors
///
/// Retourne une erreur si l'identifiant n'est ni un UUID valide ni un entier,
/// ou si aucun backup correspondant n'existe.
pub async fn resolve_backup_id(
    arg: &str,
    hostname: &str,
    backups: &Backups,
) -> Result<(Uuid, usize)> {
    // Essayer d'abord comme UUID
    if let Ok(uuid) = Uuid::parse_str(arg) {
        let backup = backups
            .get_backup(hostname, uuid)
            .await
            .ok_or_else(|| eyre::eyre!("Backup {} not found for host {}", arg, hostname))?;
        return Ok((uuid, backup.number));
    }
    // Essayer ensuite comme numéro séquentiel
    if let Ok(n) = arg.parse::<usize>() {
        let backup = backups
            .get_backup_by_number(hostname, n)
            .await
            .ok_or_else(|| eyre::eyre!("Backup number {} not found for host {}", n, hostname))?;
        return Ok((backup.id, n));
    }
    Err(eyre::eyre!(
        "Invalid backup identifier '{}': expected a UUID or a number",
        arg
    ))
}

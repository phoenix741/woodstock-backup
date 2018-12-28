import * as path from 'path'

/**
 * Location of configurations, all the backups.
 */
export const NG_MASS_BACKUP_PATH = process.env.NG_MASS_BACKUP_PATH || '/var/lib/ngmassbackup'

/**
 * Location of the configuration.
 */
export const NG_MASS_BACKUP_CONFIG_PATH = process.env.NG_MASS_BACKUP_CONFIG_PATH || path.join(NG_MASS_BACKUP_PATH, 'config')

/**
 * Location of the list of hosts and configuration of each hosts.
 */
export const NG_MASS_BACKUP_CONFIG_HOSTS_PATH = path.join(NG_MASS_BACKUP_CONFIG_PATH, 'hosts.yml')

/**
 * Location of backups of the backup of each hosts.
 */
export const NG_MASS_BACKUP_HOST_PATH = process.env.NG_MASS_BACKUP_HOST_PATH || path.join(NG_MASS_BACKUP_PATH, 'hosts')

/**
 * Location of the logs of the application.
 */
export const NG_MASS_BACKUP_LOG_PATH = process.env.NG_MASS_BACKUP_LOG_PATH || path.join(NG_MASS_BACKUP_PATH, 'log')

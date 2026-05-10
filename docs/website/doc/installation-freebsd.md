# Installation sur FreeBSD

Ce guide décrit comment installer le serveur Woodstock Backup sur FreeBSD à l'aide des paquets `.pkg` officiels.

::: info Différences FreeBSD vs Linux
Sur FreeBSD, les conventions de chemins diffèrent légèrement de Linux :

- Données : `/var/db/woodstock/` (au lieu de `/var/lib/woodstock/`)
- Binaires : `/usr/local/bin/` (au lieu de `/usr/bin/`)
- Config : `/usr/local/etc/woodstock/` (au lieu de `/etc/woodstock/`)
- Services : rc.d (au lieu de systemd)
:::

## Prérequis

- FreeBSD 14.x ou supérieur
- Accès `root`
- **Valkey** (ou Redis) installé et en fonctionnement
- Espace disque suffisant pour les sauvegardes

## 1. Configurer le dépôt de paquets

Les paquets Woodstock sont distribués via le registre générique de Gitea. Téléchargez le paquet directement depuis les releases :

```bash
# Récupérer la dernière version disponible (remplacer X.Y.Z par la version souhaitée)
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-server-${WOODSTOCK_VERSION}.pkg"
```

## 2. Installer le serveur

```bash
pkg install /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg
```

Cela installe :

- Les 4 binaires serveur dans `/usr/local/bin/`
- L'interface web Vue.js dans `/usr/local/share/woodstock/static/`
- 4 scripts rc.d dans `/usr/local/etc/rc.d/`
- Le fichier de configuration exemple `/usr/local/etc/woodstock/server.env.sample`
- L'utilisateur système `woodstock` (UID 565, non-root)
- Les répertoires de données dans `/var/db/woodstock/`

## 3. Installer Valkey

Le serveur Woodstock nécessite **Valkey** (compatible Redis) pour la file de jobs et les verrous distribués :

```bash
pkg install databases/valkey
```

Activer et démarrer Valkey :

```bash
echo 'valkey_enable="YES"' >> /etc/rc.conf
service valkey start
```

Vérifier que Valkey est opérationnel :

```bash
valkey-cli ping
# Réponse attendue : PONG
```

## 4. Configurer le serveur

Créez le fichier de configuration à partir de l'exemple fourni :

```bash
cp /usr/local/etc/woodstock/server.env.sample /usr/local/etc/woodstock/server.env
chmod 640 /usr/local/etc/woodstock/server.env
chown root:woodstock /usr/local/etc/woodstock/server.env
```

Éditez le fichier :

```bash
vi /usr/local/etc/woodstock/server.env
```

Les paramètres essentiels :

```ini
# Chemin des données de sauvegarde (convention FreeBSD)
BACKUP_PATH=/var/db/woodstock

# Connexion Valkey/Redis
REDIS_HOST=localhost
REDIS_PORT=6379

# Interface web pre-compilée
STATIC_PATH=/usr/local/share/woodstock/static

# Niveau de log : error, warn, info, debug
LOG_LEVEL=info

# Port de l'API REST (défaut : 3000)
MANAGEMENT_API_PORT=3000

# Concurrence du worker
BACKUP_CONCURRENCY=2
RESTORE_CONCURRENCY=8
MAINTENANCE_CONCURRENCY=2
```

::: tip Espace disque
Si vos données de sauvegarde doivent résider sur un disque différent, montez-le sur `/var/db/woodstock` ou modifiez `BACKUP_PATH`.
:::

## 5. Activer et démarrer les services

Ajoutez les 4 services à `/etc/rc.conf` :

```bash
# Activer tous les services Woodstock
sysrc woodstock_worker_enable="YES"
sysrc woodstock_scheduler_enable="YES"
sysrc woodstock_api_enable="YES"
sysrc woodstock_client_api_enable="YES"
```

Démarrez les services dans l'ordre recommandé (worker et scheduler en premier) :

```bash
service woodstock_worker start
service woodstock_scheduler start
service woodstock_api start
service woodstock_client_api start
```

Les 4 services disponibles :

| Service rc.d | Rôle | Port |
|--------------|------|------|
| `woodstock_api` | API REST + interface web | 3000 |
| `woodstock_client_api` | Passerelle mTLS pour les agents | 8443 |
| `woodstock_worker` | Worker de backup/restauration | — |
| `woodstock_scheduler` | Planificateur CRON | — |

## 6. Vérifier l'installation

```bash
# Vérifier l'état des services
service woodstock_api status
service woodstock_worker status

# Accéder à l'interface web
fetch -o - http://localhost:3000/ | head -5

# Consulter les logs
tail -f /var/log/woodstock/*.log
```

L'interface web est disponible sur `http://<adresse-du-serveur>:3000`.

## 7. Installer le client sur les machines à sauvegarder

```bash
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-client-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-client-${WOODSTOCK_VERSION}.pkg"

pkg install /tmp/woodstock-client-${WOODSTOCK_VERSION}.pkg
```

Puis configurez le client :

```bash
cp /usr/local/etc/woodstock/config.yaml.sample /usr/local/etc/woodstock/config.yaml
vi /usr/local/etc/woodstock/config.yaml

# Activer et démarrer le service client
sysrc woodstock_client_enable="YES"
service woodstock_client start
```

Voir le [guide de configuration de l'agent](/doc/agent) pour les détails.

## Pare-feu (pf)

Si vous utilisez `pf`, ajoutez les règles suivantes dans `/etc/pf.conf` :

```pf
# Interface web Woodstock (HTTPS recommandé via reverse proxy)
pass in on em0 proto tcp to port 3000

# Passerelle mTLS pour les agents (obligatoire)
pass in on em0 proto tcp to port 8443
```

Puis rechargez les règles :

```bash
pfctl -f /etc/pf.conf
```

## Structure des données sur FreeBSD

```
/var/db/woodstock/
├── certs/          # Certificats mTLS
├── config/         # Fichiers YAML de configuration
├── hosts/          # Données par machine sauvegardée
├── logs/           # Journaux de l'application
├── pool/           # Stockage CAS (chunks dédupliqués)
├── events/         # Journal d'audit
└── jobs/           # État des jobs

/usr/local/etc/woodstock/
└── server.env      # Configuration du serveur

/usr/local/share/woodstock/
└── static/         # Interface web Vue.js
```

## Mise à jour

```bash
# Télécharger la nouvelle version
WOODSTOCK_VERSION="X.Y.Z"
fetch -o /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg \
  "https://gogs.shadoware.org/api/packages/ShadowareOrg/generic/woodstock-freebsd/${WOODSTOCK_VERSION}/woodstock-server-${WOODSTOCK_VERSION}.pkg"

# Mettre à jour (arrêt automatique des services par pre-deinstall)
pkg upgrade /tmp/woodstock-server-${WOODSTOCK_VERSION}.pkg

# Redémarrer les services
service woodstock_worker start
service woodstock_scheduler start
service woodstock_api start
service woodstock_client_api start
```

## Désinstallation

```bash
# Supprimer le paquet (conserve les données dans /var/db/woodstock)
pkg delete woodstock-server
```

::: warning Conservation des données
Le paquet FreeBSD ne supprime **pas** automatiquement `/var/db/woodstock` lors de la désinstallation. Supprimez manuellement ce répertoire si vous souhaitez effacer toutes les sauvegardes :

```bash
rm -rf /var/db/woodstock
```

:::

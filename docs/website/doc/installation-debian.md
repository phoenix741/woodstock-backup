# Installation via Paquet Debian

Ce guide décrit comment installer le serveur Woodstock Backup sur un système Debian/Ubuntu à l'aide du paquet `.deb` officiel.

## Prérequis

- Debian 12 (Bookworm) ou Ubuntu 24.04 LTS
- Accès `root` ou `sudo`
- **Valkey** (ou Redis) installé et en fonctionnement
- Espace disque suffisant pour les sauvegardes

## 1. Configurer le dépôt Gitea

Ajoutez le dépôt Debian de Woodstock :

```bash
# Ajouter la clé GPG
curl -fsSL https://gogs.shadoware.org/api/packages/ShadowareOrg/debian/repository.key \
  | gpg --dearmor -o /usr/share/keyrings/woodstock-archive-keyring.gpg

# Ajouter le dépôt
echo "deb [signed-by=/usr/share/keyrings/woodstock-archive-keyring.gpg] \
  https://gogs.shadoware.org/api/packages/ShadowareOrg/debian bookworm main" \
  | tee /etc/apt/sources.list.d/woodstock.list

apt-get update
```

## 2. Installer le serveur

```bash
apt-get install woodstock-server
```

Cela installe :

- Les 4 binaires serveur : `api_server`, `client_api_server`, `job_worker`, `scheduler`
- L'interface web Vue.js dans `/usr/share/woodstock/static/`
- 4 services systemd + 1 target `woodstock.target`
- Le fichier de configuration `/etc/woodstock/server.env`
- L'utilisateur système `woodstock` (UID dédié, non-root)
- Le répertoire de données `/var/lib/woodstock/` avec la structure complète

## 3. Installer Valkey (si nécessaire)

Le serveur Woodstock nécessite **Valkey** (compatible Redis) pour la file de jobs et les verrous distribués :

```bash
# Sur Debian Bookworm
apt-get install valkey
systemctl enable --now valkey-server
```

Ou avec Redis :

```bash
apt-get install redis-server
systemctl enable --now redis-server
```

## 4. Configurer le serveur

Éditez le fichier d'environnement :

```bash
nano /etc/woodstock/server.env
```

Les paramètres essentiels :

```ini
# Chemin des données de sauvegarde (doit avoir assez d'espace)
BACKUP_PATH=/var/lib/woodstock

# Connexion Redis/Valkey
REDIS_HOST=localhost
REDIS_PORT=6379

# Interface web pre-compilée
STATIC_PATH=/usr/share/woodstock/static

# Niveau de log : error, warn, info, debug
LOG_LEVEL=info

# Port de l'API REST (défaut : 3000)
MANAGEMENT_API_PORT=3000
```

::: tip Espace disque
Si vos données de sauvegarde doivent résider sur un disque ou une partition différents, montez-la sur `/var/lib/woodstock` ou modifiez `BACKUP_PATH` pour pointer vers le chemin souhaité.
:::

## 5. Démarrer les services

Woodstock fournit une **target systemd** unique qui démarre les 4 services en une commande :

```bash
# Activer et démarrer tous les services
systemctl enable --now woodstock.target

# Vérifier l'état
systemctl status woodstock-api woodstock-client-api woodstock-worker woodstock-scheduler
```

Les 4 services peuvent également être gérés individuellement :

| Service | Rôle | Port |
|---------|------|------|
| `woodstock-api` | API REST + interface web | 3000 |
| `woodstock-client-api` | Passerelle mTLS pour les agents | 8443 |
| `woodstock-worker` | Worker de backup/restauration | — |
| `woodstock-scheduler` | Planificateur CRON | — |

## 6. Vérifier l'installation

```bash
# Accéder à l'interface web
curl http://localhost:3000/

# Vérifier les logs
journalctl -u woodstock-api -f
journalctl -u woodstock-worker -f
```

L'interface web est disponible sur `http://<adresse-du-serveur>:3000`.

## 7. Installer le client sur les machines à sauvegarder

```bash
apt-get install woodstock-client
```

Puis configurez le client :

```bash
nano /etc/woodstock/config.yml
systemctl enable --now ws_client_daemon
```

Voir le [guide de configuration de l'agent](/doc/agent) pour les détails.

## Pare-feu

Ouvrir les ports nécessaires sur le serveur :

```bash
# Interface web (HTTPS recommandé via reverse proxy)
ufw allow 3000/tcp

# Passerelle mTLS pour les agents (obligatoire)
ufw allow 8443/tcp
```

## Mise à jour

```bash
apt-get update && apt-get upgrade woodstock-server
```

Les services sont automatiquement redémarrés après la mise à jour.

## Désinstallation

```bash
# Supprimer le paquet (conserve les données)
apt-get remove woodstock-server

# Supprimer le paquet ET toutes les données (/var/lib/woodstock)
apt-get purge woodstock-server
```

::: warning
`apt-get purge` supprime définitivement toutes les sauvegardes stockées dans `/var/lib/woodstock`. Assurez-vous d'avoir une copie de vos données avant d'utiliser cette commande.
:::

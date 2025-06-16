#!/bin/bash

# Chemin vers le dossier de backups
BACKUP_PATH="${BACKUP_PATH:-/var/lib/woodstock/backups}/hosts"

# Couleurs pour les messages
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Recherche des fichiers backup.yml dans $BACKUP_PATH...${NC}"

# Vérifier que le dossier BACKUP_PATH existe
if [ ! -d "$BACKUP_PATH" ]; then
    echo -e "${RED}Erreur: Le dossier $BACKUP_PATH n'existe pas${NC}"
    echo "Utilisez la variable d'environnement BACKUP_PATH pour définir l'emplacement des backups."
    echo "Exemple: BACKUP_PATH=/chemin/vers/backups $0"
    exit 1
fi

# Trouver tous les fichiers backup.yml dans BACKUP_PATH
find "$BACKUP_PATH" -name "backup.yml" -type f | while read -r file; do
    echo -e "${GREEN}Traitement de $file...${NC}"
    
    # Créer un fichier temporaire
    temp_file=$(mktemp)
    

    # Version utilisant sed (plus basique mais sans dépendances externes)
    # Cette approche est plus risquée car basée sur du pattern matching simple
    
    # 1. Remplacer les lignes où completed: true par status: Completed
    sed 's/completed: true/status: Completed/g' "$file" > "$temp_file"
    
    # 2. Remplacer les lignes où completed: false par status: Aborted
    sed -i 's/completed: false/status: Aborted/g' "$temp_file"
    
    # Vérifier que le fichier temporaire n'est pas vide avant de remplacer
    if [ -s "$temp_file" ]; then
        # Garder une sauvegarde du fichier original
        cp "$file" "${file}.bak"
        
        # Remplacer l'original par la version modifiée
        mv "$temp_file" "$file"
        echo -e "${GREEN}✓ Fichier $file traité avec succès${NC}"
    else
        echo -e "${RED}✗ Erreur lors du traitement de $file - fichier temporaire vide${NC}"
        rm "$temp_file"
    fi
done

# Afficher les statistiques
backup_count=$(find "$BACKUP_PATH" -name "backup.yml" | wc -l)
echo -e "${GREEN}Terminé! $backup_count fichiers backup.yml traités${NC}"
#!/bin/sh
set -e

# Usage: ./generate_data.sh <target_dir> <size_in_mb>

TARGET_DIR=${1:-/tmp/source}
TOTAL_SIZE_MB=${2:-1024} # Default 1GB

echo "--- Generating Test Data in $TARGET_DIR (Target: ~${TOTAL_SIZE_MB}MB) ---"

mkdir -p "$TARGET_DIR"

# 1. Structure arborescente (Simule un Home utilisateur)
mkdir -p "$TARGET_DIR/Documents/Projects/ProjectA"
mkdir -p "$TARGET_DIR/Documents/Projects/ProjectB"
mkdir -p "$TARGET_DIR/Photos/2023/Vacation"
mkdir -p "$TARGET_DIR/Photos/2024/Party"
mkdir -p "$TARGET_DIR/Downloads/Cache"
mkdir -p "$TARGET_DIR/.hidden_conf"

# 2. Génération de fichiers volumineux (Données aléatoires - Mauvais pour la dédup)
# On divise la taille cible par 2 pour les données aléatoires
RANDOM_SIZE=$((TOTAL_SIZE_MB / 2))
echo "Generating $RANDOM_SIZE MB of random binary data..."

# Un gros fichier binaire (ex: simule une video)
dd if=/dev/urandom of="$TARGET_DIR/Photos/2024/Party/video_raw.mp4" bs=1M count=$((RANDOM_SIZE / 2)) status=none
echo "- Created: Photos/2024/Party/video_raw.mp4"

# Plusieurs fichiers moyens répartis
dd if=/dev/urandom of="$TARGET_DIR/Documents/Projects/ProjectA/data.bin" bs=1M count=$((RANDOM_SIZE / 4)) status=none
dd if=/dev/urandom of="$TARGET_DIR/Documents/Projects/ProjectB/assets.pak" bs=1M count=$((RANDOM_SIZE / 4)) status=none

# 3. Génération de fichiers textuels/répétitifs (Bon pour la dédup/compression)
# On utilise yes pour générer du texte répétitif rapidement
echo "Generating repetitive data (good for dedup testing)..."

yes "Log entry line for testing purpose throughout the file." | head -n 100000 > "$TARGET_DIR/Documents/Projects/ProjectA/app.log"
cp "$TARGET_DIR/Documents/Projects/ProjectA/app.log" "$TARGET_DIR/Documents/Projects/ProjectB/app.log" # Duplication exacte
echo "- Created & Duplicated: app.log"

# 4. Fichiers à EXCLURE (Patterns communs)
echo "Generating files to be EXCLUDED..."

# Fichiers temporaires (*.tmp)
dd if=/dev/urandom of="$TARGET_DIR/Downloads/Cache/temp_download.tmp" bs=1M count=10 status=none
touch "$TARGET_DIR/Documents/Projects/ProjectA/scratch.tmp"

# Fichiers caches (*.cache, ou dossiers specifiques)
dd if=/dev/zero of="$TARGET_DIR/Downloads/Cache/browser.cache" bs=1M count=50 status=none

# Fichier Node modules (souvent exclus)
mkdir -p "$TARGET_DIR/Documents/Projects/ProjectA/node_modules/library"
echo "fake code" > "$TARGET_DIR/Documents/Projects/ProjectA/node_modules/library/index.js"

# 5. Petits fichiers (pour tester la latence/overhead)
echo "Generating 100 small text files..."
for i in $(seq 1 100); do
    echo "Content for file $i" > "$TARGET_DIR/Documents/small_file_$i.txt"
done

echo "--- Data Generation Complete ---"
du -sh "$TARGET_DIR"
find "$TARGET_DIR" | head -n 20
echo "..."

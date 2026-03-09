#!/bin/bash

# Migration script: Transform BackupStatus from old string format to new adjacently tagged format
# Old format: status: Completed
# New format: status:
#               status: Completed
# For variants with data (Finishing, Aborting, Failed, Removing), adds a default details field

# Path to backups folder
BACKUP_PATH="${BACKUP_PATH:-/var/lib/woodstock/backups}/hosts"

# Colors for messages
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== BackupStatus Format Migration ===${NC}"
echo -e "${YELLOW}Searching for backup.yml files in $BACKUP_PATH...${NC}"

# Check if BACKUP_PATH exists
if [ ! -d "$BACKUP_PATH" ]; then
    echo -e "${RED}Error: Directory $BACKUP_PATH does not exist${NC}"
    echo "Use the BACKUP_PATH environment variable to set the backup location."
    echo "Example: BACKUP_PATH=/path/to/backups $0"
    exit 1
fi

# Counter for statistics
total_files=0
migrated_files=0
skipped_files=0
error_files=0

# Find all backup.yml files in BACKUP_PATH
find "$BACKUP_PATH" -name "backup.yml" -type f | while read -r file; do
    total_files=$((total_files + 1))
    echo -e "${YELLOW}Processing $file...${NC}"
    
    # Check if file contains old format (simple string status)
    if ! grep -qE '^\s+status:\s+(Completed|Aborted|InProgress|Finishing|Aborting|Failed|Removing)\s*$' "$file"; then
        echo -e "${BLUE}  ↳ Skipped (already migrated or empty)${NC}"
        skipped_files=$((skipped_files + 1))
        continue
    fi
    
    # Create backup
    cp "$file" "${file}.bak"
    
    # Create temporary file
    temp_file=$(mktemp)
    
    # Use awk to transform the format properly
    # Transform adjacently-tagged enum format:
    #   OLD: "  status: Completed"
    #   NEW: "  status:\n    status: Completed"
    # For variants with data:
    #   OLD: "  status: Finishing"
    #   NEW: "  status:\n    status: Finishing\n    details: ToCompact"
    awk '
    /^([[:space:]]+)status:[[:space:]]+(Completed|Aborted|InProgress)[[:space:]]*$/ {
        match($0, /^([[:space:]]+)/, indent)
        spaces = substr($0, 1, RLENGTH)
        status = $2
        print spaces "status:"
        print spaces "  status: " status
        next
    }
    /^([[:space:]]+)status:[[:space:]]+Finishing[[:space:]]*$/ {
        match($0, /^([[:space:]]+)/, indent)
        spaces = substr($0, 1, RLENGTH)
        print spaces "status:"
        print spaces "  status: Finishing"
        print spaces "  details: ToCompact"
        next
    }
    /^([[:space:]]+)status:[[:space:]]+Aborting[[:space:]]*$/ {
        match($0, /^([[:space:]]+)/, indent)
        spaces = substr($0, 1, RLENGTH)
        print spaces "status:"
        print spaces "  status: Aborting"
        print spaces "  details: ToCompact"
        next
    }
    /^([[:space:]]+)status:[[:space:]]+Failed[[:space:]]*$/ {
        match($0, /^([[:space:]]+)/, indent)
        spaces = substr($0, 1, RLENGTH)
        print spaces "status:"
        print spaces "  status: Failed"
        print spaces "  details: Compact"
        next
    }
    /^([[:space:]]+)status:[[:space:]]+Removing[[:space:]]*$/ {
        match($0, /^([[:space:]]+)/, indent)
        spaces = substr($0, 1, RLENGTH)
        print spaces "status:"
        print spaces "  status: Removing"
        print spaces "  details: ToRemoveInPool"
        next
    }
    { print }
    ' "$file" > "$temp_file"
    
    # Verify that the temp file is not empty
    if [ -s "$temp_file" ]; then
        # Replace original with migrated version
        mv "$temp_file" "$file"
        echo -e "${GREEN}  ✓ File migrated successfully${NC}"
        migrated_files=$((migrated_files + 1))
    else
        echo -e "${RED}  ✗ Error during migration - keeping original${NC}"
        rm "$temp_file"
        # Restore from backup
        mv "${file}.bak" "$file"
        error_files=$((error_files + 1))
    fi
done

# Display statistics
echo ""
echo -e "${BLUE}=== Migration Complete ===${NC}"
echo -e "Total files found: $(find "$BACKUP_PATH" -name "backup.yml" | wc -l)"
echo -e "${GREEN}Successfully migrated files${NC}"
echo ""
echo -e "${YELLOW}Note: Backup files (.bak) have been created for all modified files${NC}"

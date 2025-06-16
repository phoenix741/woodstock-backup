#!/bin/bash

# Parameter verification
if [ $# -eq 0 ]; then
    echo "Usage: $0 'command to execute'"
    exit 1
fi

# Configuration
ORIGINAL_BACKUP_PATH="${BACKUP_PATH:-/var/lib/woodstock}"
TEMP_ID=$(date +%Y%m%d_%H%M%S_$$)
TEMP_BACKUP_DIR="${ORIGINAL_BACKUP_PATH}/.tmp"
TEMP_BACKUP_PATH="${TEMP_BACKUP_DIR}/${TEMP_ID}"

# Cleanup function to be called at the end or on interruption
cleanup() {
    echo "Cleaning up resources..."
    
    # Restore the original BACKUP_PATH
    export BACKUP_PATH="$ORIGINAL_BACKUP_PATH"
    echo "BACKUP_PATH restored to: $BACKUP_PATH"
    
    # Remove the temporary btrfs subvolume
    if [ -d "$TEMP_BACKUP_PATH" ]; then
        echo "Removing temporary btrfs subvolume..."
        sudo btrfs subvolume delete "$TEMP_BACKUP_PATH"
    fi
    
    # Remove the .tmp directory if it's empty
    if [ -d "$TEMP_BACKUP_DIR" ] && [ -z "$(ls -A "$TEMP_BACKUP_DIR")" ]; then
        rmdir "$TEMP_BACKUP_DIR"
    fi
    
    echo "Cleanup completed"
}

# Intercept interrupt signals to ensure cleanup
trap cleanup EXIT SIGINT SIGTERM

# Check dependencies
if ! command -v btrfs &> /dev/null; then
    echo "Btrfs tools are not installed. Please install them."
    exit 1
fi

echo "Preparing temporary btrfs subvolume..."

# Create .tmp directory if it doesn't exist
mkdir -p "$TEMP_BACKUP_DIR"

# Create a btrfs subvolume as snapshot
echo "Creating btrfs snapshot from $ORIGINAL_BACKUP_PATH to $TEMP_BACKUP_PATH"
btrfs subvolume snapshot "$ORIGINAL_BACKUP_PATH" "$TEMP_BACKUP_PATH"
if [ $? -ne 0 ]; then
    echo "Error creating btrfs snapshot"
    exit 1
fi

# Set the new BACKUP_PATH to the snapshot
export BACKUP_PATH="$TEMP_BACKUP_PATH"
echo "Temporary BACKUP_PATH: $BACKUP_PATH"

# Execute the provided command
echo "Executing command: $@"
eval "$@"
CMD_EXIT_CODE=$?

# Pause before exit
read -p "Press Enter to exit..." _

echo "Command completed with exit code: $CMD_EXIT_CODE"

# Cleanup will be performed automatically thanks to the trap

exit $CMD_EXIT_CODE

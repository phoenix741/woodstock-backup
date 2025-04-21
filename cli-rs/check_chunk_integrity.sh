#!/bin/bash

# Woodstock Backup - Chunk Integrity Checker
# This script verifies that every .info file has its corresponding .zz chunk file
# in the backup pool directory.

set -euo pipefail

# Configuration
BACKUP_PATH="${BACKUP_PATH:-/var/lib/woodstock}"
MISSING_FILES_TEMP=$(mktemp)
ERROR_COUNT=0
DELETE_MODE=false

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --delete)
            DELETE_MODE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--delete] [--help]"
            echo ""
            echo "Options:"
            echo "  --delete    Delete .info files that don't have corresponding .zz files"
            echo "  --help      Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  BACKUP_PATH Directory to check (default: /var/lib/woodstock)"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    rm -f "$MISSING_FILES_TEMP"
}
trap cleanup EXIT

# Function to print colored output
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if backup path exists
if [[ ! -d "$BACKUP_PATH" ]]; then
    log_error "Backup path does not exist: $BACKUP_PATH"
    exit 1
fi

log_info "Checking chunk integrity in: $BACKUP_PATH"
if [[ "$DELETE_MODE" == true ]]; then
    log_warning "DELETE MODE ENABLED - Orphaned .info files will be removed"
fi
log_info "Starting verification process..."

# Find all .info files and check for corresponding .zz files
# Using process substitution to avoid subshell issues
while IFS= read -r -d '' info_file; do
    # Get the base name without the .info extension
    base_file="${info_file%.info}"
    zz_file="${base_file}.zz"
    
    # Check if the corresponding .zz file exists
    if [[ ! -f "$zz_file" ]]; then
        echo "$info_file" >> "$MISSING_FILES_TEMP"
    fi
done < <(find "$BACKUP_PATH" -name "*.info" -type f -print0)

# Count total missing files first
if [[ -f "$MISSING_FILES_TEMP" && -s "$MISSING_FILES_TEMP" ]]; then
    ERROR_COUNT=$(wc -l < "$MISSING_FILES_TEMP")
    
    log_error "Missing .zz files for the following .info files:"
    echo
    
    # Display missing files with relative paths for better readability
    while IFS= read -r missing_file; do
        relative_path="${missing_file#$BACKUP_PATH/}"
        echo "  - $relative_path"
    done < "$MISSING_FILES_TEMP"
    
    echo
    log_error "Total files with missing .zz chunks: $ERROR_COUNT"
    
    # Delete orphaned .info files if requested
    if [[ "$DELETE_MODE" == true ]]; then
        echo
        log_warning "Deleting orphaned .info files..."
        
        deleted_count=0
        failed_count=0
        
        # Temporarily disable strict error handling for deletion loop
        set +e
        
        while IFS= read -r missing_file; do
            # Ensure we have the absolute path
            if [[ "$missing_file" != /* ]]; then
                missing_file="$BACKUP_PATH/$missing_file"
            fi
            
            # Verify file exists before attempting deletion
            if [[ -f "$missing_file" ]]; then
                # Attempt deletion and capture exit code
                rm "$missing_file"
                rm_exit_code=$?
                
                if [[ $rm_exit_code -eq 0 ]]; then
                    ((deleted_count++))
                    log_info "Successfully deleted: ${missing_file#$BACKUP_PATH/}"
                else
                    ((failed_count++))
                    log_error "Failed to delete (exit code $rm_exit_code): ${missing_file#$BACKUP_PATH/}"
                fi
            else
                ((failed_count++))
                log_error "File not found (skipping): ${missing_file#$BACKUP_PATH/}"
            fi
        done < "$MISSING_FILES_TEMP"
        
        # Re-enable strict error handling
        set -e
        
        echo
        log_info "Deletion summary:"
        log_info "  - Successfully deleted: $deleted_count files"
        if [[ $failed_count -gt 0 ]]; then
            log_error "  - Failed to delete: $failed_count files"
        fi
        exit 0
    else
        echo
        log_info "Use --delete option to remove these orphaned .info files"
        exit 1
    fi
else
    log_info "All .info files have their corresponding .zz chunk files"
    log_info "Chunk integrity verification completed successfully"
    echo
    log_info "Total errors found: 0"
fi

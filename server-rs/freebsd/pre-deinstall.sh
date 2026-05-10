#!/bin/sh
# FreeBSD pre-deinstall script for woodstock-server

# Stop services if running
for svc in woodstock_api woodstock_client_api woodstock_worker woodstock_scheduler; do
    if service "${svc}" status > /dev/null 2>&1; then
        echo "Stopping ${svc} service..."
        service "${svc}" stop || true
    fi
done

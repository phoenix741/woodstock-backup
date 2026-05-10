#!/bin/sh
# FreeBSD pre-deinstall script for woodstock-client

# Stop the service if running
if service woodstock_client status > /dev/null 2>&1; then
    echo "Stopping woodstock_client service..."
    service woodstock_client stop || true
fi

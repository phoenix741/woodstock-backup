#!/bin/sh
# FreeBSD post-install script for woodstock-server

# Create woodstock group if it doesn't exist
if ! pw group show woodstock > /dev/null 2>&1; then
    pw groupadd woodstock -g 565
fi

# Create woodstock user if it doesn't exist
if ! pw user show woodstock > /dev/null 2>&1; then
    pw useradd woodstock \
        -u 565 \
        -g woodstock \
        -d /var/db/woodstock \
        -s /usr/sbin/nologin \
        -c "Woodstock Backup Server" \
        -G woodstock
fi

# Create data directories
for dir in /var/db/woodstock \
           /var/db/woodstock/certs \
           /var/db/woodstock/config \
           /var/db/woodstock/hosts \
           /var/db/woodstock/logs \
           /var/db/woodstock/pool \
           /var/db/woodstock/events \
           /var/db/woodstock/jobs; do
    if [ ! -d "${dir}" ]; then
        mkdir -p "${dir}"
    fi
    chown woodstock:woodstock "${dir}"
done

chmod 750 /var/db/woodstock
chmod 750 /var/db/woodstock/certs

# Create log directory
if [ ! -d /var/log/woodstock ]; then
    mkdir -p /var/log/woodstock
    chown woodstock:woodstock /var/log/woodstock
    chmod 750 /var/log/woodstock
fi

# Protect env file
if [ -f /usr/local/etc/woodstock/server.env ]; then
    chown root:woodstock /usr/local/etc/woodstock/server.env
    chmod 640 /usr/local/etc/woodstock/server.env
fi

echo ""
echo "===================================================================="
echo " Woodstock Backup Server installed successfully."
echo ""
echo " 1. Install and start Valkey (Redis-compatible):"
echo "    pkg install databases/valkey"
echo "    echo 'valkey_enable=\"YES\"' >> /etc/rc.conf"
echo "    service valkey start"
echo ""
echo " 2. Edit server configuration:"
echo "    vi /usr/local/etc/woodstock/server.env"
echo ""
echo " 3. Enable and start Woodstock services:"
echo "    echo 'woodstock_worker_enable=\"YES\"' >> /etc/rc.conf"
echo "    echo 'woodstock_scheduler_enable=\"YES\"' >> /etc/rc.conf"
echo "    echo 'woodstock_api_enable=\"YES\"' >> /etc/rc.conf"
echo "    echo 'woodstock_client_api_enable=\"YES\"' >> /etc/rc.conf"
echo "    service woodstock_worker start"
echo "    service woodstock_scheduler start"
echo "    service woodstock_api start"
echo "    service woodstock_client_api start"
echo ""
echo " API available at: http://localhost:3000"
echo " Agent gateway: https://localhost:8443 (mTLS)"
echo "===================================================================="

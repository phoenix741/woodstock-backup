#!/bin/sh
# FreeBSD post-install script for woodstock-server

# Create woodstock group if it doesn't exist. GID 565 is our preferred, fixed
# value (mirrors how official FreeBSD ports pin low system GIDs), but since
# this package isn't registered in the ports tree's GIDs file it isn't
# guaranteed to be free, so fall back to a system-assigned GID if taken.
if ! pw group show woodstock > /dev/null 2>&1; then
    pw groupadd woodstock -g 565 2>/dev/null || pw groupadd woodstock
fi

# Create woodstock user if it doesn't exist. Same fixed-UID-with-fallback
# strategy as above.
if ! pw user show woodstock > /dev/null 2>&1; then
    pw useradd woodstock \
        -u 565 \
        -g woodstock \
        -d /var/db/woodstock \
        -s /usr/sbin/nologin \
        -c "Woodstock Backup Server" \
        -G woodstock 2>/dev/null || \
    pw useradd woodstock \
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
#
# The ownership fix has to run unconditionally: /var/log/woodstock may already
# exist, created as root:wheel by the client package (or by the FreeBSD package
# machinery from the `directories` manifest key). Applying it only on creation
# left the server processes, which run as `woodstock`, unable to open their log
# file — and tracing-appender panics on that, so every service died at startup.
mkdir -p /var/log/woodstock
chown woodstock:woodstock /var/log/woodstock
chmod 750 /var/log/woodstock

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

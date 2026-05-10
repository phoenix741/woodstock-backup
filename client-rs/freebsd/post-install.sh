#!/bin/sh
# FreeBSD post-install script for woodstock-client

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
        -c "Woodstock Backup" \
        -G woodstock
fi

# Create data directories
for dir in /var/db/woodstock; do
    if [ ! -d "${dir}" ]; then
        mkdir -p "${dir}"
    fi
    chown woodstock:woodstock "${dir}"
    chmod 750 "${dir}"
done

# Create log directory
if [ ! -d /var/log/woodstock ]; then
    mkdir -p /var/log/woodstock
    chown woodstock:woodstock /var/log/woodstock
    chmod 750 /var/log/woodstock
fi

# Install config sample if no config present
if [ ! -f /usr/local/etc/woodstock/config.yaml ]; then
    mkdir -p /usr/local/etc/woodstock
    if [ -f /usr/local/etc/woodstock/config.yaml.sample ]; then
        cp /usr/local/etc/woodstock/config.yaml.sample /usr/local/etc/woodstock/config.yaml
        chmod 640 /usr/local/etc/woodstock/config.yaml
        chown root:woodstock /usr/local/etc/woodstock/config.yaml
    fi
fi

echo ""
echo "===================================================================="
echo " Woodstock Client installed successfully."
echo " Edit /usr/local/etc/woodstock/config.yaml to configure the agent."
echo " Then enable and start the service:"
echo "   echo 'woodstock_client_enable=\"YES\"' >> /etc/rc.conf"
echo "   service woodstock_client start"
echo "===================================================================="

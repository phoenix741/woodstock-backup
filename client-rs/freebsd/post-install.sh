#!/bin/sh
# FreeBSD post-install script for woodstock-client
#
# The client agent runs as root (see woodstock_client.rc): it needs to read
# arbitrary user/system files and take filesystem snapshots, so unlike the
# server it has no dedicated unprivileged system user.

# Create data directories
for dir in /var/db/woodstock /var/log/woodstock; do
    if [ ! -d "${dir}" ]; then
        mkdir -p "${dir}"
    fi
done

# Install config sample if no config present
if [ ! -f /usr/local/etc/woodstock/config.yaml ]; then
    mkdir -p /usr/local/etc/woodstock
    if [ -f /usr/local/etc/woodstock/config.yaml.sample ]; then
        cp /usr/local/etc/woodstock/config.yaml.sample /usr/local/etc/woodstock/config.yaml
        chmod 600 /usr/local/etc/woodstock/config.yaml
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

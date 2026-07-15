#!/bin/bash
# Switch forwarding backend from Realm to nftables-nat-rust.
# The shared rules file /etc/nat.conf is always kept.

set -e

REPO="Taylor000/nftables-nat-rust"
BRANCH="${BRANCH:-master}"
RAW_BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}"
RULES_FILE="/etc/nat.conf"

if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

realm_service_exists() {
    systemctl list-unit-files realm.service --no-legend 2>/dev/null | grep -q '^realm\.service' ||
        systemctl status realm >/dev/null 2>&1
}

backup_rules() {
    if [ -f "$RULES_FILE" ]; then
        backup_file="${RULES_FILE}.switch-to-nft.$(date +%Y%m%d%H%M%S).bak"
        cp -p "$RULES_FILE" "$backup_file"
        echo "Rules backup: ${backup_file}"
    fi
}

disable_realm_if_present() {
    if ! realm_service_exists; then
        return
    fi

    echo "Disabling realm.service. ${RULES_FILE} will be kept."
    systemctl stop realm 2>/dev/null || true
    systemctl disable realm 2>/dev/null || true
}

backup_rules
disable_realm_if_present

tmp_setup="/tmp/nftables-nat-setup.$$"
curl -fsSL "${RAW_BASE_URL}/setup.sh" -o "$tmp_setup"
bash "$tmp_setup" simple
rm -f "$tmp_setup"

echo ""
echo "Switched to nftables-nat-rust."
echo "Rules file kept: ${RULES_FILE}"
echo "Current backend: forward-status"
echo "Status: systemctl status nat"

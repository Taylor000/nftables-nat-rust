#!/bin/bash
# Show current forwarding backend status.

set -u

service_active() {
    systemctl is-active --quiet "$1" 2>/dev/null
}

service_enabled() {
    systemctl is-enabled --quiet "$1" 2>/dev/null
}

state_line() {
    service_name="$1"
    label="$2"

    active="inactive"
    enabled="disabled"
    service_active "$service_name" && active="active"
    service_enabled "$service_name" && enabled="enabled"

    printf '%-8s service=%-13s boot=%s\n' "$label" "$active" "$enabled"
}

realm_active=0
nat_active=0
service_active realm && realm_active=1
service_active nat && nat_active=1

if [ "$realm_active" -eq 1 ] && [ "$nat_active" -eq 1 ]; then
    echo "Current backend: CONFLICT (realm and nft are both active)"
elif [ "$realm_active" -eq 1 ]; then
    echo "Current backend: Realm"
elif [ "$nat_active" -eq 1 ]; then
    echo "Current backend: nft"
else
    echo "Current backend: none"
fi

state_line realm Realm
state_line nat nft

echo "Rules file: /etc/nat.conf"

#!/bin/bash
#
# Title:        Transparent Proxy iptables Redirector
# Description:  A script to transparently redirect outgoing traffic for a
#               specific destination to a local proxy, while intelligently
#               avoiding the common infinite redirection loop.
#               Handles both IPv4 and IPv6 traffic correctly.
# Author:       Gemini
#
# How it works:
# The infinite loop occurs when the proxy, in forwarding traffic to the
# original destination, has its own traffic redirected back to itself.
# This script solves this by creating a dedicated system user for the proxy.
# The iptables/ip6tables rule is then crafted to redirect all traffic
# EXCEPT traffic originating from this dedicated user, thus breaking the loop.

# --- Configuration ---
# The user that your proxy will run as. This is CRITICAL to prevent loops.
# The script will create this user if it doesn't exist.
readonly PROXY_USER="proxy-injector"

# The local port your proxy is listening on.
readonly PROXY_PORT="8080"

# The destination host you want to intercept traffic for.
readonly TARGET_HOST="rlgncook.speedtest.sbcglobal.net"

# The destination port you want to intercept.
readonly TARGET_PORT="8080"
# --- End Configuration ---


# Global array to store the full rule commands we add, so we can cleanly remove them later.
declare -a ADDED_RULES
# Store original ip_forward value
ORIGINAL_IP_FORWARD=""


# --- Main Functions ---

#
# Function to be called on script exit (e.g., via Ctrl+C) to clean up.
#
cleanup() {
    echo -e "\n[*] Cleaning up iptables and ip6tables rules..."
    tput sgr0 # Reset text formatting

    # Revert ip_forwarding if we changed it
    if [[ -n "$ORIGINAL_IP_FORWARD" && $(cat /proc/sys/net/ipv4/ip_forward) != "$ORIGINAL_IP_FORWARD" ]]; then
        echo "[*] Restoring original net.ipv4.ip_forward value to '$ORIGINAL_IP_FORWARD'"
        sysctl -w net.ipv4.ip_forward="$ORIGINAL_IP_FORWARD" >/dev/null
    fi

    # Remove the rules in reverse order of their addition for clean teardown.
    # The array stores the full command ('iptables ...' or 'ip6tables ...').
    for ((i=${#ADDED_RULES[@]}-1; i>=0; i--)); do
        local full_rule_command="${ADDED_RULES[i]}"
        # Replace the '-A' (Append) with '-D' (Delete) to create the delete command.
        local delete_command="${full_rule_command/-A/-D}"
        echo "[-] Removing rule: $delete_command"
        eval "$delete_command" 2>/dev/null
    done

    echo "[+] Cleanup complete."
    exit 0
}


#
# Function to set up the environment and iptables rules.
#
setup_rules() {
    # Check for root privileges, as they are required for iptables and user management.
    if [[ $EUID -ne 0 ]]; then
       echo "[!] This script must be run as root. Aborting." >&2
       exit 1
    fi

    # Set the trap. This ensures that the 'cleanup' function is called when the
    # script receives an INT (Ctrl+C), TERM, or EXIT signal.
    trap cleanup INT TERM EXIT

    # --- User Setup ---
    if ! id -u "$PROXY_USER" >/dev/null 2>&1; then
        echo "[*] Proxy user '$PROXY_USER' not found. Creating it..."
        useradd --system --shell /usr/sbin/nologin "$PROXY_USER"
        if [ $? -ne 0 ]; then
            echo "[!] Failed to create user '$PROXY_USER'. Aborting." >&2
            exit 1
        fi
        echo "[+] User '$PROXY_USER' created successfully."
    else
        echo "[*] Found existing proxy user '$PROXY_USER'."
    fi
    local proxy_uid
    proxy_uid=$(id -u "$PROXY_USER")

    # --- DNS Resolution (IPv4 and IPv6) ---
    echo "[*] Resolving host: $TARGET_HOST..."
    local ipv4_ips ipv6_ips
    ipv4_ips=($(getent ahostsv4 "$TARGET_HOST" | awk '{ print $1 }' | sort -u))
    ipv6_ips=($(getent ahostsv6 "$TARGET_HOST" | awk '{ print $1 }' | sort -u))

    if [ ${#ipv4_ips[@]} -eq 0 ] && [ ${#ipv6_ips[@]} -eq 0 ]; then
        echo "[!] Could not resolve '$TARGET_HOST'. Please check the hostname and your DNS settings. Aborting." >&2
        exit 1
    fi
    [[ ${#ipv4_ips[@]} -gt 0 ]] && echo "[+] '$TARGET_HOST' resolved to IPv4: ${ipv4_ips[*]}"
    [[ ${#ipv6_ips[@]} -gt 0 ]] && echo "[+] '$TARGET_HOST' resolved to IPv6: ${ipv6_ips[*]}"


    # --- IP Forwarding ---
    ORIGINAL_IP_FORWARD=$(cat /proc/sys/net/ipv4/ip_forward)
    if [[ "$ORIGINAL_IP_FORWARD" != "1" ]]; then
        echo "[*] Enabling IP forwarding (net.ipv4.ip_forward=1)..."
        sysctl -w net.ipv4.ip_forward=1 >/dev/null
    fi

    # --- Add iptables Rules ---
    echo "[*] Adding iptables rules..."

    # Add rules for IPv4 addresses
    for ip in "${ipv4_ips[@]}"; do
        # CORRECTED SYNTAX: The '!' now comes *after* '-m owner'
        local rule_args="-t nat -A OUTPUT -p tcp -d $ip --dport $TARGET_PORT -m owner ! --uid-owner $proxy_uid -j REDIRECT --to-port $PROXY_PORT"
        local full_command="iptables $rule_args"
        echo "[+] Adding IPv4 rule: $full_command"
        eval "$full_command"
        if [ $? -ne 0 ]; then
            echo "[!] Failed to add IPv4 rule for IP $ip. Aborting." >&2
            exit 1
        fi
        ADDED_RULES+=("$full_command")
    done

    # Add rules for IPv6 addresses
    for ip in "${ipv6_ips[@]}"; do
        local rule_args="-t nat -A OUTPUT -p tcp -d $ip --dport $TARGET_PORT -m owner ! --uid-owner $proxy_uid -j REDIRECT --to-port $PROXY_PORT"
        local full_command="ip6tables $rule_args"
        echo "[+] Adding IPv6 rule: $full_command"
        eval "$full_command"
        if [ $? -ne 0 ]; then
            echo "[!] Failed to add IPv6 rule for IP $ip. Aborting." >&2
            exit 1
        fi
        ADDED_RULES+=("$full_command")
    done


    echo
    echo "---------------------------------------------------------------------"
    tput bold; echo "[SUCCESS] IPTables rules are now active."; tput sgr0
    echo
    echo "  Your proxy should be run as the '$PROXY_USER' user to avoid loops."
    echo "  Example command:"
    tput setaf 2; echo "    sudo -u $PROXY_USER /path/to/your/proxy-binary"; tput sgr0
    echo
    echo "  Press CTRL+C in this terminal to stop and cleanly remove all rules."
    echo "---------------------------------------------------------------------"
}


# --- Script Entry Point ---
main() {
    setup_rules

    # Keep the script running indefinitely. The `trap` will handle the exit.
    while true; do
        sleep 86400 # Sleep for a day; the actual wakeup time doesn't matter.
    done
}

main "$@"
# End of script
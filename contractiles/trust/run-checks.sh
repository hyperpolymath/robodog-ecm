#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
# run-checks.sh — Execute all Trustfile operational checks
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# Parses the [OPERATIONAL_CHECKS] section of Trustfile.a2ml and runs each
# check sequentially. Exits non-zero if any critical check fails.

TRUSTFILE="$(dirname "$0")/Trustfile.a2ml"
FAILED=0
CRITICAL_FAILED=0
PASSED=0

# Colours (if terminal supports them)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' CYAN='' BOLD='' NC=''
fi

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  Robodog ECM — Trustfile Operational Checks                 ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Extract checks from the [OPERATIONAL_CHECKS] section.
# We use awk to pull out structured blocks: name, description, run, severity.
in_section=0
current_name=""
current_desc=""
current_run=""
current_severity="warning"
current_section_label=""

run_check() {
    local name="$1"
    local desc="$2"
    local cmd="$3"
    local sev="$4"

    if [[ -z "$name" || -z "$cmd" ]]; then
        return
    fi

    # Strip outer quotes from run command
    cmd="${cmd#\"}"
    cmd="${cmd%\"}"

    printf "  %-42s " "$name"

    if eval "$cmd" >/dev/null 2>&1; then
        echo -e "[${GREEN}PASS${NC}]"
        PASSED=$((PASSED + 1))
    else
        if [[ "$sev" == "critical" ]]; then
            echo -e "[${RED}FAIL${NC}] (critical)"
            CRITICAL_FAILED=$((CRITICAL_FAILED + 1))
        else
            echo -e "[${YELLOW}WARN${NC}]"
        fi
        FAILED=$((FAILED + 1))
    fi
}

while IFS= read -r line; do
    # Enter the OPERATIONAL_CHECKS section
    if [[ "$line" == "### [OPERATIONAL_CHECKS]" ]]; then
        in_section=1
        continue
    fi

    # Exit at next top-level section separator
    if [[ $in_section -eq 1 && "$line" == "---" ]]; then
        # Flush last check and mark done so we don't double-flush
        run_check "$current_name" "$current_desc" "$current_run" "$current_severity"
        current_name=""
        break
    fi

    [[ $in_section -eq 0 ]] && continue

    # Section label comments (# ── Secrets ──)
    if [[ "$line" =~ ^#\ ──\ (.+)\ ── ]]; then
        # Flush any pending check before new section
        run_check "$current_name" "$current_desc" "$current_run" "$current_severity"
        current_name=""
        current_desc=""
        current_run=""
        current_severity="warning"

        echo -e "\n  ${BOLD}${BASH_REMATCH[1]}${NC}"
        continue
    fi

    # Skip pure comment lines
    [[ "$line" =~ ^#\  ]] && continue
    [[ -z "$line" ]] && continue

    # Check name line: ### some-check-name
    if [[ "$line" =~ ^###\ ([a-zA-Z][a-zA-Z0-9_-]+)$ ]]; then
        # Flush previous check
        run_check "$current_name" "$current_desc" "$current_run" "$current_severity"
        current_name="${BASH_REMATCH[1]}"
        current_desc=""
        current_run=""
        current_severity="warning"
        continue
    fi

    # Description line
    if [[ "$line" =~ ^-\ description:\ (.+)$ ]]; then
        current_desc="${BASH_REMATCH[1]}"
        continue
    fi

    # Run line
    if [[ "$line" =~ ^-\ run:\ (.+)$ ]]; then
        current_run="${BASH_REMATCH[1]}"
        continue
    fi

    # Severity line
    if [[ "$line" =~ ^-\ severity:\ (.+)$ ]]; then
        current_severity="${BASH_REMATCH[1]}"
        continue
    fi

done < "$TRUSTFILE"

# Flush final check if file ends without ---
run_check "$current_name" "$current_desc" "$current_run" "$current_severity"

echo ""
echo -e "${CYAN}────────────────────────────────────────────────────────────────${NC}"
echo -e "  Passed: ${GREEN}${PASSED}${NC}  |  Failed: ${RED}${FAILED}${NC}  |  Critical: ${RED}${CRITICAL_FAILED}${NC}"
echo -e "${CYAN}────────────────────────────────────────────────────────────────${NC}"

if [[ $CRITICAL_FAILED -gt 0 ]]; then
    echo -e "\n${RED}BLOCKED: ${CRITICAL_FAILED} critical check(s) failed.${NC}"
    echo -e "${RED}Build/deploy is not permitted until all critical checks pass.${NC}"
    exit 1
fi

if [[ $FAILED -gt 0 ]]; then
    echo -e "\n${YELLOW}WARNING: ${FAILED} non-critical check(s) need attention before release.${NC}"
    exit 0
fi

echo -e "\n${GREEN}All checks passed.${NC}"
exit 0

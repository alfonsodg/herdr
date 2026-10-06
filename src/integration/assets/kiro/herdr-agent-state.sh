#!/bin/sh
# installed by herdr
# managed by herdr; reinstalling or updating the integration overwrites this file.
# add custom hooks beside this file instead of editing it.
# HERDR_INTEGRATION_ID=kiro
# HERDR_INTEGRATION_VERSION=1

set -eu

action="${1:-}"
[ "$action" = "session" ] || exit 0

[ "${HERDR_ENV:-}" = "1" ] || exit 0
[ -n "${HERDR_PANE_ID:-}" ] || exit 0
[ -n "${HERDR_SOCKET_PATH:-}" ] || exit 0

if [ -n "${HERDR_BIN_PATH:-}" ]; then
    [ -x "$HERDR_BIN_PATH" ] || exit 0
else
    command -v herdr >/dev/null 2>&1 || exit 0
fi
command -v python3 >/dev/null 2>&1 || exit 0

# SessionStart stdout is injected into Kiro context. Keep every path silent.
python3 -c '
import json
import os
import subprocess
import sys
import time

try:
    content = sys.stdin.read().strip()
    if not content:
        raise ValueError("empty input")
    payload = json.loads(content)
    event_name = payload.get("hook_event_name") or payload.get("trigger")
    if event_name and event_name not in ("SessionStart", "agentSpawn", "AgentSpawn"):
        raise ValueError("event mismatch")

    session_id = payload.get("session_id")
    if not isinstance(session_id, str) or not session_id or len(session_id) > 256:
        raise ValueError("invalid session_id")
    if any(ord(c) < 32 or ord(c) == 127 for c in session_id):
        raise ValueError("control char in session_id")

    command = os.environ.get("HERDR_BIN_PATH") or "herdr"
    args = [
        command, "pane", "report-agent-session", os.environ["HERDR_PANE_ID"],
        "--source", "herdr:kiro", "--agent", "kiro-cli",
        "--agent-session-id", session_id, "--seq", str(time.time_ns()),
    ]

    subprocess.run(
        args,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=1,
        check=False,
    )
except Exception:
    pass
' 2>/dev/null || true

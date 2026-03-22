#!/usr/bin/env python3
"""
minimal agent-side example for publishing cron telemetry into UMBRA.

usage:
1. set UMBRA_UAP, UMBRA_TOKEN and AGENT_ID
2. adapt build_snapshot() to your scheduler
3. run the script whenever schedules change
"""

from __future__ import annotations

import json
import os
from datetime import datetime, timedelta, timezone
from urllib import request


def build_snapshot() -> dict:
    now = datetime.now(timezone.utc)
    next_run = now + timedelta(hours=12)

    return {
        "agentName": os.getenv("AGENT_NAME", "forge"),
        "jobs": [
            {
                "id": "daily-build",
                "job": "daily build",
                "timing": "09:00",
                "recurrence": "weekdays",
                "timezone": "Europe/Berlin",
                "enabled": True,
                "lastRun": now.isoformat(),
                "nextRun": next_run.isoformat(),
                "lastStatus": "ok",
                "notes": "ships internal build + digest",
                "source": "python scheduler",
                "command": "python build.py",
            }
        ],
    }


def post_snapshot(payload: dict) -> None:
    base_url = os.environ["UMBRA_UAP"].rstrip("/")
    token = os.environ["UMBRA_TOKEN"]
    agent_id = os.environ.get("AGENT_ID", "forge")
    url = f"{base_url}/api/agents/{agent_id}/cron-jobs"

    body = json.dumps(payload).encode("utf-8")
    req = request.Request(
        url,
        data=body,
        headers={
            "Content-Type": "application/json",
            "X-Agent-Token": token,
        },
        method="POST",
    )

    with request.urlopen(req, timeout=5) as response:
        print(response.read().decode("utf-8"))


if __name__ == "__main__":
    post_snapshot(build_snapshot())

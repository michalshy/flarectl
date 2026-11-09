#!/usr/bin/env python3
"""Fill a local Flare server with test runs through the HTTP API.

Dev tool only.

    python3 dev/seed.py
    python3 dev/seed.py --runs 500 --parallel 16
    FLARE_URL=http://localhost:8080 FLARE_ADMIN_TOKEN=... python3 dev/seed.py

Re-running is safe: existing jobs and the dev worker are reused, new runs are added.

What it produces - dev data for working on things without having to run anything
"""

import argparse
import json
import os
import random
import sys
import time
import urllib.error
import urllib.request
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed

# Fixed id so re-runs reuse the same worker instead of registering a new one each time.
WORKER_ID = "0f5e2c1a-6d1b-4e8a-9c3f-7a2b4d6e8f10"

JOBS = [
    {
        "key": "nightly-build",
        "name": "Nightly build",
        "steps": ["fetch", "build", "cook", "upload"],
        "weight": 6,
        "outcomes": {"succeeded": 80, "failed": 12, "aborted": 5, "crash": 3},
        "triggers": {"schedule": 8, "ui": 1, "external": 1},
    },
    {
        "key": "flaky",
        "name": "Flaky integration tests",
        "steps": ["setup", "run tests", "teardown"],
        "weight": 5,
        "outcomes": {"succeeded": 50, "failed": 50},
        "triggers": {"external": 1},
    },
    {
        "key": "panics",
        "name": "Crashes inside a step",
        "steps": ["prepare", "process", "report"],
        "weight": 2,
        "outcomes": {"crash": 85, "succeeded": 15},
        "triggers": {"external": 1},
    },
    {
        "key": "backup",
        "name": "Database backup (no steps)",
        "steps": [],
        "weight": 4,
        "outcomes": {"succeeded": 90, "failed": 10},
        "triggers": {"external": 1},
    },
    {
        "key": "asset-pipeline",
        "name": "Asset pipeline (many steps)",
        "steps": [f"process batch {i:02}" for i in range(1, 26)],
        "weight": 1,
        "outcomes": {"succeeded": 70, "failed": 20, "aborted": 10},
        "triggers": {"ui": 1, "external": 1},
    },
    {
        "key": "windows-package",
        "name": "Package Windows build",
        "command": "build.bat package",
        "labels": {"os": "windows"},
        "steps": ["compile", "sign", "zip"],
        "weight": 3,
        "outcomes": {"succeeded": 85, "failed": 10, "aborted": 5},
        "triggers": {"schedule": 3, "ui": 1},
    },
    {
        "key": "long-names",
        "name": "A job with a deliberately very long name to check how tables, headers "
        "and breadcrumbs truncate text that does not fit",
        "steps": [
            "a step whose name is long enough to wrap onto a second line in a narrow column",
            "zażółć gęślą jaźń ✓ ünïcødé",
            "x",
        ],
        "weight": 1,
        "outcomes": {"succeeded": 60, "failed": 40},
        "triggers": {"external": 1},
    },
    {
        "key": "never-run",
        "name": "Job without runs",
        "steps": [],
        "weight": 0,
        "outcomes": {"succeeded": 1},
        "triggers": {"external": 1},
    },
]

FINAL = {
    "succeeded": ("succeeded", [0, None]),
    "failed": ("failed", [1, 2, 255, None]),
    "aborted": ("aborted", [130, 143, None]),
    "crash": ("failed", [101]),
}


class ApiError(Exception):
    def __init__(self, method, path, status, body):
        super().__init__(f"{method} {path} -> HTTP {status}: {body}")
        self.status = status


class Api:
    def __init__(self, url):
        self.url = url.rstrip("/")

    def call(self, method, path, token, body=None):
        data = json.dumps(body).encode() if body is not None else None
        headers = {"Authorization": f"Bearer {token}"}
        if data is not None:
            headers["Content-Type"] = "application/json"
        req = urllib.request.Request(
            f"{self.url}/v1{path}", data=data, method=method, headers=headers
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                raw = resp.read()
        except urllib.error.HTTPError as e:
            raise ApiError(method, path, e.code, e.read().decode(errors="replace")) from None
        return json.loads(raw) if raw else None


def pick(rng, weights):
    keys = list(weights)
    return rng.choices(keys, weights=[weights[k] for k in keys])[0]


def ensure_worker(api, admin):
    api.call("PUT", f"/workers/{WORKER_ID}", admin, {"name": "dev-worker-1", "version": "dev"})


def ensure_job(api, admin, spec):
    body = {
        "key": spec["key"],
        "name": spec["name"],
        "command": spec.get("command"),
        "required_labels": spec.get("labels", {}),
    }
    try:
        created = api.call("POST", "/jobs", admin, body)
        return created["id"], created["token"]
    except ApiError as e:
        if e.status != 409:
            raise
    # Key already exists from an earlier run of this script: reuse the job.
    job = next(j for j in api.call("GET", "/jobs", admin) if j["key"] == spec["key"])
    token = api.call("GET", f"/jobs/{job['id']}/token", admin)["token"]
    return job["id"], token


def start_run(api, token, trigger):
    run_id = str(uuid.uuid4())
    worker_id = WORKER_ID if trigger in ("schedule", "ui") else None
    api.call("PUT", f"/runs/{run_id}", token, {"trigger": trigger, "worker_id": worker_id})
    return run_id


def start_step(api, token, run_id, name, position):
    step_id = str(uuid.uuid4())
    api.call(
        "PUT", f"/runs/{run_id}/steps/{step_id}", token, {"name": name, "position": position}
    )
    return step_id


def play_run(api, job, rng, max_step_delay):
    """One complete run. Returns the outcome name."""
    token = job["token"]
    run_id = start_run(api, token, pick(rng, job["triggers"]))
    outcome = pick(rng, job["outcomes"])
    steps = job["steps"]

    # Index of the step where things go wrong; len(steps) means every step succeeds.
    last = len(steps) if outcome == "succeeded" or not steps else rng.randrange(len(steps))

    for pos, name in enumerate(steps[: last + 1]):
        step_id = start_step(api, token, run_id, name, pos)
        time.sleep(rng.uniform(0, max_step_delay))
        api.call("POST", f"/runs/{run_id}/heartbeat", token)
        if pos < last:
            status = "succeeded"
        elif outcome == "failed":
            status = "failed"
        else:
            # aborted / crash: leave the step running; finishing the run fails it.
            continue
        api.call("POST", f"/runs/{run_id}/steps/{step_id}/finish", token, {"status": status})

    final_status, codes = FINAL[outcome]
    api.call(
        "POST",
        f"/runs/{run_id}/finish",
        token,
        {"final_status": final_status, "exit_code": rng.choice(codes)},
    )
    return outcome


def start_abandoned(api, job, rng):
    """A run that stops reporting mid-step, for the sweeper to fail."""
    token = job["token"]
    run_id = start_run(api, token, pick(rng, job["triggers"]))
    if job["steps"]:
        start_step(api, token, run_id, job["steps"][0], 0)
    return job, run_id


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--url", default=os.environ.get("FLARE_URL", "http://localhost:8080"))
    p.add_argument(
        "--admin-token",
        default=os.environ.get("FLARE_ADMIN_TOKEN", "flare-dev-admin-token"),
        help="defaults to the token in docker-compose.yml",
    )
    p.add_argument("--runs", type=int, default=300, help="complete runs to create (default 300)")
    p.add_argument("--parallel", type=int, default=8, help="runs in flight at once (default 8)")
    p.add_argument("--max-step-delay", type=float, default=0.5, help="seconds, per step (default 0.5)")
    p.add_argument("--timeouts", type=int, default=6, help="runs left for the sweeper (default 6)")
    p.add_argument(
        "--wait",
        type=float,
        default=45,
        help="seconds to wait for the sweeper; server timeout + one sweep interval (default 45)",
    )
    p.add_argument("--seed", type=int, default=None, help="random seed for repeatable outcomes")
    args = p.parse_args()

    api = Api(args.url)
    rng = random.Random(args.seed)

    try:
        ensure_worker(api, args.admin_token)
        jobs = []
        for spec in JOBS:
            job_id, token = ensure_job(api, args.admin_token, spec)
            jobs.append({**spec, "id": job_id, "token": token})
    except (ApiError, urllib.error.URLError) as e:
        sys.exit(f"setup failed: {e}")
    print(f"{len(jobs)} jobs and worker {WORKER_ID} ready")

    runnable = [j for j in jobs if j["weight"] > 0]

    # Abandoned runs first, so the sweeper clock runs while the normal runs are created.
    abandoned = [start_abandoned(api, rng.choice(runnable), rng) for _ in range(args.timeouts)]
    started_abandoned = time.monotonic()
    print(f"{len(abandoned)} runs left without heartbeats")

    counts, errors = {}, []
    plan = rng.choices(runnable, weights=[j["weight"] for j in runnable], k=args.runs)
    with ThreadPoolExecutor(max_workers=args.parallel) as pool:
        futures = [
            pool.submit(play_run, api, job, random.Random(rng.random()), args.max_step_delay)
            for job in plan
        ]
        for i, f in enumerate(as_completed(futures), 1):
            try:
                outcome = f.result()
                counts[outcome] = counts.get(outcome, 0) + 1
            except (ApiError, OSError) as e:
                errors.append(e)
            if i % 50 == 0:
                print(f"  {i}/{args.runs} runs")
    print("runs by outcome:", ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))

    if abandoned:
        remaining = args.wait - (time.monotonic() - started_abandoned)
        if remaining > 0:
            print(f"waiting {remaining:.0f}s for the sweeper...")
            time.sleep(remaining)
        late = 0
        # Half of the timed-out runs get a finish after the fact -> late_report event.
        for job, run_id in abandoned[: (len(abandoned) + 1) // 2]:
            try:
                api.call(
                    "POST", f"/runs/{run_id}/finish", job["token"],
                    {"final_status": "succeeded", "exit_code": 0},
                )
                print(f"  {run_id} was still running: finish accepted, raise --wait")
            except ApiError as e:
                if e.status == 409:
                    late += 1
                else:
                    errors.append(e)
        print(f"{late} late reports recorded")

    if errors:
        print(f"{len(errors)} errors, first ones:", file=sys.stderr)
        for e in errors[:5]:
            print(f"  {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

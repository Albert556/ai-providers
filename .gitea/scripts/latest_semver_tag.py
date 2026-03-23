#!/usr/bin/env python3
"""Fetch git refs from an API endpoint and print the latest semver tag."""

import argparse
import functools
import json
import re
import sys
import urllib.error
import urllib.request


SEMVER = re.compile(r"^v(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?(?:\+.*)?$")


def parse_tag(tag):
    match = SEMVER.match(tag)
    if not match:
        return None
    return (
        int(match[1]),
        int(match[2]),
        int(match[3]),
        match[4],
        tag,
    )


def compare_pre(a, b):
    if a is None and b is None:
        return 0
    if a is None:
        return 1
    if b is None:
        return -1

    a_parts, b_parts = a.split("."), b.split(".")
    for a_part, b_part in zip(a_parts, b_parts):
        a_num, b_num = a_part.isdigit(), b_part.isdigit()
        if a_num and b_num:
            diff = int(a_part) - int(b_part)
            if diff != 0:
                return diff
        elif a_num != b_num:
            return -1 if a_num else 1
        else:
            if a_part < b_part:
                return -1
            if a_part > b_part:
                return 1

    return len(a_parts) - len(b_parts)


def compare_tags(a, b):
    a_core = a[:3]
    b_core = b[:3]
    if a_core < b_core:
        return -1
    if a_core > b_core:
        return 1

    pre = compare_pre(a[3], b[3])
    if pre < 0:
        return -1
    if pre > 0:
        return 1
    return 0


def build_headers(auth_scheme, token):
    headers = {"Accept": "application/json"}
    if not token:
        return headers

    if auth_scheme == "bearer":
        headers["Authorization"] = f"Bearer {token}"
    elif auth_scheme == "token":
        headers["Authorization"] = f"token {token}"
    elif auth_scheme != "none":
        raise ValueError(f"Unsupported auth scheme: {auth_scheme}")

    return headers


def fetch_refs(endpoint, headers):
    request = urllib.request.Request(endpoint, headers=headers)
    try:
        with urllib.request.urlopen(request) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code in {404, 409}:
            return []
        raise


def extract_tags(payload):
    if not isinstance(payload, list):
        raise ValueError("Expected the API to return a JSON array of refs")

    tags = []
    for item in payload:
        if not isinstance(item, dict):
            continue
        ref = item.get("ref", "")
        if not isinstance(ref, str) or not ref.startswith("refs/tags/"):
            continue
        parsed = parse_tag(ref[len("refs/tags/") :])
        if parsed is not None:
            tags.append(parsed)
    return tags


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", required=True)
    parser.add_argument(
        "--auth-scheme",
        choices=["none", "bearer", "token"],
        default="none",
    )
    parser.add_argument("--token", default="")
    args = parser.parse_args()

    headers = build_headers(args.auth_scheme, args.token)
    payload = fetch_refs(args.endpoint, headers)
    tags = extract_tags(payload)

    if not tags:
        return 0

    latest = max(tags, key=functools.cmp_to_key(compare_tags))
    print(latest[4])
    return 0


if __name__ == "__main__":
    sys.exit(main())

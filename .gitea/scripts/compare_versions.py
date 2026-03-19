#!/usr/bin/env python3
"""Compare two semver versions. Exit 0, prints 'true' if new > old."""
import re
import sys


def parse_semver(v):
    m = re.match(r'^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?(?:\+.*)?$', v)
    if not m:
        print(f'Invalid semver: {v}', file=sys.stderr)
        sys.exit(2)
    return (int(m[1]), int(m[2]), int(m[3]), m[4])


def compare_pre(a, b):
    if a is None and b is None:
        return 0
    if a is None:
        return 1
    if b is None:
        return -1
    a_parts, b_parts = a.split('.'), b.split('.')
    for ap, bp in zip(a_parts, b_parts):
        a_num, b_num = ap.isdigit(), bp.isdigit()
        if a_num and b_num:
            diff = int(ap) - int(bp)
            if diff != 0:
                return diff
        elif a_num != b_num:
            return -1 if a_num else 1
        else:
            if ap < bp:
                return -1
            if ap > bp:
                return 1
    return len(a_parts) - len(b_parts)


if len(sys.argv) != 3:
    print(f'Usage: {sys.argv[0]} <old_version> <new_version>', file=sys.stderr)
    sys.exit(2)

o = parse_semver(sys.argv[1])
n = parse_semver(sys.argv[2])

core = (n[0], n[1], n[2]) > (o[0], o[1], o[2])
if (n[0], n[1], n[2]) == (o[0], o[1], o[2]):
    result = compare_pre(n[3], o[3]) > 0
else:
    result = core
print('true' if result else 'false')

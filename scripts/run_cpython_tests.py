# target/debug/oxython scripts/run_cpython_tests.py --cpython-dir cpython -j auto -v

import argparse
import importlib
import os
import sys
from pathlib import Path


def cpu_count_fallback():
    try:
        import os as _os

        return max(1, _os.cpu_count() or 1)
    except Exception:
        return 1


def read_list(path):
    out = []
    if not path:
        return out
    p = Path(path)
    if not p.exists():
        return out
    for line in p.read_text().splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        # accept names like "test_ssl.py" or "test_ssl"
        if s.endswith(".py"):
            s = s[:-3]
        out.append(s)
    return out


def main():
    ap = argparse.ArgumentParser(
        description="Run CPython Lib/test under a non-CPython interpreter"
    )
    ap.add_argument(
        "--cpython-dir", default="cpython", help="Path to CPython checkout root"
    )
    ap.add_argument("-j", "--jobs", default="1", help="Worker count: N or 'auto'")
    ap.add_argument("-v", "--verbose", default=True, action="store_true")
    ap.add_argument("-k", "--match", default=None, help="Run tests matching pattern")
    ap.add_argument(
        "--skip-file", default="tests/skips.txt", help="List of test_*.py to exclude"
    )
    ap.add_argument(
        "--extra-arg",
        action="append",
        default=[],
        help="Extra args passed through to regrtest, repeatable",
    )
    ap.add_argument(
        "tests", nargs="*", help="Optional explicit tests like test_json test_os"
    )
    args = ap.parse_args()

    cpython_dir = Path(args.cpython_dir).resolve()
    lib_dir = cpython_dir / "Lib"

    if not lib_dir.exists():
        sys.exit(f"Missing {lib_dir}. Point --cpython-dir at the CPython root.")

    # Put CPython stdlib on sys.path so 'test' and 'test.support' import
    sys.path.insert(0, str(lib_dir))

    # Keep environment tidy for tests
    os.environ.setdefault("PYTHONWARNINGS", "default")
    os.environ.setdefault("PYTHONNOUSERSITE", "1")

    # Build regrtest arg list
    regrtest_args = []

    # Parallelism
    if args.jobs == "auto":
        regrtest_args += ["-j", str(cpu_count_fallback())]
    else:
        regrtest_args += ["-j", str(int(args.jobs))]

    if args.verbose:
        regrtest_args.append("-v")

    # Exclusions
    for name in read_list(args.skip_file):
        regrtest_args += ["-x", name]

    # Pattern focus
    if args.match:
        regrtest_args += ["-k", args.match]

    # Any extra passthrough flags
    regrtest_args += args.extra_arg

    # If user passed explicit tests, append them last
    regrtest_args += args.tests

    # Import and run
    try:
        regrtest = importlib.import_module("test.regrtest")
    except Exception as e:
        sys.exit(f"Could not import test.regrtest from {lib_dir}: {e}")

    # Hand off to CPython's test runner
    # This exits the process with an appropriate status code
    regrtest.main(regrtest_args)


if __name__ == "__main__":
    main()

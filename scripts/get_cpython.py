# Clone or update a shallow partial clone of CPython at a given tag.
# python scripts/get_cpython.py v3.14.0            # get recent
# python scripts/get_cpython.py v3.12.6 --force    # for downgrades

import argparse
import os
import re
import shlex
import subprocess
import sys

DEF_URL = "https://github.com/python/cpython.git"
DEF_DIR = "cpython"
TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")


def run(cmd, cwd=None, check=True, capture=False):
    if isinstance(cmd, str):
        cmd = shlex.split(cmd)
    kw = {"cwd": cwd, "check": check}
    if capture:
        kw.update({"stdout": subprocess.PIPE, "stderr": subprocess.PIPE, "text": True})
    return subprocess.run(cmd, **kw)


def parse_tag(tag):
    m = TAG_RE.match(tag)
    if not m:
        sys.exit(f"Invalid tag: {tag}. Expected like v3.12.6")
    return tuple(map(int, m.groups()))


def current_tag(repo_dir):
    r = run(
        ["git", "tag", "--points-at", "HEAD"], cwd=repo_dir, capture=True, check=False
    )
    if r.returncode == 0:
        for t in [t.strip() for t in r.stdout.splitlines() if t.strip()]:
            if TAG_RE.match(t):
                return t
    r = run(
        ["git", "describe", "--tags", "--exact-match"],
        cwd=repo_dir,
        capture=True,
        check=False,
    )
    if r.returncode == 0 and TAG_RE.match(r.stdout.strip()):
        return r.stdout.strip()
    return None


def ensure_sparse(repo_dir):
    run(["git", "sparse-checkout", "init", "--cone"], cwd=repo_dir, check=True)
    run(
        ["git", "sparse-checkout", "set", "Lib", "Lib/test", "Tools"],
        cwd=repo_dir,
        check=True,
    )


def checkout_tag(repo_dir, tag):
    run(
        ["git", "fetch", "--depth", "1", "origin", "tag", tag], cwd=repo_dir, check=True
    )
    run(["git", "checkout", "-q", "FETCH_HEAD"], cwd=repo_dir, check=True)
    ensure_sparse(repo_dir)


def clone_or_update(tag, repo_url=DEF_URL, repo_dir=DEF_DIR, allow_downgrade=False):
    want_ver = parse_tag(tag)

    if not os.path.isdir(repo_dir):
        print(f"Cloning {tag} into {repo_dir}...")
        run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "--filter=blob:none",
                "--branch",
                tag,
                repo_url,
                repo_dir,
            ]
        )
        ensure_sparse(repo_dir)
        print("Done.")
        return

    print(f"Found existing repo at {repo_dir}")
    run(["git", "remote", "set-url", "origin", repo_url], cwd=repo_dir, check=True)

    have_tag = current_tag(repo_dir)
    if have_tag:
        have_ver = parse_tag(have_tag)
        if have_ver == want_ver:
            print(f"Already at {have_tag}. Ensuring sparse checkout and exiting.")
            ensure_sparse(repo_dir)
            return
        if have_ver > want_ver and not allow_downgrade:
            print(
                f"Current {have_tag} is newer than requested {tag}. Use --force to downgrade."
            )
            return

    print(f"Switching to {tag}...")
    checkout_tag(repo_dir, tag)
    print(f"Checked out {tag}.")


def main():
    p = argparse.ArgumentParser(
        description="Shallow partial clone of CPython at a tag, with upgrade and optional downgrade."
    )
    p.add_argument("tag", help="Tag like v3.12.6")
    p.add_argument("--url", default=DEF_URL, help=f"Git URL. Default {DEF_URL}")
    p.add_argument(
        "--dir", default=DEF_DIR, help=f"Target directory. Default {DEF_DIR}"
    )
    p.add_argument(
        "--force", action="store_true", help="Permit switching to an older tag"
    )
    args = p.parse_args()

    try:
        clone_or_update(args.tag, args.url, args.dir, allow_downgrade=args.force)
    except subprocess.CalledProcessError as e:
        msg = e.stderr if getattr(e, "stderr", None) else str(e)
        sys.exit(f"git failed: {msg}")


if __name__ == "__main__":
    main()

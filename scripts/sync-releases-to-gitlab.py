#!/usr/bin/env python3
"""Sync all GitHub Releases (with assets) to GitLab Releases."""

import json
import os
import sys
import time
import urllib.error
import urllib.request
import uuid

GL_TOKEN = os.environ.get("GL_TOKEN")
if not GL_TOKEN:
    print("FATAL: GL_TOKEN environment variable not set", file=sys.stderr)
    sys.exit(1)

GL_PROJECT = "dexoron-studio%2Fdcr"
GL_PROJECT_RAW = "dexoron-studio/dcr"
GL_API = "https://gitlab.com/api/v4"
GH_REPO = "dexoron/dcr"


def gl_api(method, path, data=None, filepath=None, retries=5):
    url = f"{GL_API}/{path}"
    headers = {"PRIVATE-TOKEN": GL_TOKEN}

    if data is not None and filepath is None:
        headers["Content-Type"] = "application/json"
        body = json.dumps(data).encode()
    elif filepath:
        boundary = uuid.uuid4().hex
        with open(filepath, "rb") as f:
            fdata = f.read()
        fname = os.path.basename(filepath)
        parts = []
        parts.append(f"--{boundary}".encode())
        parts.append(f'Content-Disposition: form-data; name="file"; filename="{fname}"'.encode())
        parts.append(b"Content-Type: application/octet-stream")
        parts.append(b"")
        parts.append(fdata)
        parts.append(f"--{boundary}--".encode())
        parts.append(b"")
        body = b"\r\n".join(parts)
        headers["Content-Type"] = f"multipart/form-data; boundary={boundary}"
    else:
        body = None

    for attempt in range(retries):
        try:
            req = urllib.request.Request(url, data=body, headers=headers, method=method)
            with urllib.request.urlopen(req) as resp:
                text = resp.read().decode()
                return json.loads(text) if text else None
        except urllib.error.HTTPError as e:
            body_text = e.read().decode() if e.fp else ""
            if e.code in (409, 429) and attempt < retries - 1:
                time.sleep(3)
                continue
            if attempt < retries - 1:
                time.sleep(1)
                continue
            print(f"  HTTP {e.code}: {body_text[:300]}", file=sys.stderr)
            return None
        except Exception as e:
            if attempt < retries - 1:
                time.sleep(1)
                continue
            print(f"  Error: {e}", file=sys.stderr)
            return None
    return None


def download_file(url, dest):
    with urllib.request.urlopen(url) as resp:
        with open(dest, "wb") as f:
            f.write(resp.read())


# --- MAIN ---
print("Fetching GitHub releases...")
with urllib.request.urlopen(f"https://api.github.com/repos/{GH_REPO}/releases?per_page=50") as r:
    gh_releases = json.loads(r.read().decode())

print(f"Found {len(gh_releases)} releases on GitHub")

# Reverse to process oldest first (GitHub returns newest first)
gh_releases.reverse()

print("Fetching existing GitLab releases and their assets...")
existing = gl_api("GET", f"projects/{GL_PROJECT}/releases?per_page=100") or []
existing = existing if isinstance(existing, list) else []

# Build map: tag -> set of existing asset names
gl_assets = {}
for rel in existing:
    tag = rel["tag_name"]
    links = rel.get("assets", {}).get("links", []) or rel.get("assets", {}).get("sources", []) or []
    gl_assets[tag] = {l.get("name", "") for l in links if isinstance(l, dict)}
    # Also get the release description to check if it needs updating
    gl_assets[tag + "__desc"] = rel.get("description", "")

print(f"Existing GitLab releases: {len(existing)}")

download_dir = "/tmp/gh-assets"
os.makedirs(download_dir, exist_ok=True)

# Reverse: process older first so newer releases with many assets get their description too
# Actually, process in original order
total = len(gh_releases)
for i, rel in enumerate(gh_releases):
    tag = rel["tag_name"]
    body = rel.get("body", "") or ""
    assets = rel.get("assets", [])
    if isinstance(assets, dict):
        assets = assets.get("nodes", []) or []

    print(f"\n{'='*60}")
    print(f"[{i+1}/{total}] {tag} ({len(assets)} assets)")

    # Create or update release
    existing_tag = tag in gl_assets
    if not existing_tag:
        print(f"  Creating release...")
        result = gl_api("POST", f"projects/{GL_PROJECT}/releases", data={
            "tag_name": tag,
            "name": rel.get("name", tag),
            "description": body,
            "ref": "master",
        })
        if result is None:
            print(f"  FAILED — skip")
            continue
        print(f"  OK")
    else:
        # Update description if needed
        existing_desc = gl_assets.get(tag + "__desc", "")
        if body and body != existing_desc:
            print(f"  Updating description...")
            gl_api("PUT", f"projects/{GL_PROJECT}/releases/{urllib.request.quote(tag, safe='')}",
                   data={"description": body})

    existing_links = gl_assets.get(tag, set())

    for j, asset in enumerate(assets):
        aname = asset.get("name", "")
        aurl = asset.get("browser_download_url", "")
        if not aurl:
            continue

        if aname in existing_links:
            print(f"  [{j+1}/{len(assets)}] {aname} — already exists")
            continue

        fpath = os.path.join(download_dir, aname)
        print(f"  [{j+1}/{len(assets)}] {aname}...", end=" ", flush=True)

        # Download from GitHub
        try:
            download_file(aurl, fpath)
        except Exception as e:
            print(f"download failed: {e}")
            continue

        # Upload to GitLab
        up = gl_api("POST", f"projects/{GL_PROJECT}/uploads", filepath=fpath)
        if not up or "url" not in up:
            size_mb = os.path.getsize(fpath) / (1024 * 1024)
            print(f"upload failed ({size_mb:.1f} MB)")
            os.remove(fpath)
            continue

        file_url = up["url"]
        full_url = f"https://gitlab.com/{GL_PROJECT_RAW}{file_url}"

        # Link asset to release
        encoded_tag = urllib.request.quote(tag, safe='')
        link = gl_api("POST",
            f"projects/{GL_PROJECT}/releases/{encoded_tag}/assets/links",
            data={"name": aname, "url": full_url},
            retries=5)
        if link:
            print(f"linked")
            existing_links.add(aname)
        else:
            print(f"link failed")

        os.remove(fpath)

    time.sleep(0.3)

print(f"\n{'='*60}")
print("Done!")
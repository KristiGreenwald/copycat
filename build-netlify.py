#!/usr/bin/env python3
"""
Build the Netlify deploy bundle from docs/.

Why this exists: assets are served with `immutable` caching, which tells the
browser never to revalidate. That is only safe when the filename changes
whenever the bytes change — otherwise an updated image is invisible to anyone
who already visited (which is exactly what happened with hero-window.webp).

So this fingerprints every /images asset with a short content hash and rewrites
the references. Source filenames in docs/ stay clean; only the bundle is hashed.

    python3 build-netlify.py [outdir]        # default: ../netlify-bundle
"""
import hashlib
import os
import re
import shutil
import sys

SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "docs")
OUT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else
                      os.path.join(os.path.dirname(SRC), "..", "netlify-bundle"))

# page in docs/ -> page in the bundle. light.html is the real homepage.
PAGES = {"light.html": "index.html", "thanks.html": "thanks.html"}

# Served from conventional paths, so they must NOT be fingerprinted.
UNHASHED = {"favicon.png", "apple-touch-icon.png", "robots.txt"}

REF = re.compile(r'(?:src|href|srcset|poster)="([^"]+)"')

NETLIFY_TOML = """\
# CopyCat — static marketing site. No build step on Netlify: this folder is
# already built by build-netlify.py, so publish it as-is.
[build]
  publish = "."

[[headers]]
  for = "/*"
  [headers.values]
    X-Content-Type-Options = "nosniff"
    X-Frame-Options = "DENY"
    Referrer-Policy = "strict-origin-when-cross-origin"
    Permissions-Policy = "camera=(), microphone=(), geolocation=(), interest-cohort=()"

# Safe to freeze: every file under /images carries a content hash in its name,
# so changed bytes always mean a changed URL.
[[headers]]
  for = "/images/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

# Unhashed: must revalidate, or updates never reach returning visitors.
[[headers]]
  for = "/*.html"
  [headers.values]
    Cache-Control = "public, max-age=0, must-revalidate"

[[headers]]
  for = "/favicon.png"
  [headers.values]
    Cache-Control = "public, max-age=3600, must-revalidate"

[[headers]]
  for = "/apple-touch-icon.png"
  [headers.values]
    Cache-Control = "public, max-age=3600, must-revalidate"

[[headers]]
  for = "/thanks"
  [headers.values]
    X-Robots-Tag = "noindex"
"""


def main():
    shutil.rmtree(OUT, ignore_errors=True)
    os.makedirs(os.path.join(OUT, "images"), exist_ok=True)

    html = {dst: open(os.path.join(SRC, s), encoding="utf-8").read()
            for s, dst in PAGES.items()}

    refs = set()
    for body in html.values():
        for r in REF.findall(body):
            if r.startswith(("http", "#", "mailto:", "data:")) or "${" in r:
                continue
            refs.add(r.split("?")[0])

    mapping, shipped, missing = {}, 0, []
    for ref in sorted(refs):
        path = os.path.join(SRC, ref)
        if not os.path.isfile(path):
            missing.append(ref)
            continue
        name = os.path.basename(ref)
        if name in UNHASHED or not ref.startswith("images/"):
            dst_rel = ref
        else:
            digest = hashlib.sha256(open(path, "rb").read()).hexdigest()[:10]
            stem, ext = os.path.splitext(name)
            dst_rel = f"images/{stem}.{digest}{ext}"
            mapping[ref] = dst_rel
        dst = os.path.join(OUT, dst_rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copy2(path, dst)
        shipped += 1

    # longest-first so images/logo.png is never rewritten by a prefix match
    for src_rel in sorted(mapping, key=len, reverse=True):
        for page in html:
            html[page] = html[page].replace(src_rel, mapping[src_rel])

    for page, body in html.items():
        open(os.path.join(OUT, page), "w", encoding="utf-8").write(body)

    open(os.path.join(OUT, "netlify.toml"), "w").write(NETLIFY_TOML)
    open(os.path.join(OUT, "robots.txt"), "w").write(
        "User-agent: *\nAllow: /\nDisallow: /thanks\n")

    print(f"bundle:      {OUT}")
    print(f"pages:       {', '.join(sorted(html))}")
    print(f"assets:      {shipped} ({len(mapping)} fingerprinted)")
    print(f"unreferenced/missing: {missing or 'none'}")
    for k, v in sorted(mapping.items()):
        print(f"  {k:34} -> {os.path.basename(v)}")


if __name__ == "__main__":
    main()

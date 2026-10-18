#!/usr/bin/env python3
"""Copy the library databases into test-fixtures/ for the differential tests,
which never open data/ itself."""
import pathlib
import shutil
import sqlite3
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / 'test-fixtures'
if OUT.exists() and any(OUT.iterdir()):
    raise SystemExit('test-fixtures already exists; goldens require these exact frozen files. Move it aside explicitly before regenerating and re-record all goldens.')
OUT.mkdir(parents=True, exist_ok=True)
for source, name in [('data/mymind.db', 'mymind.db'), ('data/hosted/test.db', 'hosted-test.db')]:
    with sqlite3.connect(f'file:{ROOT.parent / source}?mode=ro', uri=True) as src, sqlite3.connect(OUT / name) as dst:
        src.backup(dst)
        dst.execute('PRAGMA journal_mode = DELETE')
    print(OUT / name)


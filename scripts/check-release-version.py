import json
import os
import re
import tomllib
from pathlib import Path

tag = os.environ.get('RELEASE_TAG') or 'v' + json.loads(Path('package.json').read_text())['version']
if not re.fullmatch(r'v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', tag):
    raise SystemExit(f'Invalid stable release tag: {tag}')
version = tag[1:]
versions = {}
for filename in ('package.json', 'src-tauri/tauri.conf.json'):
    versions[filename] = json.loads(Path(filename).read_text())['version']
npm_lock = json.loads(Path('package-lock.json').read_text())
versions['package-lock.json'] = npm_lock['version']
versions['package-lock.json root'] = npm_lock['packages']['']['version']
cargo = tomllib.loads(Path('src-tauri/Cargo.toml').read_text())['package']
versions['src-tauri/Cargo.toml'] = cargo['version']
cargo_lock = tomllib.loads(Path('src-tauri/Cargo.lock').read_text())
versions['src-tauri/Cargo.lock'] = next(
    p['version'] for p in cargo_lock['package'] if p['name'] == cargo['name']
)
for filename, actual in versions.items():
    if actual != version:
        raise SystemExit(f'{filename}: expected {version}, got {actual}')
print(f'All manifest and lockfile versions match {tag}')

"""
サードパーティライセンスを生成するスクリプト

Cargo由来のソフトウェアとadditional.jsonに記したソフトウェアを対象とする。
LICENSES.txtが生成される。

前提条件:
- cargo-aboutをインストールしておくこと
- このファイルと同じディレクトリで実行すること
"""

import json
import platform
import subprocess
from pathlib import Path

current_platform = platform.system()

TARGETS = {
	'Windows': 'x86_64-pc-windows-msvc',
	'Darwin': 'aarch64-apple-darwin',
	'Linux': 'x86_64-unknown-linux-gnu',
}

target = TARGETS.get(current_platform)
if not target:
	raise RuntimeError(f'unsupported os: {current_platform}')

output = subprocess.run(
	[
		'cargo', 'about', 'generate',
		'--manifest-path', '../Cargo.toml',
		'--config', 'about.toml',
		'--target', target,
		'--format', 'json',
	],
	capture_output=True, text=True, encoding='utf-8', check=True
).stdout

cargo_about_data = json.loads(output)
additional_data = json.loads(Path('additional.json').read_text('utf8'))

deps = additional_data.copy()

for license in cargo_about_data['licenses']:
	for crate in license['used_by']:
		deps.append({
			'name': crate['crate']['name'],
			'version': crate['crate']['version'],
			'text': license['text'],
		})

deps.sort(key=lambda d: d['name'])

result = '# Licenses\n\n'

for dep in deps:
	if dep['name'] == 'MoltenVK' and current_platform != 'Darwin':
		continue
	if dep['name'] == 'xcb' and current_platform != 'Linux':
		continue
	result += f"## {dep['name']} {dep['version']}\n\n{dep['text']}\n\n\n"

result = result.rstrip() + '\n'

Path('LICENSES.txt').write_text(result, 'utf8')

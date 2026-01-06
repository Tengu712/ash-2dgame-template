/**
 * @file build.mjs
 * @description
 *   サードパーティライセンスを生成するスクリプト。
 *   Cargo由来のソフトウェアとadditional.jsonに記したソフトウェアを対象とする。
 *   LICENSES.txtが生成される。
 *
 * @note cargo-aboutをインストールしておくこと
 * @note このファイルと同じディレクトリで実行すること
 *
 * @example
 *   node build.mjs
 */

import { execSync } from 'child_process';
import fs from 'fs'
import os from 'os'

const target = {
	'win32': 'x86_64-pc-windows-msvc',
}[os.platform()]

if (!target) {
	throw new Error(`unsupported os: ${os.platform()}`)
}

const output = execSync(
	`cargo about generate --manifest-path ../Cargo.toml --config about.toml --target ${target} --format json`,
	{ encoding: 'utf8' }
)

const cargoAboutData = JSON.parse(output);
const additionalData = JSON.parse(fs.readFileSync('additional.json', 'utf8'));

const deps = additionalData

for (const license of cargoAboutData.licenses) {
	for (const crate of license.used_by) {
		deps.push({
			name: crate.crate.name,
			version: crate.crate.version,
			text: license.text,
		})
	}
}

deps.sort((a, b) => a.name.localeCompare(b.name))

let result = '# Licenses\n\n'

for (const dep of Object.values(deps)) {
	if (dep.name === 'xorg' && os.platform() !== 'linux') {
		continue
	}
	result += `## ${dep.name} ${dep.version}\n\n${dep.text}\n\n\n`
}

result = result.trimEnd()
result += '\n'

fs.writeFileSync('LICENSES.txt', result, 'utf8')

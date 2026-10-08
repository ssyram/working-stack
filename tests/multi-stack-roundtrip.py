import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

external = Path(sys.argv[1]).resolve()
project = Path(__file__).resolve().parents[1]
out = Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=False)
sources = []
for path in sorted(external.rglob('*.md')):
    text = path.read_text()
    if re.search(r'^## 栈状态\s*$', text, re.M) and re.search(r'^## \[[A-Za-z0-9]+-\d+\]', text, re.M):
        sources.append(path)
sources.append(project / 'WS-TOOL.md')
sources.extend(sorted((project / '.working-stack').glob('*.yaml')))
results = []

def differences(a, b, path='stack'):
    if a == b: return []
    if isinstance(a, dict) and isinstance(b, dict):
        changes = []
        for key in sorted(set(a) | set(b)):
            if key in a and key in b: changes.extend(differences(a[key], b[key], path + '.' + key))
            else: changes.append(dict(path=path + '.' + key, before=a.get(key), after=b.get(key)))
        return changes
    if isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        return [change for i, (x, y) in enumerate(zip(a, b)) for change in differences(x, y, path + '[' + str(i) + ']')]
    return [dict(path=path, before=a, after=b)]

for number, source in enumerate(sources, 1):
    case = out / ('%02d-' % number + source.stem)
    case.mkdir()
    raw = source.read_bytes()
    snapshot = case / ('input' + source.suffix)
    snapshot.write_bytes(raw)
    record = dict(source=str(source), case=str(case), input_sha256=hashlib.sha256(raw).hexdigest(), outcome='not_run')
    def run(tag, *args):
        r = subprocess.run(['working-stack'] + [str(a) for a in args], cwd=case, input='', capture_output=True, text=True, timeout=30)
        (case / (tag + '.stdout')).write_text(r.stdout)
        (case / (tag + '.stderr')).write_text(r.stderr)
        if r.returncode: raise RuntimeError(tag + ': ' + r.stderr)
        return json.loads(r.stdout)
    try:
        stack = case / 'stack.yaml' if source.suffix == '.md' else snapshot
        warnings = run('import', 'import', snapshot, '-o', stack, '--json')['warnings'] if source.suffix == '.md' else []
        run('validate', stack, 'validate', '--json')
        before = json.loads(run('model-before', stack, 'export', 'json', '--json')['text'])
        run('export', stack, 'export', 'md', '--out', case / 'reprinted.md', '--json')
        reimport = case / 'reimport.yaml'
        run('reimport', 'import', case / 'reprinted.md', '-o', reimport, '--json')
        run('reimport-validate', reimport, 'validate', '--json')
        after = json.loads(run('model-after', reimport, 'export', 'json', '--json')['text'])
        changes = differences(before, after)
        (case / 'model-diff.json').write_text(json.dumps(changes, ensure_ascii=False, indent=2))
        record.update(outcome='pass' if not changes else 'model_diff', frames=len(before['frames']),
            differences=len(changes), paths=[change['path'] for change in changes], import_warnings=warnings)
    except (RuntimeError, subprocess.TimeoutExpired) as error:
        record.update(outcome='failed', error=str(error))
    record['original_file_unchanged'] = source.read_bytes() == raw
    (case / 'report.json').write_text(json.dumps(record, ensure_ascii=False, indent=2))
    results.append(record)
    print(source.name, record['outcome'], 'frames=' + str(record.get('frames', '?')), 'diff=' + str(record.get('differences', '?')))
report = dict(sources=len(results), passed=sum(r['outcome'] == 'pass' for r in results),
    failed=sum(r['outcome'] != 'pass' for r in results), results=results)
(out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps({key:value for key,value in report.items() if key != 'results'}, ensure_ascii=False))

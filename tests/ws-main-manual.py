import difflib
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

source = Path(sys.argv[1]).resolve()
out = Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=False)
raw = source.read_bytes()
(out / 'input.md').write_bytes(raw)
commands = []

def run(tag, *args):
    command = ['working-stack'] + [str(a) for a in args]
    commands.append(command)
    result = subprocess.run(command, cwd=out, input='', text=True, capture_output=True, timeout=30)
    (out / (tag + '.stdout')).write_text(result.stdout)
    (out / (tag + '.stderr')).write_text(result.stderr)
    assert result.returncode == 0, (command, result.stderr)
    return result.stdout

entry = run('ENTRY', 'doc', 'ENTRY')
manual = run('manual', 'doc', 'ai-manual')
assert 'working-stack import path/to/WS-MAIN.md -o mytask' in manual
assert 'export md --out reprinted.md' in manual
assert 'working-stack doc' in entry
run('help', '--help')
run('docs', 'doc', '--list')
stack = out / 'stack.yaml'
imported = json.loads(run('import', 'import', out / 'input.md', '-o', stack, '--json'))
run('validate', stack, 'validate')
run('export-md', stack, 'export', 'md', '--out', out / 'reprinted.md')
before = json.loads(json.loads(run('model-before', stack, 'export', 'json', '--json'))['text'])
reimport = out / 'reimport.yaml'
run('reimport', 'import', out / 'reprinted.md', '-o', reimport, '--json')
run('reimport-validate', reimport, 'validate')
after = json.loads(json.loads(run('model-after', reimport, 'export', 'json', '--json'))['text'])
output = (out / 'reprinted.md').read_bytes()
old_lines = raw.decode().splitlines(keepends=True)
new_lines = output.decode().splitlines(keepends=True)
diff = ''.join(difflib.unified_diff(old_lines, new_lines, fromfile='original/WS-MAIN.md', tofile='tool/reprinted.md'))
(out / 'comparison.diff').write_text(diff)
model_diff = []

def compare(a, b, path='stack'):
    if a == b: return
    if isinstance(a, dict) and isinstance(b, dict):
        for key in sorted(set(a) | set(b)):
            if key in a and key in b: compare(a[key], b[key], path + '.' + key)
            else: model_diff.append(dict(path=path + '.' + key, before=a.get(key), after=b.get(key)))
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)): compare(x, y, path + '[' + str(i) + ']')
    else:
        model_diff.append(dict(path=path, before=a, after=b))

compare(before, after)
(out / 'model-diff.json').write_text(json.dumps(model_diff, ensure_ascii=False, indent=2))
remarks_equal = all(after['frames'].get(k, {}).get('content', {}).get('remarks') == f['content']['remarks'] for k, f in before['frames'].items())
notes_equal = before['notes'] == after['notes']
checks = {'P1 底帧持有完整目标': '1', '四红内容债逐条过': '16',
          '| V_INTAKE instructions+门 |': '19', '[resumed · parallel]': '29'}
retained = {}
for fragment, owner in checks.items():
    if fragment in raw.decode():
        remarks = '\n'.join(before['frames'][owner]['content']['remarks'].values())
        retained[fragment] = fragment in remarks and fragment in output.decode()
report = {
    'binary_on_PATH': shutil.which('working-stack'),
    'input_sha256': hashlib.sha256(raw).hexdigest(),
    'input_lines': len(old_lines), 'output_lines': len(new_lines),
    'frame_count': len(before['frames']),
    'original_file_unchanged': source.read_bytes() == raw,
    'byte_identical': raw == output,
    'all_frame_remarks_equal_after_reimport': remarks_equal,
    'all_notes_equal_after_reimport': notes_equal,
    'previous_missing_content_retained_in_owner': retained,
    'full_model_differences_after_reimport': len(model_diff),
    'model_difference_paths': [d['path'] for d in model_diff],
    'import_warnings': imported['warnings'],
    'commands': commands,
}
(out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps({k: v for k, v in report.items() if k not in ['commands', 'import_warnings', 'model_difference_paths']}, ensure_ascii=False, indent=2))
print('comparison:', out / 'comparison.diff')
print('model differences:', out / 'model-diff.json')
assert remarks_equal and notes_equal and all(retained.values())
assert not model_diff, '完整模型往返差异见 model-diff.json'

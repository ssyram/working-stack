import hashlib
import json
from pathlib import Path
import subprocess
import sys

binary = str(Path(sys.argv[1]).resolve())
source = Path(sys.argv[2])
out = Path(sys.argv[3]).resolve()
out.mkdir(parents=True, exist_ok=False)
raw = source.read_bytes()
(out / 'input.md').write_bytes(raw)
stack = str(out / 'stack.yaml')

def request(op, args, tag, target=None):
    payload = dict(stack=stack if target is None else target, op=op, args=args, reason='验证 parse → push → re-print')
    r = subprocess.run([binary], input=json.dumps(payload, ensure_ascii=False, indent=2), text=True, capture_output=True, timeout=30)
    (out / (tag + '.response.txt')).write_text(r.stdout + r.stderr)
    assert r.returncode == 0, r.stderr
    return json.loads(r.stdout)

imported = request('import', {'md': str(out / 'input.md'), 'out': stack}, 'import')
before = json.loads(request('export', {'format': 'json'}, 'before-model')['text'])
request('export', {'format': 'md', 'out': str(out / 'before-push.md')}, 'before-md')
active = [int(k) for k, f in before['frames'].items() if f['status']['kind'] == 'active' and not any(
    x['parent'] == int(k) and x['status']['kind'] in ['active', 'paused'] for x in before['frames'].values())]
assert len(active) == 1, active
parent = active[0]
request('push', {'parent': str(parent), 'title': '往返测试帧', 'goal': '只验证输入输出，不改原栈'}, 'push')
after = json.loads(request('export', {'format': 'json'}, 'after-model')['text'])
request('validate', {}, 'validate')
request('export', {'format': 'md', 'out': str(out / 'after-push.md')}, 'after-md')
old_unchanged = all(after['frames'][k] == f for k, f in before['frames'].items())
assert old_unchanged and before['notes'] == after['notes'], 'push 改变旧帧或 notes'
assert len(after['frames']) == len(before['frames']) + 1
output = (out / 'after-push.md').read_text()
probes = ['P1 底帧持有完整目标', '四红内容债逐条过', '| V_INTAKE instructions+门 |', '[resumed · parallel]']
missing = [p for p in probes if p in raw.decode() and p not in output]
assert not missing, missing
owners = {'1': ['P1 底帧持有完整目标'], '16': ['四红内容债逐条过'],
          '19': ['| V_INTAKE instructions+门 |'], '29': ['[resumed · parallel]']}
for owner, fragments in owners.items():
    remarks = '\n'.join(before['frames'][owner]['content']['remarks'].values())
    for fragment in fragments:
        if fragment in raw.decode(): assert fragment in remarks, (owner, fragment)
import re
sections = re.split(r'(?m)^## \[OP-(\d+)\].*\n', raw.decode())
checked_lines = 0
for i in range(1, len(sections), 2):
    owner, body = str(int(sections[i])), sections[i + 1]
    remarks = '\n'.join(before['frames'][owner]['content']['remarks'].values())
    for line in body.splitlines():
        stripped = line.strip()
        if stripped.startswith('|') or any(token in stripped for token in ['[resumed', '[pending', '[stopped', '[terminated early', '{{ skill_dir }}']):
            encoded = stripped.replace('\\', '\\\\').replace('{{', '\\{{')
            assert encoded in remarks, (owner, stripped)
            checked_lines += 1
reimport = str(out / 'reimport.yaml')
request('import', {'md': str(out / 'after-push.md'), 'out': reimport}, 'reimport')
again = json.loads(request('export', {'format': 'json'}, 'reimport-model', target=reimport)['text'])
for owner, frame in after['frames'].items():
    assert again['frames'][owner]['content']['remarks'] == frame['content']['remarks'], owner
assert again['notes'] == after['notes'], '全局 notes 往返改变'
report = {
    'input_sha256': hashlib.sha256(raw).hexdigest(),
    'input_lines': len(raw.splitlines()),
    'input_frames': len(before['frames']),
    'push_parent': parent,
    'new_id': max(map(int, after['frames'])),
    'old_frames_and_notes_unchanged_after_push': old_unchanged,
    'import_warnings': imported['warnings'],
    'missing_probes': missing,
    'table_and_unknown_state_lines_checked_in_original_owner': checked_lines,
    'all_frame_remarks_and_global_notes_survive_reimport': True,
    'original_file_unchanged': source.read_bytes() == raw,
    'scope': '此前丢失内容按原帧保留为备注；全部备注/全局 note 再导入保持相等。并非原文件字节或全部核心字段语义一致证明；FOCUS 不作为判据。',
}
(out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps({k: v for k, v in report.items() if k != 'import_warnings'}, ensure_ascii=False, indent=2))
print('import warnings:', len(imported['warnings']))

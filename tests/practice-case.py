#!/usr/bin/env python3
"""执行SKILL案例并记录真实输出；业务事件为模拟设定。"""
import argparse
import difflib
import json
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument('output', type=Path)
parser.add_argument('--binary', default='working-stack')
args = parser.parse_args()
binary = str(Path(shutil.which(args.binary) or args.binary).resolve())
source = Path(__file__).resolve().parents[1] / 'skills/working-stack/references/practice.md'
args.output.mkdir(parents=True, exist_ok=False)
text = source.read_text()
sections = re.split(r'(?m)^## ', text)[1:]
results = []

for mode, target in [('managed', 'practice'), ('single', './practice.yaml')]:
    destination = args.output / mode
    destination.mkdir()
    with tempfile.TemporaryDirectory() as temporary:
        cwd = Path(temporary)
        events = []
        previous_md = previous_yaml = ''
        first_goals = None
        old_push = None
        implementation_push = None
        sections_saved = []

        def run(command, payload=None):
            process = subprocess.run(command, input=payload or '', cwd=cwd,
                                     text=True, capture_output=True)
            if process.returncode:
                raise RuntimeError(' '.join(command[1:]) + '\n' + process.stderr)
            return process.stdout

        for number, section in enumerate(sections, 1):
            title = section.splitlines()[0]
            blocks = re.findall(r'```(sh|json)\n(.*?)\n```', section, re.S)
            if not blocks:
                continue
            before = len(events)
            for language, body in blocks:
                commands = [body] if language == 'json' else body.splitlines()
                for original in commands:
                    if not original.strip() or original.startswith('#'):
                        continue
                    if language == 'json':
                        payload = json.loads(original)
                        payload['stack'] = target
                        output = run([binary], json.dumps(payload, ensure_ascii=False))
                    else:
                        command = shlex.split(original)
                        if command[0] == 'working-stack':
                            command[0] = binary
                            command[1] = target
                        elif command[0] == 'python3':
                            command[0] = sys.executable
                        output = run(command)
                    events.append({'command': original, 'stdout': output, 'exit_code': 0})
            prefix = '{:02d}'.format(number)
            model = json.loads(run([binary, target, 'export', 'json']))
            md = run([binary, target, 'export', 'md'])
            yaml = run([binary, target, 'export', 'yaml'])
            context = run([binary, target, 'show', 'context'])
            (destination / (prefix + '.context.md')).write_text(context)
            (destination / (prefix + '.md')).write_text(md)
            (destination / (prefix + '.yaml')).write_text(yaml)
            (destination / (prefix + '.json')).write_text(json.dumps(model, ensure_ascii=False, indent=2))
            (destination / (prefix + '.md.diff')).write_text(''.join(difflib.unified_diff(
                previous_md.splitlines(True), md.splitlines(True), fromfile='previous.md', tofile='current.md')))
            (destination / (prefix + '.yaml.diff')).write_text(''.join(difflib.unified_diff(
                previous_yaml.splitlines(True), yaml.splitlines(True), fromfile='previous.yaml', tofile='current.yaml')))
            (destination / (prefix + '.commands.json')).write_text(json.dumps(events[before:], ensure_ascii=False, indent=2))
            snapshot = cwd / ('stage-' + prefix + '.md')
            snapshot.write_text(md)
            imported = './stage-' + prefix + '.yaml'
            run([binary, 'import', str(snapshot), '-o', imported])
            restored = json.loads(run([binary, imported, 'export', 'json']))
            assert restored['notes'] == model['notes']
            assert '全局范围约束' in context and '验收仍由用户决定' in context
            if '3' in model['frames']:
                push3 = [event for event in model['frames']['3']['log'] if event['action'] == 'push'][0]
                if implementation_push is not None:
                    assert push3 == implementation_push
                implementation_push = push3
            if title.startswith('4A.'):
                assert '先核对材料' in model['frames']['1']['plan'][1]['text']
                assert model['frames']['3']['plan'][3]['text'] == '核对材料后重新运行直接测试'
                assert '顺序裁决' in model['frames']['1']['content']['remarks']
                assert '继续理由' in model['frames']['3']['content']['remarks']
            if first_goals is None:
                first_goals = model['frames']['1']['goal']
            assert model['frames']['1']['goal'] == first_goals
            if '5' in model['frames']:
                push = [entry for entry in model['frames']['5']['log'] if entry['action'] == 'push'][0]
                if old_push is not None:
                    assert push == old_push
                old_push = push
                assert model['frames']['5']['parent'] == (1 if title.startswith('5A.') else 3)
            if title.startswith('5A.'):
                assert [model['frames'][str(i)]['parent'] for i in [2, 3, 5, 6]] == [1, 1, 1, 1]
                assert all(model['frames'][str(i)]['status']['kind'] == 'paused' for i in [2, 3, 5, 6])
            if title.startswith('7.'):
                assert model['frames']['1']['status']['kind'] == 'paused'
                assert model['frames']['1']['plan'][2]['progress']['kind'] == 'blocked'
            sections_saved.append({'prefix': prefix, 'title': title})
            previous_md, previous_yaml = md, yaml

        assert len(model['frames']) == 6
        assert [model['frames'][str(i)]['status']['kind'] for i in range(1, 7)] == [
            'done', 'done', 'done', 'expired', 'done', 'expired']
        assert any(event['action'] == 'correct' for event in model['frames']['3']['log'])
        assert model['frames']['5']['content']['principles'][0]['fate'] == {'kind': 'released_to', 'id': 1}
        assert model['frames']['5']['content']['principles'][1]['fate']['kind'] == 'kept_local'
        assert model['frames']['3']['content']['principles'][0]['fate']['kind'] == 'reclassified'
        assert model['frames']['1']['content']['principles'][0]['fate']['kind'] == 'kept_local'
        assert run([binary, target, 'validate']).strip() == 'OK'
        (destination / 'timeline.md').write_text(run([binary, target, 'timeline']))
        (destination / 'timeline.json').write_text(run([binary, target, 'timeline', '--json']))
        (destination / 'commands.json').write_text(json.dumps(events, ensure_ascii=False, indent=2))
        (destination / 'final.md').write_text(previous_md)
        (destination / 'final.yaml').write_text(previous_yaml)
        git = cwd / '.working-stack' / '.git'
        if mode == 'managed':
            assert git.exists()
            tracked = run(['git', '-C', str(git.parent), 'ls-files']).splitlines()
            assert tracked == ['practice.yaml']
            assert run(['git', '-C', str(git.parent), 'status', '--porcelain']) == ''
        else:
            assert not git.exists() and not (cwd / '.git').exists()
        result = {'mode': mode, 'commands': len(events), 'frames': 6, 'stages': sections_saved,
                  'root_goals_preserved': True, 'reparent_keeps_push_history': True,
                  'awaiting_acceptance_stays_open': True, 'route_change_updates_parent': True,
                  'all_stage_notes_roundtrip': True, 'validate': 'OK'}
        (destination / 'report.json').write_text(json.dumps(result, ensure_ascii=False, indent=2))
        results.append(result)
(args.output / 'report.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(json.dumps(results, ensure_ascii=False, indent=2))

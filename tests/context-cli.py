import json
from pathlib import Path
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory() as directory:
    stack = Path(directory, 'stack.yaml')
    def call(op, args=None):
        p = subprocess.run([binary], input=json.dumps(dict(stack=str(stack), op=op, args=args or {}, reason='模拟上下文测试')),
                           text=True, capture_output=True, timeout=10)
        assert p.returncode == 0, p.stderr
        return json.loads(p.stdout)
    call('init', {'title':'上下文', 'prefix':'T'})
    call('push', {'title':'根', 'goal':['可用','用户验收']})
    call('push', {'parent':'T-01', 'title':'子', 'goal':'子目标'})
    call('update', {'id':'T-02', 'plan_add':['具体下一步'], 'plan_set':['0=in_progress'], 'remark':['审批=用户未验收']})
    before = stack.read_bytes()
    context = call('show', {'what':'context'})['text']
    assert all(s in context for s in ['T-01 → T-02','FOCUS：具体下一步','2.plan[0]','用户未验收','用户验收'])
    p = subprocess.run([binary, str(stack), 'show', 'context'], input='', text=True, capture_output=True)
    assert p.returncode == 0 and p.stdout.rstrip('\n') == context.rstrip('\n')
    assert stack.read_bytes() == before
    call('pause', {'id':'T-02'})
    before = stack.read_bytes()
    assert '焦点：无' in call('show', {'what':'context'})['text']
    assert '所选分支' in call('show', {'what':'context','id':'T-02'})['text']
    assert stack.read_bytes() == before
    p = subprocess.run([binary, str(stack), 'show', 'context', 'T-99'], input='', text=True, capture_output=True)
    assert p.returncode != 0 and stack.read_bytes() == before
    p = subprocess.run([binary, str(stack), 'update', '--help'], input='', text=True, capture_output=True)
    assert p.returncode == 0 and '--remark' in p.stdout and '替换旧结果' in p.stdout and 'show context' in p.stdout
    p = subprocess.run([binary, str(stack), 'update', 'T-02', '--remark', '正文', '-r', '模拟错误'], input='', text=True, capture_output=True)
    assert p.returncode != 0 and "--remark '当前状态=" in p.stderr and stack.read_bytes() == before
    print('context CLI/JSON readonly, selection, help and remark diagnostics passed')

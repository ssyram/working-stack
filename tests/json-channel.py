import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request

binary = str(Path(sys.argv[1]).resolve())
checks = 0

def check(condition, label):
    global checks
    assert condition, label
    checks += 1
    print('PASS', label)

with tempfile.TemporaryDirectory(prefix='ws-json-') as directory:
    stack = str(Path(directory, 'stack.yaml'))
    def cli(*args, data=''):
        return subprocess.run([binary, *args], input=data, text=True, capture_output=True, cwd=directory, timeout=10)
    def request(op, args=None, reason='测试原因', **extra):
        payload = dict(stack=stack, op=op, args=args or {}, reason=reason, **extra)
        result = cli(data=json.dumps(payload, ensure_ascii=False, indent=2))
        assert result.returncode == 0, (payload, result.stderr)
        return json.loads(result.stdout)
    def state():
        result = cli(stack, 'export', 'json')
        assert result.returncode == 0, result.stderr
        return json.loads(result.stdout)
    request('init', {'prefix': 'J', 'title': '测试栈'})
    request('push', {'title': '根', 'goal': '字符串目标', 'origin': '根来源'})
    check(state()['frames']['1']['goal'] == ['字符串目标'], '多行 JSON / 字符串 goal 实际保存')
    check(state()['frames']['1']['origin'] == ['根来源'], '根帧 origin 实际保存')
    request('push', {'parent': 'J-01', 'title': '子', 'goal': ['一', '二'], 'origin': ['来源一', '来源二']})
    check(state()['frames']['2']['goal'] == ['一', '二'], '数组 goal 实际保存')
    check(state()['frames']['2']['origin'] == ['来源一', '来源二'], '数组 origin 实际保存')
    request('update', {'id': 'J-02', 'goal': [], 'origin': [], 'title': ''})
    f = state()['frames']['2']
    check(f['goal'] == [] and f['origin'] == [] and f['title'] == '', '空数组及空标题清空')
    request('update', {'id': 'J-02', 'title': '--不是参数', 'plan_add': ['步骤']})
    check(state()['frames']['2']['title'] == '--不是参数', '以横线开头的值保持原样')
    request('update', {'id': 'J-02', 'plan_set': ['0=in_progress'], 'plan_text': ['0=新步骤'], 'plan_add_child': ['0=子步骤']})
    plan = state()['frames']['2']['plan'][0]
    check(plan['text'] == '新步骤' and plan['children'][0]['text'] == '子步骤', '计划正文和嵌套内容实际保存')
    request('update', {'id': 'J-02', 'principle_add': ['原则'], 'remark': ['说明=正文'], 'result_add': ['结果'], 'side_add': ['副作用']})
    request('update', {'id': 'J-02', 'settle': ['0=kept_local']})
    c = state()['frames']['2']['content']
    check(c['results'] == ['结果'] and c['side_effects'] == ['副作用'] and c['remarks']['说明'] == '正文', '正文各栏目保存')
    check(c['principles'][0]['fate'] is not None, '原则结算保存')
    request('note', {'key': 'k', 'body': '正文', 'meta': ['appendix', 'order:10', 'checklist']}, reason='人指定原因')
    check(state()['log'][-1]['detail'] == '人指定原因', 'note 记录调用者原因')
    check(state()['notes']['k']['meta'] == ['appendix', 'order:10', 'checklist'], 'note meta 数组实际保存')
    request('note', {'key': 'k', 'body': ''})
    check(state()['notes']['k']['body'] == '', 'note 空正文写入')
    before = Path(stack).read_bytes()
    request('note', {'key': 'k'})
    check(Path(stack).read_bytes() == before, 'note 读取不改文件')
    for payload in [
        {'stack': stack, 'op': 'update', 'args': {'id': 'J-02', 'titel': '错字'}, 'reason': '测试'},
        {'stack': stack, 'op': 'update', 'args': {'id': 'J-02', 'goal': ['好', 4]}, 'reason': '测试'},
    ]:
        result = cli(data=json.dumps(payload))
        check(result.returncode != 0 and Path(stack).read_bytes() == before, '非法字段/类型不改栈')
    result = cli(stack, 'push', '--title', '不能执行', '-r', '回退', data='{\n invalid')
    check(result.returncode != 0 and Path(stack).read_bytes() == before, '解析失败不回退执行 CLI')
    for what in ['overview', 'tree', 'focus', 'frame', 'log', 'timeline', 'notes', 'pops']:
        args = {'what': what}
        if what in ['frame', 'log']: args['id'] = 'J-02'
        check(isinstance(request('show', args)['text'], str), 'show ' + what)
    request('pause', {'id': 'J-02'})
    request('switch', {'id': 'J-02'})
    request('pause', {'id': 'J-02'})
    request('resume', {'id': 'J-02'})
    check(state()['frames']['2']['status']['kind'] == 'active', 'pause/switch/resume 实际状态')
    request('pop', {'id': 'J-02', 'summary': '完成'})
    check(state()['frames']['2']['status']['kind'] == 'done', 'pop 实际状态')
    request('update', {'id': 'J-02', 'reopen': 'active'})
    check(state()['frames']['2']['status']['kind'] == 'active', 'reopen 实际状态')
    request('pop', {'id': 'J-02', 'expired': True, 'summary': '停止', 'residual': ['剩余'], 'pos': ['正向']})
    check(state()['frames']['2']['status']['kind'] == 'expired', 'expired 实际状态')
    check(request('validate')['text'] == 'OK', 'validate')
    check(json.loads(request('export', {'format': 'json'})['text']) == state(), 'export JSON 内容')
    md = str(Path(directory, 'export.md'))
    request('export', {'format': 'md', 'out': md})
    imported = str(Path(directory, 'import.yaml'))
    request('import', {'md': md, 'out': imported})
    check(Path(imported).exists(), 'import 参数可用')
    candidate = str(Path(directory, 'candidate.yaml'))
    Path(candidate).write_bytes(Path(stack).read_bytes())
    request('apply', {'candidate': candidate})
    check(state()['frames']['1']['title'] == '根', 'apply 候选可用')
    os.environ['EDITOR'] = '/usr/bin/true'
    request('edit')
    check(state()['frames']['1']['title'] == '根', 'edit 非交互编辑器可用')
    check(request('list')['ok'], 'list')
    check(cli(stack, 'show', 'tree').returncode == 0, '原 CLI 保留')
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0)); port = sock.getsockname()[1]
    check(cli('managed', 'init', '--prefix', 'J').returncode == 0, '托管模式初始化')
    check(cli('managed', 'push', '--title', '托管根', '-r', '建根').returncode == 0, '托管模式原 CLI 写入')
    server = subprocess.Popen([binary], stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, cwd=directory, text=True)
    server.stdin.write(json.dumps({'stack': 'managed', 'op': 'web', 'args': {'port': port}}))
    server.stdin.close()
    try:
        url = f'http://127.0.0.1:{port}'
        for _ in range(100):
            try:
                with urllib.request.urlopen(url + '/api/state', timeout=1) as response: json.load(response)
                break
            except OSError: time.sleep(.05)
        body = json.dumps({'op': 'update', 'args': {'id': 'J-01', 'goal': '网页目标'}, 'reason': '网页原因'}).encode()
        with urllib.request.urlopen(urllib.request.Request(url + '/api/op', data=body), timeout=3) as response:
            data = json.load(response)
        check(data['ok'] and data['stack']['frames']['1']['goal'] == ['网页目标'], '托管 JSON web port + POST 共用转换、返回实际文件状态且无 stdin 阻塞')
    finally:
        server.terminate(); server.wait(timeout=5)
print(f'{checks} assertions passed')

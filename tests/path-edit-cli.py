import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request

root = Path(__file__).resolve().parents[1]
binary = str(Path(sys.argv[1]).resolve())
checks = 0

def check(condition, label):
    global checks
    assert condition, label
    checks += 1
    print('PASS', label)

with tempfile.TemporaryDirectory(dir=root / 'target', prefix='path-edit-') as directory:
    stack = str(Path(directory, 'stack.yaml'))
    def call(op, args=None, reason='修改计划', target=None, good=True):
        request = dict(stack=target or stack, op=op, args=args or {}, reason=reason)
        result = subprocess.run([binary], input=json.dumps(request, ensure_ascii=False), text=True, capture_output=True, timeout=15)
        if not good: return result
        assert result.returncode == 0, (request, result.stderr)
        return json.loads(result.stdout)
    def get(path, target=None): return call('get', {'path':path}, target=target)['value']
    def state(): return json.loads(call('export', {'format':'json'})['text'])
    call('init', {'prefix':'P','title':'路径编辑测试'})
    call('push', {'title':'根'})
    before = Path(stack).read_bytes()
    check(get('P-01.plan') == [] and Path(stack).read_bytes() == before, 'get 原生 JSON 值且不写文件')
    call('insert', {'path':'P-01.plan[0]', 'value':'B'}, reason='初始步骤')
    call('insert', {'path':'P-01.plan[0]', 'value':'A'})
    call('insert', {'path':'P-01.plan', 'value':'D'})
    call('insert', {'path':'P-01.plan[2]', 'value':'C'})
    check([p['text'] for p in get('P-01.plan')] == ['A','B','C','D'], '前/中/末/列表追加插入')
    check(state()['frames']['1']['log'][-4]['detail'].startswith('初始步骤') and '变更字段：plan' in state()['frames']['1']['log'][-4]['detail'], '插入记录调用者原因')
    call('set', {'path':'P-01.plan[0].children','value':['子项', {'text':'二级','progress':'done','children':['三级']}]})
    check(get('P-01.plan[0].children[1].children[0].text') == '三级', '嵌套短字符串/对象列表')
    call('set', {'path':'P-01.plan[0].progress','value':'in_progress'})
    child = get('P-01.plan[0].children')
    call('set', {'path':'P-01.plan[0]','value':'新正文'})
    check(get('P-01.plan[0].progress')['kind'] == 'in_progress' and get('P-01.plan[0].children') == child, '字符串编辑保留状态与子项')
    call('set', {'path':'P-01.plan[0].children[0].text','value':'改后子项'})
    check(get('P-01.plan[0].children[0].text') == '改后子项', '按路径原位编辑正文')
    call('remove', {'path':'P-01.plan[0].children[0]'})
    check(len(get('P-01.plan[0].children')) == 1, '删除嵌套项')
    call('move', {'from':'P-01.plan[0]','path':'P-01.plan[3]'})
    check([p['text'] for p in get('P-01.plan')] == ['B','C','D','新正文'], '移动按移除后的目标位置')
    count = len(state()['frames']['1']['log'])
    call('set', {'path':'P-01.plan[0].text','value':'B'})
    check(len(state()['frames']['1']['log']) == count, '无净变化不造事件')
    before = Path(stack).read_bytes()
    for op, args in [('insert', {'path':'P-01.plan[100]','value':'越界'}),
                     ('move', {'from':'P-01.plan[0]','path':'P-01.plan[100]'}),
                     ('set', {'path':'P-01.plan[0].text','value':3}),
                     ('set', {'path':'P-01.status','value':{'kind':'paused','reason':'绕过'}})]:
        result = call(op,args,good=False)
        check(result.returncode != 0 and Path(stack).read_bytes() == before, op + ' 失败无副作用')
    call('set', {'path':'P-01.plan','value':[{'text':'已完成','progress':'done'},'剩余']})
    check(get('P-01.plan[0].progress')['kind'] == 'item_done' and len(get('P-01.plan')) == 2, '整段重拟/拆分并保留声明状态')
    call('insert', {'path':'P-01.plan[1].children','value':'下一步'})
    check(get('P-01.plan[1].children[0].text') == '下一步', '直接向 children 列表插入')
    call('insert', {'path':'P-01.goal','value':'目标'})
    call('set', {'path':'P-01.goal[0]','value':'细化目标'})
    check(get('P-01.goal[0]') == '细化目标', '目标按相同路径编辑')
    call('insert', {'path':'P-01.results','value':'结果'})
    check(get('P-01.content.results[0]') == '结果', '内容字段简写与完整路径同值')
    call('note', {'key':'k','body':'旧正文'})
    call('set', {'path':'note:k.body','value':'新正文'},reason='修改共享正文')
    call('insert', {'path':'note:k.meta','value':'未知提示'})
    check(get('note:k.meta[0]') == '未知提示' and state()['log'][-2]['detail'].startswith('修改共享正文') and 'notes.k.body' in state()['log'][-2]['detail'], 'note 修改记录栈级原因')
    call('set', {'path':'P-01.goal[0]','value':'参见 {{1.plan[0]}}'})
    before = Path(stack).read_bytes()
    result = call('set', {'path':'P-01.plan','value':[]}, good=False)
    check(result.returncode != 0 and Path(stack).read_bytes() == before, '已有引用失效由保存校验拒绝')
    call('push', {'parent':'P-01','title':'另一个帧'})
    call('move', {'from':'P-01.plan[1]','path':'P-02.plan'})
    check(get('P-02.plan[0].text') == '剩余', '跨帧移动完整项')
    check(state()['frames']['2']['log'][-1]['detail'] == state()['frames']['1']['log'][-1]['detail'], '一次移动给两帧记录相同原因')
    result = subprocess.run([binary,stack,'insert','P-02.plan[0]','--value','"CLI项"','-r','CLI补项'],input='',text=True,capture_output=True)
    check(result.returncode == 0 and get('P-02.plan[0].text') == 'CLI项', '原 CLI 参数通道可用')
    md = str(Path(directory,'stack.md'))
    call('export', {'format':'md','out':md})
    call('insert', {'path':'P-02.plan','value':'MD项'}, target=md)
    check(get('P-02.plan[2].text',target=md) == 'MD项', 'Markdown 文件的路径插入可用')
    with socket.socket() as sock:
        sock.bind(('127.0.0.1',0)); port = sock.getsockname()[1]
    server = subprocess.Popen([binary,stack,'web','--port',str(port)],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    try:
        url = 'http://127.0.0.1:' + str(port)
        for _ in range(100):
            try:
                urllib.request.urlopen(url+'/api/state',timeout=1).close(); break
            except OSError: time.sleep(.05)
        payload = json.dumps(dict(op='insert',args={'path':'P-02.plan[0]','value':'网页项'},reason='网页补项')).encode()
        with urllib.request.urlopen(urllib.request.Request(url+'/api/op',data=payload),timeout=3) as response:
            data=json.load(response)
        check(data['ok'] and data['stack']['frames']['2']['plan'][0]['text'] == '网页项', 'Web 同一 JSON 操作及提交链可用')
        payload = json.dumps(dict(op='get',args={'path':'P-02.plan[0]'})).encode()
        with urllib.request.urlopen(urllib.request.Request(url+'/api/op',data=payload),timeout=3) as response:
            data=json.load(response)
        check(data['ok'] and data['value']['text'] == '网页项', 'Web get 直接返回相同结构值')
    finally:
        server.terminate(); server.wait(timeout=5)
print(str(checks) + ' path-edit assertions passed')

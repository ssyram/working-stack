import json
from pathlib import Path
import subprocess
import sys
import tempfile

root=Path(__file__).resolve().parents[1]
binary=str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(dir=root/'target',prefix='timeline-') as d:
    stack=str(Path(d,'stack.yaml'))
    def call(op,args=None):
        payload=dict(stack=stack,op=op,args=args or {},reason='时间线测试')
        r=subprocess.run([binary],input=json.dumps(payload),text=True,capture_output=True,timeout=10)
        assert r.returncode==0,r.stderr
        return json.loads(r.stdout)
    call('init',{'title':'时序','prefix':'T'})
    call('push',{'title':'根'})
    call('push',{'parent':'T-01','title':'子'})
    call('update',{'id':'T-02','plan_add':['一步']})
    call('note',{'key':'k','body':'全局'})
    call('pop',{'id':'T-02','summary':'完成'})
    before=Path(stack).read_bytes()
    timeline=call('timeline')
    rows=timeline['events']
    assert len(rows)==5,rows
    assert rows[0]['action']=='push' and rows[0]['frame']==1
    assert rows[-1]['action']=='pop' and rows[-1]['frame']==2
    assert any(row['frame'] is None for row in rows)
    assert all(row['time_known'] and '.' in row['at'] for row in rows)
    assert len(set(row['at'] for row in rows))==5
    assert Path(stack).read_bytes()==before
    r=subprocess.run([binary,stack,'timeline'],input='',text=True,capture_output=True)
    assert r.returncode==0 and r.stdout.startswith('# 事件时间线')
    r2=subprocess.run([binary,stack,'show','timeline'],input='',text=True,capture_output=True)
    assert r2.returncode==0 and r2.stdout.rstrip('\n')==r.stdout.rstrip('\n')
    print('9 timeline CLI assertions passed')

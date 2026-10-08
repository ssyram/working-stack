import json
from pathlib import Path
import subprocess
import sys
import tempfile

root=Path(__file__).resolve().parents[1]
binary=str(Path(sys.argv[1]).resolve())
checks=0

def check(condition,label):
    global checks
    assert condition,label
    checks+=1
    print('PASS',label)

with tempfile.TemporaryDirectory(dir=root/'target',prefix='trial-fix-') as directory:
    stack=str(Path(directory,'stack.yaml'))
    def call(op,args=None,reason='本次业务变化'):
        payload=dict(stack=stack,op=op,args=args or {},reason=reason)
        result=subprocess.run([binary],input=json.dumps(payload,ensure_ascii=False),text=True,capture_output=True,timeout=10)
        assert result.returncode==0,(payload,result.stderr)
        return json.loads(result.stdout)
    def state():return json.loads(call('export',{'format':'json'})['text'])
    call('init',{'title':'回归','prefix':'R'})
    call('push',{'title':'根'})
    call('push',{'parent':'R-01','title':'子'})
    before=state()
    call('update',{'id':'R-02','plan_add':['一步','二步'],'plan_set':['0=in_progress'],
                   'result_add':['结果'],'remark':['依据=证据']},reason='补齐计划与依据')
    after=state();prefix=len(before['frames']['2']['log'])
    logs=after['frames']['2']['log']
    check(logs[:prefix]==before['frames']['2']['log'],'历史日志原样前缀')
    check(len(logs)==prefix+1,'五个字段级动作仅一条新增Update')
    detail=logs[-1]['detail']
    check(detail.startswith('补齐计划与依据') and all(name in detail for name in ['plan','content.results','content.remarks.依据']),'原因与变更字段自动生成')
    call('update',{'id':'R-02','plan_set':['0=ready','0=in_progress']})
    check(state()['frames']['2']['log']==logs,'单提交净变化为零不造日志')
    call('update',{'id':'R-02','principle_add':['规则一','规则二']})
    before=state()
    call('update',{'id':'R-02','settle':['0=release:R-01','1=reclass:results']},reason='原则按职责承接')
    after=state()
    check(after['frames']['2']['content']['principles'][0]['fate']=={'kind':'released_to','id':1},'正式release处置成功')
    check(after['frames']['1']['content']['principles'][0]['rule']=='{{2.principles[0]}}','接收帧保留结构化引用')
    check(after['frames']['2']['content']['results'][-1]=='{{2.principles[1]}}','reclass结果保留结构化引用')
    for frame in ['1','2']:
        check(len(after['frames'][frame]['log'])==len(before['frames'][frame]['log'])+1,'受影响帧各一条Update：'+frame)
    check(call('validate')['text']=='OK','所有新原则引用通过I8')
    md=str(Path(directory,'updated.md')); replay=str(Path(directory,'reimport.yaml'))
    call('export',{'format':'md','out':md})
    call('import',{'md':md,'out':replay})
    r=subprocess.run([binary,replay,'export','json'],input='',text=True,capture_output=True)
    restored=json.loads(r.stdout); original=state()
    if restored != original:
        for fid, frame in original['frames'].items():
            if restored['frames'].get(fid) != frame:
                print('模型差异帧',fid)
                for key in frame:
                    if restored['frames'][fid].get(key) != frame[key]:
                        print(key, 'before=',frame[key], 'after=',restored['frames'][fid].get(key))
    check(r.returncode==0 and restored==original,'新日志摘要与原则转存完整MD往返')
    call('pop',{'id':'R-02','summary':'子目标完成'})
    check(state()['frames']['2']['status']['kind']=='done','pop默认Done，无多余完成flag')
    before=state()['frames']['2']['log']
    call('update',{'id':'R-02','title':'更正后的子','reopen':'active'},reason='明确撤销误关闭')
    logs=state()['frames']['2']['log']
    check(logs[:len(before)]==before and [e['action'] for e in logs[len(before):]]==['update','correct'],'Update合并不丢Correct及旧历史')
    check('原 Closure' in logs[-1]['detail'] and '子目标完成' in logs[-1]['detail'],'Correct保留既有关闭说明')
    check('status' not in logs[-2]['detail'] and 'title' in logs[-2]['detail'],'内容更新不抢Correct解释状态变化')
    call('set',{'path':'R-02.plan[1].progress','value':'done'})
    check(state()['frames']['2']['plan'][1]['progress']['kind']=='item_done','计划状态路径编辑仍可用')
    manual=(root/'docs/ai-manual.md').read_text()
    check('--done' not in manual and '帧 reopen' in manual,'手册不误导完成flag/对象')
print(str(checks)+' skill-trial repair assertions passed')

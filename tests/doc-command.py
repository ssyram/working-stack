import json
from pathlib import Path
import os
import re
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
binary = str(Path(sys.argv[1]).resolve())
checks = 0

def check(value, label):
    global checks
    assert value, label
    checks += 1
    print('PASS', label)

env = os.environ.copy()
env['WS_DOCUMENTATION_PATH'] = str(root / 'docs')
with tempfile.TemporaryDirectory(dir=root / 'tests', prefix='doc-cwd-') as cwd:
    def run(*args, data=''):
        return subprocess.run([binary, *args], cwd=cwd, input=data, env=env, text=True, capture_output=True, timeout=10)
    initial = run('doc')
    check(initial.returncode == 0, '无需栈目标且 cwd 无关')
    check(initial.stdout == run('doc', '--list').stdout, 'doc 等同 doc --list')
    names = initial.stdout.splitlines()
    expected = ['ENTRY.md  (<-- the entry file)'] + sorted(p.relative_to(root / 'docs').as_posix()
        for p in (root / 'docs').rglob('*.md') if p.is_file() and p != root / 'docs/ENTRY.md')
    check(names == expected, '实际 ENTRY.md 置顶标注，其余只有自动排序名称')
    entry_source = (root / 'docs/ENTRY.md').read_text()
    entry = run('doc', 'ENTRY').stdout
    check(entry.startswith(entry_source), '入口原文在索引之前保留')
    for name in expected[1:]:
        assert '- [`' + name + '`](' + name + ') — `working-stack doc ' + name + '`' in entry, name
    check(entry.count('\n## 全量文档索引\n') == 1, '入口自动索引完整且不重复')
    check((root / 'docs/ENTRY.md').read_text() == entry_source, '读取入口不写回索引')
    check(run('doc', 'ai-manual').stdout == (root / 'docs/ai-manual.md').read_text(), 'manual 原文相等')
    check(run('doc', 'ai-manual.md').stdout == run('doc', 'ai-manual').stdout, '后缀可有可无')
    for args in [(), ('--help',)]:
        result = run(*args)
        check(result.returncode == 0 and 'working-stack doc ENTRY' in result.stdout, '裸命令/帮助推荐实际入口')
    with tempfile.TemporaryDirectory(dir=root / 'docs', prefix='doc-dynamic-') as nested:
        directory = Path(nested) / 'markdown-output'
        directory.mkdir()
        file = directory / 'README.md'
        body = '# 动态文档\n\n正文，没有末尾换行'
        file.write_text(body)
        name = file.relative_to(root / 'docs').as_posix()
        check(name in run('doc').stdout.splitlines(), '新增嵌套文档无需重编即可出现')
        check('working-stack doc ' + name in run('doc', 'ENTRY').stdout, '入口全量索引即时包含新增文档')
        result = run('doc', name)
        check(result.returncode == 0 and result.stdout == body, '嵌套文档原文，无额外换行')
        check(run('doc', name[:-3]).stdout == body, '嵌套省略后缀读取')
        result = run(data=json.dumps({'op': 'doc', 'args': {'name': name}}))
        check(result.returncode == 0 and json.loads(result.stdout)['text'] == body, 'JSON 输入读文档')
        result = run(data=json.dumps({'op': 'doc', 'args': {'list': True}}))
        check(name in json.loads(result.stdout)['text'].splitlines(), 'JSON 列表')
    check(name not in run('doc').stdout.splitlines(), '文档退出目录后列表即时变化')
    check('working-stack doc ' + name not in run('doc', 'ENTRY').stdout, '入口索引同步反映目录退出')
    check(run('doc', '不存在').returncode != 0, '不存在的文档报错')
    check(not (Path(cwd) / '.working-stack').exists(), '不创建工作栈')
    skill = (root / 'skills/working-stack/SKILL.md').read_text()
    frontmatter = skill.split('---', 2)[1]
    check('name: working-stack' in frontmatter and 'description:' in frontmatter, '附带 SKILL 元数据可识别')
    for name in sorted(set(re.findall(r'working-stack doc ([A-Za-z0-9_/.-]+)', skill))):
        check(run('doc', name).returncode == 0, 'SKILL 文档指引可读取：' + name)
print(f'{checks} assertions passed')

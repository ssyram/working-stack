import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
checks = 0

def check(value, label):
    global checks
    assert value, label
    checks += 1
    print('PASS', label)

base_env = os.environ.copy()
base_env.pop('WS_DOCUMENTATION_PATH', None)
with tempfile.TemporaryDirectory(dir=root / 'target', prefix='doc-dist-test-') as temporary:
    directory = Path(temporary)
    cwd = directory / 'unrelated'
    cwd.mkdir()
    def run(binary, *args, env=None):
        return subprocess.run([str(binary), *args], cwd=cwd, env=base_env if env is None else env,
            input='', text=True, capture_output=True, timeout=10)
    source_entry = (root / 'docs/ENTRY.md').read_text()
    suffix = '.exe' if sys.platform == 'win32' else ''
    binary_name = 'working-stack' + suffix
    debug = root / 'target/debug' / binary_name
    distributor = root / 'target/debug/examples' / ('distribute' + suffix)
    rendered_entry = run(debug, 'doc', 'ENTRY').stdout
    check(rendered_entry.startswith(source_entry) and '\n## 全量文档索引\n' in rendered_entry, 'debug 读取入口正文及动态索引，不依赖 cwd')
    alone = directory / 'alone'
    alone.mkdir()
    shutil.copy2(root / 'target/release' / binary_name, alone / binary_name)
    check(run(alone / binary_name, 'doc').returncode != 0, 'release 不回退到编译目录')
    package = directory / 'package'
    subprocess.run([str(distributor), 'package', str(package)], cwd=cwd, env=base_env, check=True)
    check(run(package / binary_name, 'doc', 'ENTRY').stdout == rendered_entry, 'Rust 发布目录从异 cwd 读取相同伴随入口/索引')
    skill = root / 'skills/working-stack/SKILL.md'
    check((package / 'skills/working-stack/SKILL.md').read_bytes() == skill.read_bytes(), '发布包附带语义 SKILL 原文')
    expected = sorted(p.relative_to(root / 'docs').as_posix() for p in (root / 'docs').rglob('*.md'))
    copied = sorted(p.relative_to(package / 'documentation').as_posix() for p in (package / 'documentation').rglob('*.md'))
    check(copied == expected, 'Rust 发布器复制全部 Markdown 及嵌套目录')
    for name in expected:
        assert (package / 'documentation' / name).read_bytes() == (root / 'docs' / name).read_bytes(), name
    moved = directory / 'moved'
    package.rename(moved)
    check(run(moved / binary_name, 'doc', 'ENTRY.md').stdout == rendered_entry, '整个发布目录搬动后仍可读取')
    if os.name != 'nt':
        link = cwd / binary_name
        link.symlink_to(moved / binary_name)
        check(run(link, 'doc', 'ENTRY').stdout == rendered_entry, '全局式 symlink 跟随真实二进制定位 documentation')
    override = directory / 'override'
    override.mkdir()
    (override / 'ENTRY.md').write_text('用户入口，无末尾换行')
    env = base_env.copy()
    env['WS_DOCUMENTATION_PATH'] = str(override)
    user_entry = '用户入口，无末尾换行\n\n## 全量文档索引\n\n'
    check(run(moved / binary_name, 'doc', 'ENTRY', env=env).stdout == user_entry, '环境变量覆盖 release 的入口和索引')
    check(run(debug, 'doc', 'ENTRY', env=env).stdout == user_entry, '环境变量覆盖 debug 的入口和索引')
    empty = directory / 'empty'
    empty.mkdir()
    env['WS_DOCUMENTATION_PATH'] = str(empty)
    check(run(moved / binary_name, 'doc', env=env).stdout == '', '显式来源为空时不混入其他来源')
    install_root = directory / 'rust-install'
    subprocess.run([str(distributor), 'install', '--root', str(install_root)], cwd=cwd, env=base_env, check=True)
    check(run(install_root / 'bin' / binary_name, 'doc', 'ENTRY').stdout == rendered_entry,
        '跨平台 Rust 安装器通过 Cargo 安装并复制伴随文档')
    check((install_root / 'bin/skills/working-stack/SKILL.md').read_bytes() == skill.read_bytes(), '安装器附带相同 SKILL')
    cargo_root = directory / 'cargo-only'
    subprocess.run(['cargo', 'install', '--path', str(root), '--locked', '--offline', '--root', str(cargo_root),
        '--target-dir', str(root / 'target')], env=base_env, check=True)
    installed = cargo_root / 'bin' / binary_name
    check(installed.exists(), '裸 Cargo 安装在隔离根生成二进制')
    check(not (cargo_root / 'bin/documentation').exists() and run(installed, 'doc').returncode != 0,
        '裸 Cargo 不冒充伴随文档安装')
    shutil.copytree(root / 'docs', cargo_root / 'bin/documentation')
    check(run(installed, 'doc', 'ENTRY').stdout == rendered_entry, '裸 Cargo 安装补伴随目录后可读')
print(f'{checks} distribution assertions passed')

# working-stack

以本层owner为主体、人/AI共用的工作状态树：继承目的、认识依据、剩余责任与下一步，不以AI交付代替用户理解/采用。用户主导时AI协助；明确委派的局部自治中主线程AI可在授权内裁决，不代签上层保留权。协作与复杂项状态填写见[`docs/owner-collaboration.md`](docs/owner-collaboration.md)。核心语言蓝本：[`playground.md`](playground.md)；设计：[`docs/principles.md`](docs/principles.md)（Q）+ [`docs/architecture.md`](docs/architecture.md)（D.root）+ [`docs/detailed-design.md`](docs/detailed-design.md)（函数合同）。

## 安装

Windows / Linux / macOS（已有 Rust/Cargo）：

```text
git clone https://github.com/ssyram/working-stack.git
cd working-stack
cargo run --locked --example distribute -- install
```

Rust 安装器通过 Cargo 安装二进制，再复制完整 docs 到旁边的 documentation。默认安装根为 CARGO_HOME 或用户 .cargo，支持 `--root ROOT`；对应 bin 目录需要在 PATH 中。

缺工具链时可用 `./install.sh`（macOS/Linux）或 `install.ps1`（Windows）引导；两者调用同一 Rust 安装器。运行工具本身不需要 Rust、Python 或 Shell。

使用入口：`working-stack doc ENTRY`。它先介绍用法，末尾动态列出全部其他文档的链接和 doc 指引；裸命令及 `--help` 同样给出这个入口。

附带的语义管理 skill：`skills/working-stack/SKILL.md`。安装器及发布目录会一并复制；按所用 Agent 宿主的 skills 目录安装，不自动覆盖你的全局提示词。

## 文档与发布目录

`WS_DOCUMENTATION_PATH` 优先覆盖；debug（`cargo build`）默认用编译时 docs；release（`cargo build --release`）默认用实际二进制旁的 documentation，不依赖调用时 cwd。

```text
cargo run --locked --example distribute -- package target/working-stack-release
```

Rust 发布器生成可搬动的二进制 + documentation + skills 目录，支持 `--target TARGET`（需要对应工具链）；不依赖平台 Shell。生成目录可压缩作为下载包。

裸 `cargo install --path . --locked` 只安装二进制，不复制 docs；需另带 documentation 或指定 WS_DOCUMENTATION_PATH。完整文档分发合同见 `docs/doc-command.md`。

## 当前边界

CLI、解析往返及文档分发有直接回归测试；跨平台安装使用同一 Rust 实现，目前实际运行测试在 macOS。Web 是本地可视化界面，尚未完成全部浏览器交互验收；公开源码不等于已部署多人在线服务。仓库暂未授予开源许可证，public 可见性本身不授权再分发。

## 三种目标（用户零负担区分）

| 用法 | 模式 | 说明 |
|---|---|---|
| `working-stack mytask …` | 托管 | `<cwd>/.working-stack/mytask.yaml` + 目录内自动 git（每次操作一个限定提交） |
| `working-stack path/stack.yaml …` | 文件 | native 承载（yaml/json），完整读写，无自动 git |
| `working-stack path/WS-MAIN.md …` | md 管道 | **非承载**：parse（import）→ 变换 → re-print（canonical 导出形）；只读查看零副作用 |

## 常用操作（变更必带 `-r` 原因，事件自动记入帧 log）

```sh
working-stack doc                     # 所选目录动态列表：ENTRY.md 首位，其他自动排序
working-stack doc ai-manual           # 读取 AI 手册，可写 ai-manual.md
working-stack mytask init --title '工作线' --prefix T
working-stack mytask push --title '子任务' --goal '目标' -r '进入原因'      # 省略 parent = 建根
working-stack mytask push T-01 --title '下钻' -r '原因'
working-stack mytask pause T-02 -r '等待外部回信'
working-stack mytask resume T-02 -r '回信已到'                              # 别名 switch
working-stack mytask pop T-03 --summary '结果一句话' --pos '正向交付' -r '收尾'
working-stack mytask pop T-03 --expired --summary '未到验收阶段' --residual '剩余义务→T-05' -r '裁决'
working-stack mytask update T-02 --plan-add '新步骤' --plan-set 0.1=in_progress -r '调整'
working-stack mytask get 'T-02.plan' --json
working-stack mytask insert 'T-02.plan[1]' --value '"新步骤"' -r '补项'
working-stack mytask set 'T-02.plan[0].text' --value '"新正文"' -r '细化'
working-stack mytask move 'T-02.plan' --from 'T-02.plan[0]' -r '移到末尾'
working-stack mytask update T-02 --settle 0=release:T-01 -r '原则上移'     # 严格祖先；接收帧物化承接
working-stack mytask update T-03 --reopen active -r '误记更正'             # Correct 事件保留旧关闭类别及摘要
working-stack mytask note checklist --body '共享清单' --meta appendix --meta order:10 -r '更新清单'
working-stack mytask apply candidate.yaml -r '整文件批量编辑'
working-stack mytask edit -r '手工整理'
```

## 查询面（AI 工作流：/recalibration、/occams-razor）

```sh
working-stack mytask show context    # Owner复工材料：导览/责任/依据/当前协作说明（已录入）
working-stack mytask show overview   # 根目标/焦点路径/TOP+FOCUS/Paused 分支/已 pop 计数
working-stack mytask show focus      # 焦点帧全节 + FOCUS 项
working-stack mytask show tree       # 栈状态树（TOP/TOP-PAUSED/PAUSED 派生标记）
working-stack mytask show frame T-02
working-stack mytask show timeline   # 跨帧事件时间线
working-stack mytask show log T-02 / notes / pops
working-stack mytask validate
working-stack mytask export md --out WS.md
working-stack import path/WS-MAIN.md -o mytask
working-stack mytask web             # http://127.0.0.1:7788（与 CLI 同一 API）
working-stack list
```

全部命令支持 `--json`（含 diff/版本指纹/warnings，供 AI 消费）。

完整Owner中心教程及真实MD/YAML/diff：[skills/working-stack/references/practice.md](skills/working-stack/references/practice.md)。人类状态按具体对象/版本/来源有据说明，不增理解枚举；授权内技术完成与用户保留的整体验收分开。

## 语义速查

- 状态：`Active`（在栈内）/`Paused reason`（分支头）/`Done|Expired Closure`（关闭，正向/反向/正交/未完义务 + 依据）。
- 焦点 = 唯一 Active tip（派生，不存储）；TOP/PAUSED = 焦点帧 FOCUS 项 Blocked 的渲染。
- 引用：`{{12.goal}}`、`{{note:k.body}}`、`{{12.plan[1]}}`、`{{12.log[3].detail}}`（canonical 为纯数字 id）。
- 不变量 I1–I10 保存前全量校验；非法结构 fail closed（exit 2）。
- 误记更正走 `reopen`（Correct 事件保留旧关闭类别及摘要，不是完整 Closure 快照）；已关闭 id 不复活（apply 转移检查兜底）。

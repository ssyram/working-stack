# 语义型 SKILL 与 ENTRY 全量索引

## 用户目的与实际产物

用户要求在仓库本地附带给工具使用者的 SKILL：参考 working-stack 概念和语义管理，不重复人工格式/日志维护；具体操作按需读 doc ENTRY，并要求 ENTRY 先解释使用方式再索引全部其他文档。

源码：`skills/working-stack/SKILL.md`。保留用户拥有、工作线/帧/计划、para与注意力、暂停/回合退出、目标/验收、成果/原则/备注、原则处置与历史等语义义务。只有 ENTRY、ai-manual、path-edit/README 三个最低限度的 doc 指引，不复制 CLI参数或 Markdown 模板。技能采用 name/description frontmatter，可由用户安装到其 Agent 宿主的 skills 目录；没有改全局提示词或 Agent 配置。

ENTRY.md 正文介绍如何使用入口与工具、安装及附带 skill。doc_catalog 对 ENTRY 追加所选目录的全量动态索引，包含文档链接和可复制 doc 指引；列表和索引复用同一扫描。其他文档仍原样返回，读取不写回入口，不制造静态目录账。源路径选择、原文例外与分发合同已同步 doc-command、architecture、AI manual、README。

Rust 安装器/发布目录复制 skills，Cargo源码包显式包含该目录。全局命令链接更新为 `target/working-stack-skill-entry/working-stack`。

## 运行依据

- Rust54/54通过，dev/examples/release构建成功。
- 文档24条断言：入口正文先于索引、索引覆盖全部其他文件、不重复且不写回、目录动态增减、其他文档原文返回、SKILL metadata及三个指引可读取。
- 分发15条断言：安装与发布附带相同SKILL，既有路径/搬动/symlink/env行为保持。
- 路径操作25/25、JSON38/38通过。
- Cargo package --list确认包含skills/working-stack/SKILL.md。
- 全局 doc ENTRY实测输出正文+全部其他文档索引；发布目录SKILL与源码逐字节相等。示例输出保留 `target/entry-with-index.md`。

SKILL文本的语义充分性由用户阅读判断；上述测试不冒充独立语义审查。现有print/import多栈边界未在本轮修复。实测平台仍为macOS。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约10（入口投影） |
| 修改代码行数 | 约25（测试及分发接线） |
| 删除代码行数 | 0 |
| 新增文档行数 | SKILL约55行，入口/合同说明若干 |
| 涉及文件数 | 约12，另有全局链接及示例输出 |
| 新增测试用例数 | 文档增强8条，分发2条 |
| 测试通过率 | Rust54/54、doc24/24、分发15/15、路径25/25、JSON38/38 |
| 发现 bug 数 | 0 |
| 修复 bug 数 | 0 |
| 迭代轮次 | 设计/实现各1 |

经验同步 .pi/AI-learning.md：薄SKILL承接语义，工具doc承担操作知识，目录索引复用扫描而非重复人工维护。

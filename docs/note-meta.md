# Note Meta：开放的打印提示列表

## 用户目的 → 设计性质

Meta 是 `[String]`：给打印器提供位置、顺序及 Appendix 提示，不定义 note 的业务类别或共享语义。读取者仅应用认识且在自身视图适用的提示；其余忽略，不报错、不改写或删除原字符串。

```haskell
type Meta = [String]
data Note = Note { meta :: Meta, body :: SourcefulText }
```

Rust 对应 `type Meta = Vec<String>`；native JSON/YAML 为字符串数组，缺省 `[]`。所有 note 仍为单份正文，可被任意帧引用，不需要共享开关。

## Markdown 打印器（职责：D.print）

本轮支持：`before:tree`、`after:tree`、`before:frames`、`after:frames`、`appendix`、`order:N`（N 为有符号整数）。位置缺省 Appendix；order 缺省 0。相同位置按 order 升序，再按 note key 排序；每份 note 仅打印一次。after:frames 在帧节后、栈变化记录前；Appendix 位于全文末尾。非 Appendix 组以“共享内容”为小节名。

顺序扫描 meta：每个适用的位置提示覆盖当前位置，每个可解析的 order 覆盖当前 order。未知提示（例如分类名称）、不存在的位置（如 after:missing）、格式不适用的 order 均忽略；字符串原样保存在 meta，仍参与持久化及机械 diff。没有额外校验器、重试或纠正。

`description`约定为全局导览，正文直接打印在#标题后，不输出共享内容/工作线说明包装，也不在别的位置重复。该用途使位置/顺序提示不适用，但原meta仍完整保留；其他notes布局不变。无附件的文件头散文导入为description、默认meta为`["before:tree"]`，不自动创建空导览。

导览可以解释本层owner及授权边界、全局材料位置；正文声明不是权限引擎。最新人类协作说明也可用单份note，但须绑定具体对象/版本/对话来源，不把meta当owner/理解状态字段。布局提示不决定内容效力；填写指导见owner-collaboration.md。

## 承载、引用与操作（叶子合同）

- CLI note 使用重复 `--meta`；JSON 使用 `args.meta` 字符串数组。省略 body 为读取；提供 body 为写入，meta 默认为空列表。
- 引用 `note:key.meta[i]` 得到第 i 个字符串；没有 `.kind/.shared` 字段。meta 是普通 String，不解析内部插值。
- Markdown普通note标题只显示可读名称，不展示JSON key/meta。内部信息在文末`## 格式信息`的`> notes: [...]`中；description附件另记`at_title:true`，仅是打印协议、不是核心Note字段，用于区分空的标题下导览与旧分节note。正文每行保留可恢复的两空格缩进（包括空行）及SourcefulText转义；导览的##字面行不会成为WS栏目。import按附件恢复原meta、空白及空正文；其他note按显示标题匹配，同名则按出现顺序。
- 既有 Markdown 的 `### note:key（类别，shared）` 输入仍可导入为普通提示列表；旧 native 对象不再符合新模型。本工作区当前栈显式迁移一次，不引入运行时双模型兼容。

## 合同推理与测试

description先作为标题下正文输出，附件携带at_title；其余notes过滤到唯一位置后排序。全过程不修改note，未知提示不改变布局也不消失。MD保留原引用表达，不将展开内容写回；context作为查询视图可展开。格式附件与正文缩进共同保证身份、meta及正文边界，旧分节description继续可读。

测试：默认 Appendix、所有位置、排序/同键序、冲突提示顺序、未知和不适用提示忽略且保留；JSON/YAML、MD 多轮往返；meta 下标引用；CLI/JSON/Web 仍共用操作链；真实 WS-MAIN 的备注与 notes 再导入保持。

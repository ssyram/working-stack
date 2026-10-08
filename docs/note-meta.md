# Note Meta：开放的打印提示列表

## 用户目的 → 设计性质

Meta 是 `[String]`：给打印器提供位置、顺序及 Appendix 提示，不定义 note 的业务类别或共享语义。读取者仅应用认识且在自身视图适用的提示；其余忽略，不报错、不改写或删除原字符串。

```haskell
type Meta = [String]
data Note = Note { meta :: Meta, body :: SourcefulText }
```

Rust 对应 `type Meta = Vec<String>`；native JSON/YAML 为字符串数组，缺省 `[]`。所有 note 仍为单份正文，可被任意帧引用，不需要共享开关。

## Markdown 打印器（owner D.print）

本轮支持：`before:tree`、`after:tree`、`before:frames`、`after:frames`、`appendix`、`order:N`（N 为有符号整数）。位置缺省 Appendix；order 缺省 0。相同位置按 order 升序，再按 note key 排序；每份 note 仅打印一次。after:frames 在帧节后、栈变化记录前；Appendix 位于全文末尾。非 Appendix 组以“共享内容”为小节名。

顺序扫描 meta：每个适用的位置提示覆盖当前位置，每个可解析的 order 覆盖当前 order。未知提示（例如分类名称）、不存在的位置（如 after:missing）、格式不适用的 order 均忽略；字符串原样保存在 meta，仍参与持久化及机械 diff。没有额外校验器、重试或纠正。

文件头散文导入为 `description` note，meta 为 `["before:tree"]`；其打印也走统一 note 机制，不依赖 key 的特殊行为。

## 承载、引用与操作（叶子合同）

- CLI note 使用重复 `--meta`；JSON 使用 `args.meta` 字符串数组。省略 body 为读取；提供 body 为写入，meta 默认为空列表。
- 引用 `note:key.meta[i]` 得到第 i 个字符串；没有 `.kind/.shared` 字段。meta 是普通 String，不解析内部插值。
- Markdown 正文 note 标题只显示可读名称，不展示JSON key/meta。内部信息集中于全文末尾 `## 格式信息`，以 `> notes: [...]` 引用行保存各note的key/meta；这不是业务内容。正文继续缩进和SourcefulText转义。import用附录匹配显示标题，恢复内部身份/提示；正文重排时按名称匹配，显示名重复则按同名出现顺序匹配。
- 既有 Markdown 的 `### note:key（类别，shared）` 输入仍可导入为普通提示列表；旧 native 对象不再符合新模型。本工作区当前栈显式迁移一次，不引入运行时双模型兼容。

## 合同推理与测试

布局扫描每次只更新派生的 `(位置,order)`；整个过程中不修改 note。过滤到唯一位置后排序，再输出全部正文和原 meta，因此未知提示不会改变布局，也不会消失。打印/导入使用末尾格式附录的完整JSON附件且正文缩进，避免提示含标点或正文标题改变结构。

测试：默认 Appendix、所有位置、排序/同键序、冲突提示顺序、未知和不适用提示忽略且保留；JSON/YAML、MD 多轮往返；meta 下标引用；CLI/JSON/Web 仍共用操作链；真实 WS-MAIN 的备注与 notes 再导入保持。

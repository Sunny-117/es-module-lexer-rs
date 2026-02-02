# ES Module Lexer 可视化架构图

## 1. 整体系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         用户代码                                 │
│  import { parse } from 'es-module-lexer';                        │
│  const [imports, exports] = parse(sourceCode);                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    API 层 (TypeScript)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   parse()    │  │    init      │  │  initSync    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  - 内存管理        - Wasm 初始化     - 同步初始化               │
│  - 编码转换        - 异步加载                                   │
│  - 结果提取                                                     │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    运行时选择层                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Wasm Engine  │  │ asm.js Engine│  │  Pure JS     │          │
│  │  (最快)      │  │  (CSP 兼容)  │  │  (备用)      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    核心解析引擎 (C)                              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  parse() - 主解析函数                                     │  │
│  │    ├─ Phase 1: Facade 模式 (纯模块快速解析)              │  │
│  │    └─ Phase 2: 完整解析 (处理所有语法)                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  tryParseImportStatement()                                │  │
│  │    ├─ 静态 import                                         │  │
│  │    ├─ 动态 import()                                       │  │
│  │    ├─ import.meta                                         │  │
│  │    ├─ import attributes                                   │  │
│  │    └─ source/defer phase                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  tryParseExportStatement()                                │  │
│  │    ├─ export default                                      │  │
│  │    ├─ export { ... }                                      │  │
│  │    ├─ export * from                                       │  │
│  │    └─ export var/let/const/function/class                │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 2. 数据结构关系图

```

┌─────────────────────────────────────────────────────────────────┐
│                    WebAssembly Memory                            │
├─────────────────────────────────────────────────────────────────┤
│  Source Code Buffer (UTF-16)                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ char16_t source[sourceLen]                                │  │
│  │ "import foo from 'bar'; export const x = 1;"             │  │
│  └──────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Analysis Data (动态分配区)                                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Import 链表                                               │  │
│  │ ┌─────────┐    ┌─────────┐    ┌─────────┐               │  │
│  │ │Import #1│───→│Import #2│───→│Import #3│───→ NULL      │  │
│  │ └─────────┘    └─────────┘    └─────────┘               │  │
│  │    ↓                                                      │  │
│  │ ┌─────────┐    ┌─────────┐                               │  │
│  │ │Attr #1  │───→│Attr #2  │───→ NULL                      │  │
│  │ └─────────┘    └─────────┘                               │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Export 链表                                               │  │
│  │ ┌─────────┐    ┌─────────┐    ┌─────────┐               │  │
│  │ │Export #1│───→│Export #2│───→│Export #3│───→ NULL      │  │
│  │ └─────────┘    └─────────┘    └─────────┘               │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

Import 结构详解:
┌─────────────────────────────────────────────────────────────────┐
│ struct Import {                                                  │
│   const char16_t* start;           ─┐                            │
│   const char16_t* end;              │ 指向 source buffer         │
│   const char16_t* statement_start; ─┤                            │
│   const char16_t* statement_end;    │                            │
│   const char16_t* attr_index;      ─┘                            │
│   const char16_t* dynamic;         // -1: static, -2: meta, >0: dynamic
│   bool safe;                       // 是否为安全字符串           │
│   enum ImportType import_ty;      // 1-7: 不同类型               │
│   struct Attribute* attributes;   ───→ Attribute 链表           │
│   struct Import* next;            ───→ 下一个 Import            │
│ }                                                                │
└─────────────────────────────────────────────────────────────────┘
```

## 3. 解析流程图

```
开始解析
   ↓
┌──────────────────────────────────────┐
│ 初始化状态                            │
│ - facade = true                       │
│ - openTokenDepth = 0                  │
│ - pos = source                        │
└──────────────────────────────────────┘
   ↓
┌──────────────────────────────────────┐
│ Phase 1: Facade 模式                  │
│ (只处理 import/export)                │
└──────────────────────────────────────┘
   ↓
   ch = *pos
   ↓
┌─────────┬─────────┬─────────┬─────────┐
│ ch='e'  │ ch='i'  │ ch=';'  │ 其他     │
│ export? │ import? │ 跳过    │ 非模块   │
└─────────┴─────────┴─────────┴─────────┘
   ↓         ↓         ↓         ↓
   │         │         │      facade=false
   │         │         │         ↓
   │         │         │    切换到 Phase 2
   ↓         ↓         ↓
tryParse  tryParse   继续
Export    Import
   ↓         ↓
   └────┬────┘
        ↓
   是否仍是 facade?
        ↓
    ┌───┴───┐
   Yes      No
    ↓        ↓
  继续   切换到 Phase 2
  Phase 1
```


## 4. Import 解析状态机

```
tryParseImportStatement()
   ↓
pos += 6  (跳过 "import")
   ↓
commentWhitespace()
   ↓
   ch = ?
   ↓
┌──────┬──────┬──────┬──────┬──────┬──────┐
│ '('  │ '.'  │ '"'  │ "'"  │ '{'  │ 其他  │
└──────┴──────┴──────┴──────┴──────┴──────┘
   ↓      ↓      ↓      ↓      ↓      ↓
动态    import  字符串  字符串  命名   标识符
import  .meta   import  import  import import
   ↓      ↓      ↓      ↓      ↓      ↓
   │      │      │      │      │      │
   │      │      └──┬───┘      │      │
   │      │         ↓          │      │
   │      │    readImportString│      │
   │      │         ↓          │      │
   │      │    检测 "with"     │      │
   │      │         ↓          │      │
   │      │    解析 attributes │      │
   │      │         ↓          │      │
   │      │    addImport()     │      │
   │      │                    │      │
   │      ↓                    ↓      ↓
   │   检测 "meta"          解析到   继续
   │      ↓                 "from"   解析
   │   addImport()            ↓
   │   (type=-2)         readImportString()
   │                           ↓
   ↓                      addImport()
openTokenStack.push()
   ↓
dynamicImportStack.push()
   ↓
尝试解析字符串字面量
   ↓
检测 ',' (attributes)
   ↓
addImport()
(type=2/5/7)
```

## 5. Export 解析决策树

```
tryParseExportStatement()
   ↓
pos += 6  (跳过 "export")
   ↓
commentWhitespace()
   ↓
   ch = ?
   ↓
┌────┬────┬────┬────┬────┬────┬────┬────┐
│ '{' │ '*' │ 'd' │ 'a' │ 'f' │ 'c' │ 'v' │ 'l' │
└────┴────┴────┴────┴────┴────┴────┴────┘
  ↓    ↓    ↓    ↓    ↓    ↓    ↓    ↓
  │    │    │    │    │    │    │    │
  │    │    │    │    │    │    │    │
  │    │ export export export export export export
  │    │ * as  default async function class var/let
  │    │                                    /const
  │    │    ↓    ↓    ↓    ↓    ↓    ↓
  │    │    │    │    │    │    │    │
  │    │    │    │    │    │    │    └─→ 解析变量声明
  │    │    │    │    │    │    │        (支持解构)
  │    │    │    │    │    │    │           ↓
  │    │    │    │    │    │    │        addExport()
  │    │    │    │    │    │    │
  │    │    │    │    │    │    └─→ 解析类声明
  │    │    │    │    │    │           ↓
  │    │    │    │    │    │        addExport()
  │    │    │    │    │    │
  │    │    │    │    │    └─→ 解析函数声明
  │    │    │    │    │           ↓
  │    │    │    │    │        addExport()
  │    │    │    │    │
  │    │    │    │    └─→ 检测 async function
  │    │    │    │           ↓
  │    │    │    │        解析函数名
  │    │    │    │           ↓
  │    │    │    │        addExport()
  │    │    │    │
  │    │    │    └─→ export default
  │    │    │           ↓
  │    │    │        检测后续语法
  │    │    │        (function/class/表达式)
  │    │    │           ↓
  │    │    │        addExport("default")
  │    │    │
  │    │    └─→ 解析 export list
  │    │           ↓
  │    │        readExportAs()
  │    │           ↓
  │    │        addExport()
  │    │
  │    └─→ export * from "module"
  │           ↓
  │        readExportAs()
  │           ↓
  │        检测 "from"
  │           ↓
  │        readImportString()
  │
  └─→ export { a, b as c }
         ↓
      循环解析标识符
         ↓
      readExportAs()
         ↓
      检测 "from"
         ↓
      可选: readImportString()
```


## 6. 正则表达式 vs 除法运算符判断流程

```
遇到 '/' 字符
   ↓
获取 lastToken = *lastTokenPos
   ↓
┌─────────────────────────────────────────────────────────┐
│ 判断条件 (任一满足 → 正则表达式)                         │
├─────────────────────────────────────────────────────────┤
│ 1. isExpressionPunctuator(lastToken)                     │
│    例: +, -, *, (, [, {, =, :, ;, ,, !, <, >, &, |      │
│    排除: . 后面跟数字 (如 5.0)                           │
│    排除: ++ 和 -- (如 x++)                               │
├─────────────────────────────────────────────────────────┤
│ 2. lastToken == ')' && isParenKeyword(...)               │
│    例: while(...) /regex/                                │
│        for(...) /regex/                                  │
│        if(...) /regex/                                   │
├─────────────────────────────────────────────────────────┤
│ 3. lastToken == '}' && isExpressionTerminator(...)       │
│    例: function() {} /regex/                             │
│        try {} finally {} /regex/                         │
│        class X {} /regex/                                │
├─────────────────────────────────────────────────────────┤
│ 4. isExpressionKeyword(lastTokenPos)                     │
│    例: return /regex/                                    │
│        throw /regex/                                     │
│        typeof /regex/                                    │
│        void /regex/                                      │
│        delete /regex/                                    │
│        new /regex/                                       │
│        in /regex/                                        │
│        instanceof /regex/                                │
├─────────────────────────────────────────────────────────┤
│ 5. lastToken == '/' && lastSlashWasDivision              │
│    例: a / b / c  (第二个 / 是除法)                      │
├─────────────────────────────────────────────────────────┤
│ 6. !lastToken (文件开头)                                 │
│    例: /regex/ at start                                  │
└─────────────────────────────────────────────────────────┘
   ↓
┌────────┴────────┐
│                 │
满足条件      不满足条件
   ↓                 ↓
regularExpression()  │
   ↓                 │
解析正则表达式       │
   ↓                 │
lastSlashWasDivision = false
                      ↓
                 检查特殊情况
                      ↓
              ┌──────┴──────┐
              │             │
         export default  break/continue
              /regex/        label
              ↓             ↓
         regularExpression()  regularExpression()
              ↓             ↓
         lastSlashWasDivision = false
                      ↓
              lastSlashWasDivision = true
                      ↓
                  继续解析
```

## 7. 括号/大括号匹配栈机制

```
OpenToken Stack (固定大小 1024)
┌─────────────────────────────────────────────────────────┐
│ openTokenStack[openTokenDepth]                           │
├─────────────────────────────────────────────────────────┤
│ Index │ Token Type      │ Position                       │
├───────┼─────────────────┼────────────────────────────────┤
│   0   │ AnyParen        │ pos of '('                     │
│   1   │ AnyBrace        │ pos of '{'                     │
│   2   │ ImportParen     │ pos of 'import('              │
│   3   │ Template        │ pos of '`'                     │
│   4   │ TemplateBrace   │ pos of '${'                    │
│   5   │ ClassBrace      │ pos of 'class {'               │
│  ...  │ ...             │ ...                            │
└─────────────────────────────────────────────────────────┘
        ↑
   openTokenDepth (当前栈深度)

操作流程:
┌─────────────────────────────────────────────────────────┐
│ 遇到开括号/大括号                                         │
│   ↓                                                      │
│ openTokenStack[openTokenDepth].token = token_type        │
│ openTokenStack[openTokenDepth].pos = lastTokenPos        │
│ openTokenDepth++                                         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ 遇到闭括号/大括号                                         │
│   ↓                                                      │
│ openTokenDepth--                                         │
│   ↓                                                      │
│ 检查 openTokenStack[openTokenDepth].token                │
│   ↓                                                      │
│ 根据类型执行相应操作                                      │
│   - ImportParen: 完成动态 import 解析                    │
│   - TemplateBrace: 继续解析模板字符串                    │
│   - 其他: 继续                                           │
└─────────────────────────────────────────────────────────┘

示例: import(foo ? 'a' : 'b')
┌─────────────────────────────────────────────────────────┐
│ 位置  │ 字符  │ 操作                │ openTokenDepth    │
├───────┼───────┼─────────────────────┼───────────────────┤
│   0   │ i     │ 检测到 import       │ 0                 │
│   6   │ (     │ push ImportParen    │ 1                 │
│  10   │ (     │ push AnyParen       │ 2                 │
│  18   │ )     │ pop AnyParen        │ 1                 │
│  19   │ )     │ pop ImportParen     │ 0                 │
│       │       │ 完成 import 解析    │                   │
└─────────────────────────────────────────────────────────┘
```


## 8. 动态 Import 解析流程

```
import('module')
   ↓
检测到 import(
   ↓
┌─────────────────────────────────────────────────────────┐
│ 1. 创建 Import 结构                                      │
│    addImport(startPos, pos, 0, dynamicPos)               │
│    import_write_head->import_ty = Dynamic (2)            │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 2. 压入栈                                                │
│    openTokenStack[openTokenDepth++] = ImportParen        │
│    dynamicImportStack[dynamicImportStackDepth++] = import│
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 3. 尝试解析字符串字面量                                   │
│    if (ch == '"' || ch == "'")                           │
│      stringLiteral(ch)                                   │
│      import_write_head->safe = true                      │
│      import_write_head->end = pos                        │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 4. 检测 import attributes                                │
│    if (ch == ',')                                        │
│      import_write_head->attr_index = pos                 │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 5. 继续解析直到遇到 ')'                                  │
│    在主循环中处理                                         │
└─────────────────────────────────────────────────────────┘
   ↓
遇到 ')'
   ↓
┌─────────────────────────────────────────────────────────┐
│ 6. 完成解析                                              │
│    openTokenDepth--                                      │
│    if (openTokenStack[openTokenDepth].token == ImportParen)│
│      cur_dynamic_import->statement_end = pos + 1         │
│      dynamicImportStackDepth--                           │
└─────────────────────────────────────────────────────────┘

特殊情况处理:
┌─────────────────────────────────────────────────────────┐
│ import(expr)  (非字符串字面量)                            │
│   ↓                                                      │
│ import_write_head->safe = false                          │
│ import_write_head->n = undefined                         │
│ import_write_head->end = pos (表达式结束位置)            │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ import('module', { with: { type: 'json' } })             │
│   ↓                                                      │
│ 检测到 ','                                               │
│   ↓                                                      │
│ import_write_head->attr_index = pos (指向 '{')           │
│ import_write_head->end = endPos (字符串结束位置)         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ import.source('module')  (source phase)                  │
│   ↓                                                      │
│ import_write_head->import_ty = DynamicSourcePhase (5)    │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ import.defer('module')  (defer phase)                    │
│   ↓                                                      │
│ import_write_head->import_ty = DynamicDeferPhase (7)     │
└─────────────────────────────────────────────────────────┘
```

## 9. Import Attributes 解析流程

```
import 'module' with { type: "json", integrity: "sha384-..." }
                ↑
            检测到 "with"
                ↓
┌─────────────────────────────────────────────────────────┐
│ 1. 验证语法                                              │
│    pos += 4  (跳过 "with")                               │
│    ch = commentWhitespace(true)                          │
│    if (ch != '{') return  (不是 attributes)              │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 2. 记录起始位置                                          │
│    const attrStart = pos                                 │
│    import_write_head->attr_index = attrStart             │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 3. 循环解析 key-value 对                                 │
│    do {                                                  │
│      pos++                                               │
│      ch = commentWhitespace(true)                        │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 4. 解析 key                                              │
│    if (ch == '"' || ch == "'")                           │
│      stringLiteral(ch)                                   │
│      key = readString(key_start, ch)  (处理转义)         │
│    else                                                  │
│      ch = readToWsOrPunctuator(ch)                       │
│      key = source.slice(key_start, key_end)              │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 5. 验证冒号                                              │
│    if (ch != ':') return  (语法错误)                     │
│    pos++                                                 │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 6. 解析 value (必须是字符串)                             │
│    ch = commentWhitespace(true)                          │
│    if (ch == '"' || ch == "'")                           │
│      stringLiteral(ch)                                   │
│      value = readString(value_start, ch)                 │
│    else                                                  │
│      return  (语法错误)                                  │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 7. 创建 Attribute 结构                                   │
│    Attribute* attr = (Attribute*)(analysis_head)         │
│    analysis_head += sizeof(Attribute)                    │
│    attr->key_start = key_start                           │
│    attr->key_end = key_end                               │
│    attr->value_start = value_start                       │
│    attr->value_end = value_end                           │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 8. 链接到 Import 结构                                    │
│    if (attr_write_head == NULL)                          │
│      import_write_head->attributes = attr                │
│    else                                                  │
│      attr_write_head->next = attr                        │
│    attr_write_head = attr                                │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 9. 检查后续字符                                          │
│    pos++                                                 │
│    ch = commentWhitespace(true)                          │
│    if (ch == ',') continue  (下一个 attribute)           │
│    if (ch == '}') break     (结束)                       │
│    else return              (语法错误)                   │
│    } while (true)                                        │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 10. 完成解析                                             │
│     import_write_head->statement_end = pos + 1           │
└─────────────────────────────────────────────────────────┘

结果数据结构:
┌─────────────────────────────────────────────────────────┐
│ Import {                                                 │
│   attr_index: 指向 '{'                                   │
│   attributes: ───→ Attribute {                           │
│                      key: "type"                         │
│                      value: "json"                       │
│                      next: ───→ Attribute {              │
│                                   key: "integrity"       │
│                                   value: "sha384-..."    │
│                                   next: NULL             │
│                                 }                        │
│                    }                                     │
│ }                                                        │
└─────────────────────────────────────────────────────────┘
```


## 10. 字符串转义处理 (Acorn 移植)

```
readString(start, quote)
   ↓
┌─────────────────────────────────────────────────────────┐
│ 初始化                                                   │
│ acornPos = start                                         │
│ out = ''                                                 │
│ chunkStart = acornPos                                    │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 循环读取字符                                             │
│ for (;;) {                                               │
│   ch = source.charCodeAt(acornPos)                       │
└─────────────────────────────────────────────────────────┘
   ↓
   ch = ?
   ↓
┌────────┬────────┬────────┬────────┐
│ quote  │  '\\'  │ 0x2028 │ 其他    │
│ (结束) │ (转义) │ 0x2029 │        │
└────────┴────────┴────────┴────────┘
   ↓        ↓        ↓        ↓
  break     │        │     acornPos++
            │        │     继续循环
            │        │
            │     acornPos++
            │     (行分隔符)
            │
            ↓
┌─────────────────────────────────────────────────────────┐
│ 处理转义序列                                             │
│ out += source.slice(chunkStart, acornPos)                │
│ out += readEscapedChar()                                 │
│ chunkStart = acornPos                                    │
└─────────────────────────────────────────────────────────┘

readEscapedChar()
   ↓
ch = source.charCodeAt(++acornPos)
++acornPos
   ↓
   ch = ?
   ↓
┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
│ 'n'  │ 'r'  │ 't'  │ 'b'  │ 'v'  │ 'f'  │ 'x'  │ 'u'  │
│ 110  │ 114  │ 116  │ 98   │ 118  │ 102  │ 120  │ 117  │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
   ↓      ↓      ↓      ↓      ↓      ↓      ↓      ↓
  '\n'   '\r'   '\t'   '\b'  '\v'   '\f'    │      │
                                             │      │
                                             ↓      ↓
                                    readHexChar(2) readCodePointToString()
                                             ↓      ↓
                                    String.fromCharCode(hex)
                                                    ↓
                                            处理 Unicode 转义
                                                    ↓
                                            \uXXXX 或 \u{XXXXXX}

┌──────┬──────┬──────┐
│ 13   │ 10   │ 0-7  │
│ '\r' │ '\n' │ 八进制│
└──────┴──────┴──────┘
   ↓      ↓      ↓
  检查   返回   解析八进制
  \r\n   ''     (严格模式错误)
   ↓
  返回 ''

示例转义:
┌─────────────────────────────────────────────────────────┐
│ 输入                │ 输出                                │
├─────────────────────┼─────────────────────────────────────┤
│ "\\n"               │ "\n" (换行符)                       │
│ "\\u0041"           │ "A"                                 │
│ "\\u{1F600}"        │ "😀" (emoji)                        │
│ "\\x41"             │ "A"                                 │
│ "\\\\"              │ "\\" (反斜杠)                       │
│ "\\'"               │ "'" (单引号)                        │
│ "\\\""              │ "\"" (双引号)                       │
│ "\\t"               │ "\t" (制表符)                       │
└─────────────────────────────────────────────────────────┘
```

## 11. WebAssembly 内存管理

```
┌─────────────────────────────────────────────────────────┐
│                  Wasm Memory Layout                      │
├─────────────────────────────────────────────────────────┤
│  __heap_base (固定地址)                                  │
│     ↓                                                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Source Code Buffer                                │  │
│  │ UTF-16 编码                                       │  │
│  │ 大小: (sourceLen + 1) * 2 bytes                   │  │
│  └──────────────────────────────────────────────────┘  │
│     ↓                                                    │
│  analysis_base = source + sourceLen + 1                  │
│     ↓                                                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Analysis Data (动态增长)                          │  │
│  │                                                   │  │
│  │ analysis_head ───→ [下一个可用位置]              │  │
│  │                                                   │  │
│  │ [Import #1] [Import #2] ... [Import #N]          │  │
│  │ [Export #1] [Export #2] ... [Export #N]          │  │
│  │ [Attr #1] [Attr #2] ... [Attr #N]                │  │
│  │                                                   │  │
│  └──────────────────────────────────────────────────┘  │
│     ↓                                                    │
│  [未使用空间]                                            │
│     ↓                                                    │
│  Wasm Memory End                                         │
└─────────────────────────────────────────────────────────┘

内存分配流程:
┌─────────────────────────────────────────────────────────┐
│ 1. TypeScript 层计算所需内存                             │
│    const len = source.length + 1                         │
│    const extraMem = __heap_base + len * 4 - memory.size │
│    (len * 4 = source * 2 + analysis * 2)                │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 2. 扩展 Wasm 内存 (如果需要)                             │
│    if (extraMem > 0)                                     │
│      wasm.memory.grow(Math.ceil(extraMem / 65536))      │
│    (每次增长 64KB 的倍数)                                │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 3. 分配 source buffer                                    │
│    const addr = wasm.sa(len - 1)                         │
│    // C 层:                                              │
│    // analysis_base = source + sourceLen + 1             │
│    // analysis_head = analysis_base                      │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 4. 复制源码到 Wasm 内存                                  │
│    copyLE(source, new Uint16Array(memory, addr, len))   │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 5. 解析过程中动态分配结构                                │
│    Import* import = (Import*)(analysis_head)             │
│    analysis_head += sizeof(Import)                       │
│    // 指针向后移动                                       │
└─────────────────────────────────────────────────────────┘

内存增长策略:
┌─────────────────────────────────────────────────────────┐
│ 初始大小: 64KB (1 page)                                  │
│ 增长单位: 64KB (1 page)                                  │
│ 最大大小: 4GB (理论上限)                                 │
│                                                          │
│ 实际使用:                                                │
│ - 1MB 源码 ≈ 2MB UTF-16 + ~100KB 分析数据 = ~2.1MB      │
│ - 需要 33 pages (2.1MB / 64KB)                           │
└─────────────────────────────────────────────────────────┘
```


## 12. TypeScript API 调用流程

```
用户代码
   ↓
import { parse, init } from 'es-module-lexer'
   ↓
await init  (可选，自动初始化)
   ↓
const [imports, exports, facade, hasModuleSyntax] = parse(source)
   ↓
┌─────────────────────────────────────────────────────────┐
│ parse(source: string, name = '@')                        │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 1. 检查 Wasm 是否已初始化                                │
│    if (!wasm)                                            │
│      return init.then(() => parse(source))               │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 2. 计算并分配内存                                        │
│    const len = source.length + 1                         │
│    const extraMem = wasm.__heap_base + len * 4 - ...     │
│    if (extraMem > 0)                                     │
│      wasm.memory.grow(Math.ceil(extraMem / 65536))      │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 3. 分配 source buffer                                    │
│    const addr = wasm.sa(len - 1)                         │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 4. 复制源码 (UTF-16)                                     │
│    copyLE(source, new Uint16Array(wasm.memory.buffer,   │
│                                    addr, len))           │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 5. 调用 Wasm 解析函数                                    │
│    if (!wasm.parse())                                    │
│      throw new Error(...)                                │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 6. 读取 imports                                          │
│    const imports: ImportSpecifier[] = []                 │
│    while (wasm.ri()) {  // readImport                    │
│      const s = wasm.is(), e = wasm.ie()                  │
│      const t = wasm.it(), a = wasm.ai()                  │
│      const d = wasm.id(), ss = wasm.ss(), se = wasm.se()│
│      let n                                               │
│      if (wasm.ip())  // importSafeString                 │
│        n = decode(source.slice(...))                     │
│      const at: Array<[string, string]> = []              │
│      wasm.rsa()  // resetAttributes                      │
│      while (wasm.ra()) {  // readAttribute               │
│        const aks = wasm.aks(), ake = wasm.ake()          │
│        const avs = wasm.avs(), ave = wasm.ave()          │
│        at.push([decodeIfQuoted(...), decodeIfQuoted(...)]) │
│      }                                                   │
│      imports.push({ n, t, s, e, ss, se, d, a,           │
│                     at: at.length > 0 ? at : null })     │
│    }                                                     │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 7. 读取 exports                                          │
│    const exports: ExportSpecifier[] = []                 │
│    while (wasm.re()) {  // readExport                    │
│      const s = wasm.es(), e = wasm.ee()                  │
│      const ls = wasm.els(), le = wasm.ele()              │
│      const n = decodeIfQuoted(source.slice(s, e))        │
│      const ln = ls < 0 ? undefined :                     │
│                 decodeIfQuoted(source.slice(ls, le))     │
│      exports.push({ s, e, ls, le, n, ln })               │
│    }                                                     │
└─────────────────────────────────────────────────────────┘
   ↓
┌─────────────────────────────────────────────────────────┐
│ 8. 返回结果                                              │
│    return [imports, exports, !!wasm.f(), !!wasm.ms()]    │
│           ↑        ↑         ↑           ↑               │
│           │        │         │           └─ hasModuleSyntax
│           │        │         └─ facade                   │
│           │        └─ exports 数组                       │
│           └─ imports 数组                                │
└─────────────────────────────────────────────────────────┘

Wasm 函数映射:
┌─────────────────────────────────────────────────────────┐
│ TypeScript        │ C 函数              │ 说明          │
├───────────────────┼─────────────────────┼───────────────┤
│ wasm.sa()         │ sa()                │ allocateSource│
│ wasm.parse()      │ parse()             │ 主解析函数    │
│ wasm.ri()         │ ri()                │ readImport    │
│ wasm.re()         │ re()                │ readExport    │
│ wasm.is()         │ is()                │ importStart   │
│ wasm.ie()         │ ie()                │ importEnd     │
│ wasm.it()         │ it()                │ importType    │
│ wasm.ai()         │ ai()                │ attrIndex     │
│ wasm.id()         │ id()                │ importDynamic │
│ wasm.ip()         │ ip()                │ importSafe    │
│ wasm.ss()         │ ss()                │ statementStart│
│ wasm.se()         │ se()                │ statementEnd  │
│ wasm.es()         │ es()                │ exportStart   │
│ wasm.ee()         │ ee()                │ exportEnd     │
│ wasm.els()        │ els()               │ exportLocalS  │
│ wasm.ele()        │ ele()               │ exportLocalE  │
│ wasm.f()          │ f()                 │ facade        │
│ wasm.ms()         │ ms()                │ hasModuleSyntax│
│ wasm.ra()         │ ra()                │ readAttribute │
│ wasm.rsa()        │ rsa()               │ resetAttributes│
│ wasm.aks()        │ aks()               │ attrKeyStart  │
│ wasm.ake()        │ ake()               │ attrKeyEnd    │
│ wasm.avs()        │ avs()               │ attrValueStart│
│ wasm.ave()        │ ave()               │ attrValueEnd  │
└─────────────────────────────────────────────────────────┘
```

## 13. 性能优化技术总结

```
┌─────────────────────────────────────────────────────────┐
│                   性能优化技术                           │
├─────────────────────────────────────────────────────────┤
│ 1. 编译优化                                              │
│    ├─ C → WebAssembly (WASI SDK -O3)                    │
│    ├─ C → asm.js (Emscripten -O3)                       │
│    └─ 编译器优化: 内联、循环展开、SIMD                   │
├─────────────────────────────────────────────────────────┤
│ 2. 算法优化                                              │
│    ├─ 单次遍历 (O(n) 时间复杂度)                        │
│    ├─ 两阶段解析 (Facade 模式快速路径)                   │
│    ├─ 回溯分析 (正则/除法歧义)                          │
│    └─ 状态机驱动 (switch-case)                          │
├─────────────────────────────────────────────────────────┤
│ 3. 内存优化                                              │
│    ├─ 栈分配 (避免堆分配)                               │
│    ├─ 链表结构 (动态增长)                               │
│    ├─ 指针操作 (零拷贝)                                 │
│    └─ UTF-16 原生编码 (避免转换)                        │
├─────────────────────────────────────────────────────────┤
│ 4. 字符串优化                                            │
│    ├─ memcmp 批量比较                                    │
│    ├─ 预定义常量字符串                                   │
│    ├─ 字符码比较 (charCodeAt)                           │
│    └─ 延迟字符串解码                                     │
├─────────────────────────────────────────────────────────┤
│ 5. 数据结构优化                                          │
│    ├─ 固定大小栈 (1024 元素)                            │
│    ├─ 单向链表 (简单高效)                               │
│    ├─ 位置指针 (char16_t*)                              │
│    └─ 枚举类型 (紧凑表示)                               │
├─────────────────────────────────────────────────────────┤
│ 6. 分支预测优化                                          │
│    ├─ 热路径优先 (import/export 在前)                   │
│    ├─ 早期返回 (快速失败)                               │
│    └─ switch-case 优化 (编译器优化)                     │
└─────────────────────────────────────────────────────────┘

性能对比:
┌─────────────────────────────────────────────────────────┐
│ 引擎          │ 加载时间 │ 冷启动  │ 热启动  │ 相对速度 │
├───────────────┼──────────┼─────────┼─────────┼──────────┤
│ Wasm          │ 5ms      │ 18ms    │ 14ms    │ 1.0x     │
│ asm.js        │ 2ms      │ 34ms    │ 17ms    │ 0.82x    │
│ Pure JS       │ 0ms      │ ~50ms   │ ~30ms   │ 0.47x    │
│ Acorn (参考)  │ ~10ms    │ ~300ms  │ ~150ms  │ 0.09x    │
└─────────────────────────────────────────────────────────┘

吞吐量对比 (3123 KiB 测试集):
┌─────────────────────────────────────────────────────────┐
│ Wasm:    ~223 MB/s (14ms / 3.1MB)                        │
│ asm.js:  ~182 MB/s (17ms / 3.1MB)                        │
│ Acorn:   ~21 MB/s  (150ms / 3.1MB)                       │
└─────────────────────────────────────────────────────────┘
```

## 14. 测试架构图

```
┌─────────────────────────────────────────────────────────┐
│                      测试体系                            │
├─────────────────────────────────────────────────────────┤
│ 单元测试 (test/_unit.cjs)                                │
│  ├─ Import 测试 (200+ 测试用例)                          │
│  │  ├─ 静态 import                                       │
│  │  ├─ 动态 import                                       │
│  │  ├─ import.meta                                       │
│  │  ├─ import attributes                                 │
│  │  ├─ source phase imports                              │
│  │  └─ defer phase imports                               │
│  ├─ Export 测试 (150+ 测试用例)                          │
│  │  ├─ 命名导出                                          │
│  │  ├─ 默认导出                                          │
│  │  ├─ 重导出                                            │
│  │  └─ 解构导出                                          │
│  ├─ 边缘情况测试 (100+ 测试用例)                         │
│  │  ├─ 正则/除法歧义                                     │
│  │  ├─ 模板字符串                                        │
│  │  ├─ 注释处理                                          │
│  │  ├─ Unicode 转义                                      │
│  │  └─ 嵌套结构                                          │
│  └─ 错误处理测试                                         │
│     ├─ 语法错误                                          │
│     └─ 边界条件                                          │
├─────────────────────────────────────────────────────────┤
│ 集成测试 (test/integration.cjs)                          │
│  └─ 真实库代码测试                                       │
│     ├─ angular.js (739 KB)                               │
│     ├─ d3.js (508 KB)                                    │
│     ├─ rollup.js (929 KB)                                │
│     └─ magic-string.js (35 KB)                           │
├─────────────────────────────────────────────────────────┤
│ 性能测试 (bench/index.js)                                │
│  ├─ 冷启动测试                                           │
│  ├─ 热启动测试 (25 次平均)                               │
│  ├─ Wasm vs asm.js 对比                                  │
│  └─ 内存使用分析                                         │
└─────────────────────────────────────────────────────────┘

测试执行流程:
┌─────────────────────────────────────────────────────────┐
│ npm test                                                 │
│   ↓                                                      │
│ chomp test                                               │
│   ↓                                                      │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 1. 构建 Wasm/asm.js                                  │ │
│ │    chomp build                                       │ │
│ └─────────────────────────────────────────────────────┘ │
│   ↓                                                      │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 2. 运行单元测试                                      │ │
│ │    WASM=1 mocha test/_unit.cjs                       │ │
│ │    ASM=1 mocha test/_unit.cjs                        │ │
│ │    mocha test/_unit.cjs (Pure JS)                    │ │
│ └─────────────────────────────────────────────────────┘ │
│   ↓                                                      │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 3. 运行集成测试                                      │ │
│ │    node test/integration.cjs                         │ │
│ └─────────────────────────────────────────────────────┘ │
│   ↓                                                      │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ 4. 运行性能测试 (可选)                               │ │
│ │    node bench/index.js                               │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

**文档版本**: 1.0  
**最后更新**: 2026-01-27  
**作者**: Kiro AI Assistant

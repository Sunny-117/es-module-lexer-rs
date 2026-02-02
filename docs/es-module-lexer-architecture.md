# ES Module Lexer 项目架构与原理分析

## 1. 项目概述

### 1.1 核心功能
ES Module Lexer 是一个高性能的 JavaScript 模块语法词法分析器，专门用于：
- 快速提取 ES 模块的 import/export 语句
- 解析动态 import 表达式
- 处理 import.meta 语法
- 支持 import attributes（with 语法）
- 支持 source phase 和 defer phase imports

### 1.2 性能特点
- **极致性能**：Angular 1 (720KB) 仅需 5ms 解析，而 Acorn 需要 100ms+
- **体积小巧**：仅 4KB gzipped
- **双引擎**：提供 WebAssembly 和 asm.js 两种实现

### 1.3 应用场景
主要用于 [es-module-shims](https://github.com/guybedford/es-module-shims) 项目，用于在浏览器中实现 ES 模块加载。

---

## 2. 整体架构设计

### 2.1 三层架构

```
┌─────────────────────────────────────────────────────────┐
│                    API 层 (TypeScript)                   │
│  - parse() 函数：主入口                                   │
│  - init/initSync：初始化 WebAssembly                     │
│  - 类型定义：ImportSpecifier, ExportSpecifier           │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   编译层 (Build System)                  │
│  - C → WebAssembly (WASI SDK)                           │
│  - C → asm.js (Emscripten)                              │
│  - TypeScript → JavaScript                               │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   核心引擎层 (C/JavaScript)               │
│  - lexer.c：C 语言实现（编译为 Wasm/asm.js）             │
│  - lexer.js：纯 JavaScript 实现（备用）                  │
└─────────────────────────────────────────────────────────┘
```

### 2.2 文件结构

```
es-module-lexer/
├── src/
│   ├── lexer.ts          # TypeScript API 和类型定义
│   ├── lexer.h           # C 语言头文件（数据结构定义）
│   ├── lexer.c           # C 语言核心实现
│   └── lexer.asm.js      # asm.js 包装器
├── lib/
│   ├── lexer.wasm        # 编译后的 WebAssembly 二进制
│   ├── lexer.emcc.asm.js # Emscripten 编译的 asm.js
│   └── lexer.asm.js      # 优化后的 asm.js
├── lexer.js              # 纯 JS 实现（无 Wasm）
├── test/
│   ├── _unit.cjs         # 单元测试
│   └── samples/          # 测试样本（真实库代码）
└── bench/
    └── index.js          # 性能基准测试
```

---

## 3. 核心数据结构

### 3.1 Import 数据结构

```c
struct Import {
  const char16_t* start;              // 模块说明符开始位置
  const char16_t* end;                // 模块说明符结束位置
  const char16_t* statement_start;    // import 语句开始位置
  const char16_t* statement_end;      // import 语句结束位置
  const char16_t* attr_index;         // import attributes 开始位置
  const char16_t* dynamic;            // 动态 import 标记
  bool safe;                          // 是否为安全的字符串字面量
  enum ImportType import_ty;          // import 类型
  struct Attribute* attributes;       // attributes 链表
  struct Import* next;                // 下一个 import（链表）
};
```

**ImportType 枚举**：
```typescript
enum ImportType {
  Static = 1,              // import x from 'mod'
  Dynamic = 2,             // import('mod')
  ImportMeta = 3,          // import.meta
  StaticSourcePhase = 4,   // import source x from 'mod'
  DynamicSourcePhase = 5,  // import.source('mod')
  StaticDeferPhase = 6,    // import defer * as x from 'mod'
  DynamicDeferPhase = 7,   // import.defer('mod')
}
```

### 3.2 Export 数据结构

```c
struct Export {
  const char16_t* start;        // 导出名称开始位置
  const char16_t* end;          // 导出名称结束位置
  const char16_t* local_start;  // 本地名称开始位置
  const char16_t* local_end;    // 本地名称结束位置
  struct Export* next;          // 下一个 export（链表）
};
```

### 3.3 Attribute 数据结构

```c
struct Attribute {
  const char16_t* key_start;    // 属性键开始位置
  const char16_t* key_end;      // 属性键结束位置
  const char16_t* value_start;  // 属性值开始位置
  const char16_t* value_end;    // 属性值结束位置
  struct Attribute* next;       // 下一个属性（链表）
};
```

### 3.4 OpenToken 栈结构

```c
enum OpenTokenState {
  AnyParen = 1,        // (
  AnyBrace = 2,        // {
  Template = 3,        // `
  TemplateBrace = 4,   // ${
  ImportParen = 5,     // import()
  ClassBrace = 6,      // class {}
  AsyncParen = 7,      // async()
};

struct OpenToken {
  enum OpenTokenState token;  // token 类型
  char16_t* pos;              // token 位置
};
```

---

## 4. 核心算法设计

### 4.1 两阶段解析策略

```
┌─────────────────────────────────────────────────────┐
│              Phase 1: Facade 模式解析                │
│  - 只解析 import/export 语句                         │
│  - 遇到非模块语法立即切换到 Phase 2                  │
│  - 目标：快速处理纯模块文件                          │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│              Phase 2: 完整语法解析                   │
│  - 处理所有 JavaScript 语法                          │
│  - 跟踪括号/大括号匹配                               │
│  - 处理正则表达式/除法运算符歧义                     │
│  - 处理模板字符串                                    │
└─────────────────────────────────────────────────────┘
```

**代码实现**：
```c
bool parse() {
  facade = true;
  
  // Phase 1: 纯模块解析
  while (pos++ < end) {
    ch = *pos;
    switch (ch) {
      case 'e':
        if (keywordStart(pos) && memcmp(pos + 1, &XPORT[0], 5 * 2) == 0) {
          tryParseExportStatement();
          if (!facade) {
            goto mainparse;  // 切换到 Phase 2
          }
        }
        break;
      case 'i':
        if (keywordStart(pos) && memcmp(pos + 1, &MPORT[0], 5 * 2) == 0)
          tryParseImportStatement();
        break;
      default:
        facade = false;
        goto mainparse;  // 切换到 Phase 2
    }
  }
  
  // Phase 2: 完整解析
  mainparse: while (pos++ < end) {
    // 处理所有语法...
  }
}
```

### 4.2 正则表达式 vs 除法运算符歧义处理

这是词法分析中的经典难题。ES Module Lexer 使用**回溯分析**策略：

```javascript
// 歧义示例
x / y / z    // 除法
/regex/      // 正则表达式
```

**判断策略**：
1. 检查前一个 token 类型
2. 如果是表达式标点符号 → 正则表达式
3. 如果是 `)` 且前面是 `while/for/if` → 正则表达式
4. 如果是 `}` 且是表达式终结符 → 正则表达式

```c
char16_t lastToken = *lastTokenPos;
if (isExpressionPunctuator(lastToken) ||
    lastToken == ')' && isParenKeyword(openTokenStack[openTokenDepth].pos) ||
    lastToken == '}' && isExpressionTerminator(openTokenStack[openTokenDepth].pos) ||
    isExpressionKeyword(lastTokenPos)) {
  regularExpression();  // 解析为正则表达式
} else {
  lastSlashWasDivision = true;  // 解析为除法
}
```

### 4.3 括号/大括号匹配算法

使用**栈**跟踪嵌套结构：

```c
OpenToken openTokenStack[1024];  // 固定大小栈
uint16_t openTokenDepth = 0;     // 栈深度

// 遇到开括号
case '(':
  openTokenStack[openTokenDepth].token = AnyParen;
  openTokenStack[openTokenDepth++].pos = lastTokenPos;
  break;

// 遇到闭括号
case ')':
  openTokenDepth--;
  if (dynamicImportStackDepth > 0 && 
      openTokenStack[openTokenDepth].token == ImportParen) {
    // 完成动态 import 解析
    cur_dynamic_import->statement_end = pos + 1;
    dynamicImportStackDepth--;
  }
  break;
```

### 4.4 动态 Import 解析

```c
void tryParseImportStatement() {
  // 检测 import(
  if (ch == '(') {
    openTokenStack[openTokenDepth++].pos = pos;
    addImport(startPos, pos, 0, dynamicPos);
    dynamicImportStack[dynamicImportStackDepth++] = import_write_head;
    
    // 尝试解析字符串字面量
    if (ch == '\'' || ch == '"') {
      stringLiteral(ch);
      import_write_head->safe = true;  // 标记为安全字符串
    }
    
    // 检测 import attributes
    if (ch == ',') {
      import_write_head->attr_index = pos;
    }
  }
}
```

### 4.5 Import Attributes 解析

```c
void readImportString(const char16_t* ss, char16_t ch, int phase_keyword) {
  // 解析 with { key: "value" }
  if (ch == 'w' && memcmp(pos + 1, "ith", 3 * 2) == 0) {
    pos += 4;
    ch = commentWhitespace(true);
    if (ch != '{') return;
    
    do {
      // 解析 key
      if (ch == '\'' || ch == '"') {
        stringLiteral(ch);
        key = readString(key_start, ch);
      }
      
      // 解析 value
      if (ch == '\'' || ch == '"') {
        stringLiteral(ch);
        value = readString(value_start, ch);
      }
      
      // 添加到 attributes 链表
      Attribute* attr = (Attribute*)(analysis_head);
      attr->key_start = key_start;
      attr->value_start = value_start;
      import_write_head->attributes = attr;
    } while (ch == ',');
  }
}
```

---

## 5. 内存管理策略

### 5.1 内存布局

```
┌──────────────────────────────────────────────────────┐
│                   WebAssembly Memory                  │
├──────────────────────────────────────────────────────┤
│  Source Code (UTF-16)                                 │
│  ↓                                                    │
│  [char16_t array]                                     │
├──────────────────────────────────────────────────────┤
│  Analysis Data (动态分配)                             │
│  ↓                                                    │
│  [Import structs] → [Export structs] → [Attributes]  │
└──────────────────────────────────────────────────────┘
```

### 5.2 动态内存分配

```c
void* analysis_base;   // 分析数据起始位置
void* analysis_head;   // 当前分配位置

// 分配 Import 结构
void addImport(...) {
  Import* import = (Import*)(analysis_head);
  analysis_head = analysis_head + sizeof(Import);  // 移动指针
  // ... 初始化 import
}

// 分配 Export 结构
void addExport(...) {
  Export* export = (Export*)(analysis_head);
  analysis_head = analysis_head + sizeof(Export);
  // ... 初始化 export
}
```

### 5.3 链表管理

所有数据结构使用**单向链表**连接：

```c
Import* first_import = NULL;
Import* import_write_head = NULL;  // 写入头
Import* import_read_head = NULL;   // 读取头

// 添加新 import
if (import_write_head == NULL)
  first_import = import;
else
  import_write_head->next = import;
import_write_head = import;
```

---

## 6. WebAssembly 集成

### 6.1 TypeScript 包装层

```typescript
export function parse(source: string, name = '@'): readonly [
  imports: ReadonlyArray<ImportSpecifier>,
  exports: ReadonlyArray<ExportSpecifier>,
  facade: boolean,
  hasModuleSyntax: boolean
] {
  // 1. 检查 Wasm 是否已初始化
  if (!wasm)
    return init.then(() => parse(source));

  // 2. 分配内存
  const len = source.length + 1;
  const extraMem = wasm.__heap_base + len * 4 - wasm.memory.buffer.byteLength;
  if (extraMem > 0)
    wasm.memory.grow(Math.ceil(extraMem / 65536));

  // 3. 复制源码到 Wasm 内存（UTF-16）
  const addr = wasm.sa(len - 1);
  copyLE(source, new Uint16Array(wasm.memory.buffer, addr, len));

  // 4. 调用 Wasm 解析函数
  if (!wasm.parse())
    throw new Error(`Parse error ${name}:...`);

  // 5. 读取解析结果
  const imports: ImportSpecifier[] = [];
  while (wasm.ri()) {  // readImport
    imports.push({
      n: wasm.ip() ? decode(source.slice(...)) : undefined,
      t: wasm.it(),  // importType
      s: wasm.is(),  // importStart
      e: wasm.ie(),  // importEnd
      // ...
    });
  }

  const exports: ExportSpecifier[] = [];
  while (wasm.re()) {  // readExport
    exports.push({
      s: wasm.es(),  // exportStart
      e: wasm.ee(),  // exportEnd
      // ...
    });
  }

  return [imports, exports, !!wasm.f(), !!wasm.ms()];
}
```

### 6.2 Wasm 函数导出

```c
// C 函数 → Wasm 导出
const char16_t* sa(uint32_t utf16Len);  // allocateSource
bool parse();                            // parse
bool ri();                               // readImport
uint32_t is();                           // getImportStart
uint32_t ie();                           // getImportEnd
// ... 更多 getter 函数
```

---

## 7. 性能优化技术

### 7.1 字符串比较优化

使用 `memcmp` 进行批量字符比较：

```c
// 检测 "export"
if (memcmp(pos + 1, &XPORT[0], 5 * 2) == 0)

// 预定义常量字符串
static const char16_t XPORT[] = { 'x', 'p', 'o', 'r', 't' };
static const char16_t MPORT[] = { 'm', 'p', 'o', 'r', 't' };
```

### 7.2 栈分配 vs 堆分配

```c
bool parse() {
  // 栈分配（避免堆分配开销）
  OpenToken openTokenStack_[1024];
  Import* dynamicImportStack_[512];
  
  openTokenStack = &openTokenStack_[0];
  dynamicImportStack = &dynamicImportStack_[0];
}
```

### 7.3 单次遍历

整个解析过程只遍历源码**一次**，不回退（除了正则/除法歧义处理）。

### 7.4 UTF-16 编码

直接使用 UTF-16 编码（JavaScript 原生编码），避免转换开销：

```typescript
function copyLE(src: string, outBuf16: Uint16Array) {
  const len = src.length;
  let i = 0;
  while (i < len)
    outBuf16[i] = src.charCodeAt(i++);  // 直接复制
}
```

---

## 8. 测试架构

### 8.1 测试类型

```
test/
├── _unit.cjs              # 单元测试（1680+ 行）
│   ├── Import 测试
│   │   ├── 静态 import
│   │   ├── 动态 import
│   │   ├── import.meta
│   │   ├── import attributes
│   │   ├── source phase imports
│   │   └── defer phase imports
│   ├── Export 测试
│   │   ├── 命名导出
│   │   ├── 默认导出
│   │   ├── 重导出
│   │   └── 解构导出
│   ├── 边缘情况测试
│   │   ├── 正则/除法歧义
│   │   ├── 模板字符串
│   │   ├── 注释处理
│   │   └── Unicode 转义
│   └── 错误处理测试
└── samples/               # 真实库代码测试
    ├── angular.js         # 739 KB
    ├── d3.js              # 508 KB
    ├── rollup.js          # 929 KB
    └── ...
```

### 8.2 测试策略

```javascript
suite('Lexer', () => {
  beforeEach(async () => await init);  // 初始化 Wasm

  test('Import attributes parsing', () => {
    const source = `
      import foo from 'module' with { type: "json" }
    `;
    const [impts] = parse(source);
    assert.deepStrictEqual(impts[0].at, [['type', 'json']]);
  });

  test('Dynamic import expression range', () => {
    const source = `import(("asdf"))`;
    const [[impt]] = parse(source);
    assert.strictEqual(source.slice(impt.ss, impt.se), 'import(("asdf"))');
  });
});
```

---

## 9. Benchmark 架构

### 9.1 性能测试设计

```javascript
// bench/index.js
const files = [
  'test/samples/angular.js',     // 739 KB
  'test/samples/d3.js',           // 508 KB
  'test/samples/rollup.js',       // 929 KB
  // ...
];

// 冷启动测试
console.log('Cold Run, All Samples');
files.forEach(({ code }) => {
  const start = process.hrtime.bigint();
  parse(code);
  const end = process.hrtime.bigint();
  console.log(`> ${Math.round(Number(end - start) / 1e6)}ms`);
});

// 热启动测试（25 次平均）
console.log('Warm Runs (average of 25 runs)');
for (let i = 0; i < 25; i++) {
  files.forEach(({ code }) => {
    total += timeRun(code);
    gc();  // 手动触发 GC
  });
}
```

### 9.2 性能指标

```
--- Wasm Build ---
Module load time: 5ms
Cold Run: 18ms (3123 KiB)
Warm Run: 14.16ms (3123 KiB)

--- JS Build (asm.js) ---
Module load time: 2ms
Cold Run: 34ms (3123 KiB)
Warm Run: 17.12ms (3123 KiB)
```

**性能对比**：
- Wasm: ~5ms/MB (warm)
- asm.js: ~5.5ms/MB (warm)
- Acorn: ~100ms/MB

---

## 10. 构建系统

### 10.1 构建流程

```
┌─────────────────────────────────────────────────────┐
│                   Source Files                       │
│  lexer.c + lexer.h                                   │
└─────────────────────────────────────────────────────┘
              ↓                    ↓
    ┌─────────────────┐  ┌─────────────────┐
    │   WASI SDK      │  │   Emscripten    │
    │   (clang)       │  │   (emcc)        │
    └─────────────────┘  └─────────────────┘
              ↓                    ↓
    ┌─────────────────┐  ┌─────────────────┐
    │  lexer.wasm     │  │ lexer.asm.js    │
    └─────────────────┘  └─────────────────┘
              ↓                    ↓
    ┌─────────────────────────────────────┐
    │      Base64 Encoding                │
    │  嵌入到 lexer.ts 中                  │
    └─────────────────────────────────────┘
              ↓
    ┌─────────────────────────────────────┐
    │      TypeScript Compilation         │
    │  lexer.ts → dist/lexer.js           │
    └─────────────────────────────────────┘
```

### 10.2 Chomp 构建配置

```toml
# chompfile.toml
[[task]]
name = 'build:wasm'
run = '''
  $WASI_PATH/bin/clang \
    --target=wasm32-wasi \
    -O3 \
    -o lib/lexer.wasm \
    src/lexer.c
'''

[[task]]
name = 'build:asm'
run = '''
  $EMSDK_PATH/emcc \
    -O3 \
    -s WASM=0 \
    -o lib/lexer.asm.js \
    src/lexer.c
'''
```

---

## 11. 关键设计决策

### 11.1 为什么使用 C + WebAssembly？

1. **性能**：C 编译为 Wasm 比纯 JS 快 5-10 倍
2. **内存控制**：精确控制内存布局和分配
3. **可移植性**：同一份 C 代码编译为 Wasm 和 asm.js

### 11.2 为什么不使用完整的 Parser？

1. **性能**：完整 AST 解析太慢
2. **体积**：完整 parser 体积大（Acorn ~100KB）
3. **需求**：只需要 import/export 信息，不需要完整 AST

### 11.3 为什么使用链表而不是数组？

1. **内存效率**：动态分配，不需要预分配大数组
2. **简单性**：C 语言中链表实现简单
3. **性能**：顺序遍历性能相当

### 11.4 为什么提供 asm.js 版本？

1. **CSP 兼容**：某些环境禁用 Wasm
2. **兼容性**：旧浏览器不支持 Wasm
3. **性能**：asm.js 性能接近 Wasm（仅慢 20%）

---

## 12. 架构图总结

### 12.1 数据流图

```
┌──────────────┐
│ Source Code  │
│  (String)    │
└──────────────┘
       ↓
┌──────────────────────────────────────┐
│  TypeScript Wrapper (lexer.ts)       │
│  - 内存分配                           │
│  - UTF-16 编码转换                    │
│  - Wasm 函数调用                      │
└──────────────────────────────────────┘
       ↓
┌──────────────────────────────────────┐
│  WebAssembly Engine (lexer.wasm)     │
│  - 词法分析                           │
│  - 语法识别                           │
│  - 数据结构构建                       │
└──────────────────────────────────────┘
       ↓
┌──────────────────────────────────────┐
│  Result Extraction                    │
│  - 读取 Import 链表                   │
│  - 读取 Export 链表                   │
│  - 解码字符串                         │
└──────────────────────────────────────┘
       ↓
┌──────────────────────────────────────┐
│  Return Value                         │
│  [imports, exports, facade, hasESM]  │
└──────────────────────────────────────┘
```

### 12.2 模块依赖图

```
┌─────────────────────────────────────────────────┐
│                  es-module-lexer                 │
├─────────────────────────────────────────────────┤
│  dist/lexer.js (Wasm)                            │
│    ↓                                             │
│  lib/lexer.wasm                                  │
│    ↓                                             │
│  src/lexer.c + src/lexer.h                       │
├─────────────────────────────────────────────────┤
│  dist/lexer.asm.js (asm.js)                      │
│    ↓                                             │
│  lib/lexer.emcc.asm.js                           │
│    ↓                                             │
│  src/lexer.c + src/lexer.h                       │
├─────────────────────────────────────────────────┤
│  lexer.js (Pure JS - Fallback)                   │
│    ↓                                             │
│  纯 JavaScript 实现（无依赖）                     │
└─────────────────────────────────────────────────┘
```

---

## 13. 总结

### 13.1 核心优势

1. **极致性能**：通过 C + Wasm 实现，比纯 JS parser 快 10-20 倍
2. **体积小巧**：仅 4KB gzipped，适合浏览器环境
3. **精确解析**：专注于 import/export，不做无用功
4. **多引擎支持**：Wasm、asm.js、纯 JS 三种实现

### 13.2 技术亮点

1. **两阶段解析**：Facade 模式快速处理纯模块文件
2. **回溯分析**：巧妙处理正则/除法歧义
3. **内存优化**：链表 + 栈分配，零堆分配
4. **单次遍历**：O(n) 时间复杂度

### 13.3 适用场景

- ES 模块加载器（如 es-module-shims）
- 构建工具（快速依赖分析）
- 开发工具（import/export 提取）
- 任何需要快速解析 ES 模块的场景

---

**文档版本**: 1.0  
**最后更新**: 2026-01-27  
**作者**: Kiro AI Assistant

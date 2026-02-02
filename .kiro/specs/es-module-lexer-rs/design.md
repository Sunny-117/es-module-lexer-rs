# 设计文档：es-module-lexer-rs

## 概述

es-module-lexer-rs 是 es-module-lexer 的 Rust 实现，通过 napi-rs 提供 Node.js API。本设计遵循源码对齐原则，忠实复刻原始实现的行为，同时利用 Rust 的内存安全性和性能优势。

### 设计目标

1. **功能对齐**：与 es-module-lexer 行为完全一致
2. **性能优势**：在中大型文件上比 WebAssembly 版本快 20%+
3. **内存安全**：利用 Rust 的所有权系统避免内存错误
4. **API 兼容**：提供与原始实现完全兼容的 JavaScript API
5. **可维护性**：清晰的代码结构和文档

### 技术栈

- **核心语言**：Rust (edition 2021)
- **Node.js 绑定**：napi-rs
- **构建工具**：Cargo, pnpm
- **测试框架**：Rust (cargo test), JavaScript (vitest)
- **TypeScript 构建**：tsdown
- **TS代码规范**：oxlint+oxfmt

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    JavaScript 层                         │
│  - parse() API                                           │
│  - TypeScript 类型定义                                   │
└─────────────────────────────────────────────────────────┘
                        ↓ (napi-rs)
┌─────────────────────────────────────────────────────────┐
│                    Napi 绑定层                           │
│  - 字符串编码转换 (UTF-16 ↔ Rust String)                │
│  - 数据结构转换 (Rust → JS Object)                      │
│  - 错误处理和传播                                        │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│                    Rust 核心层                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Lexer (主解析器)                                   │  │
│  │  - parse() 主函数                                  │  │
│  │  - Phase 1: Facade 模式                            │  │
│  │  - Phase 2: 完整解析                               │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Parser 模块                                        │  │
│  │  - parse_import_statement()                        │  │
│  │  - parse_export_statement()                        │  │
│  │  - parse_import_attributes()                       │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Scanner 模块                                       │  │
│  │  - string_literal()                                │  │
│  │  - regular_expression()                            │  │
│  │  - comment_whitespace()                            │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ 数据结构                                           │  │
│  │  - Import, Export, Attribute                       │  │
│  │  - OpenToken, ImportType                           │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```


### 模块组织

```
es-module-lexer-rs/
├── crates/
│   └── es-module-lexer/
│       ├── src/
│       │   ├── lib.rs              # 库入口
│       │   ├── lexer.rs            # 主解析器
│       │   ├── parser/
│       │   │   ├── mod.rs          # Parser 模块
│       │   │   ├── import.rs       # Import 解析
│       │   │   ├── export.rs       # Export 解析
│       │   │   └── attributes.rs   # Attributes 解析
│       │   ├── scanner/
│       │   │   ├── mod.rs          # Scanner 模块
│       │   │   ├── string.rs       # 字符串处理
│       │   │   ├── regex.rs        # 正则表达式处理
│       │   │   └── comment.rs      # 注释处理
│       │   ├── types.rs            # 数据结构定义
│       │   └── error.rs            # 错误类型
│       └── Cargo.toml
├── packages/
│   └── es-module-lexer-rs/
│       ├── src/
│       │   ├── index.ts            # JavaScript API
│       │   ├── binding.ts          # Napi 绑定
│       │   └── types.ts            # TypeScript 类型
│       ├── native/
│       │   ├── src/
│       │   │   └── lib.rs          # Napi-rs 绑定实现
│       │   └── Cargo.toml
│       ├── tests/
│       │   ├── unit.test.ts        # 单元测试
│       │   └── integration.test.ts # 集成测试
│       └── package.json
├── pnpm-workspace.yaml
└── README.md
```

## 组件和接口

### 核心数据结构

#### Import 结构

```rust
#[derive(Debug, Clone)]
pub struct Import {
    /// 模块说明符开始位置（字节索引）
    pub start: usize,
    /// 模块说明符结束位置（字节索引）
    pub end: usize,
    /// import 语句开始位置
    pub statement_start: usize,
    /// import 语句结束位置
    pub statement_end: usize,
    /// import attributes 开始位置（如果存在）
    pub attr_index: Option<usize>,
    /// 动态 import 标记：None=静态, Some(pos)=动态
    pub dynamic: Option<usize>,
    /// 是否为安全的字符串字面量
    pub safe: bool,
    /// Import 类型
    pub import_type: ImportType,
    /// Import attributes
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportType {
    Static = 1,
    Dynamic = 2,
    ImportMeta = 3,
    StaticSourcePhase = 4,
    DynamicSourcePhase = 5,
    StaticDeferPhase = 6,
    DynamicDeferPhase = 7,
}
```


#### Export 结构

```rust
#[derive(Debug, Clone)]
pub struct Export {
    /// 导出名称开始位置
    pub start: usize,
    /// 导出名称结束位置
    pub end: usize,
    /// 本地名称开始位置（如果不同于导出名称）
    pub local_start: Option<usize>,
    /// 本地名称结束位置
    pub local_end: Option<usize>,
}
```

#### Attribute 结构

```rust
#[derive(Debug, Clone)]
pub struct Attribute {
    /// 属性键开始位置
    pub key_start: usize,
    /// 属性键结束位置
    pub key_end: usize,
    /// 属性值开始位置
    pub value_start: usize,
    /// 属性值结束位置
    pub value_end: usize,
}
```

#### OpenToken 栈

```rust
#[derive(Debug, Clone, Copy)]
pub enum OpenTokenState {
    AnyParen,
    AnyBrace,
    Template,
    TemplateBrace,
    ImportParen,
    ClassBrace,
    AsyncParen,
}

#[derive(Debug, Clone, Copy)]
pub struct OpenToken {
    pub state: OpenTokenState,
    pub pos: usize,
}
```

#### 解析结果

```rust
#[derive(Debug)]
pub struct ParseResult {
    /// 所有 import 语句
    pub imports: Vec<Import>,
    /// 所有 export 语句
    pub exports: Vec<Export>,
    /// 是否为 facade 模式（纯模块文件）
    pub facade: bool,
    /// 是否包含模块语法
    pub has_module_syntax: bool,
}
```

### Lexer 主接口

```rust
pub struct Lexer<'a> {
    /// 源代码（UTF-8 字节）
    source: &'a [u8],
    /// 当前位置
    pos: usize,
    /// 源代码结束位置
    end: usize,
    /// 是否为 facade 模式
    facade: bool,
    /// 括号匹配栈
    open_token_stack: Vec<OpenToken>,
    /// 动态 import 栈
    dynamic_import_stack: Vec<usize>,
    /// 解析结果
    imports: Vec<Import>,
    exports: Vec<Export>,
    /// 最后一个 token 位置
    last_token_pos: usize,
    /// 最后一个 slash 是否为除法
    last_slash_was_division: bool,
}

impl<'a> Lexer<'a> {
    /// 创建新的 lexer
    pub fn new(source: &'a str) -> Self;
    
    /// 执行解析
    pub fn parse(&mut self) -> Result<ParseResult, LexerError>;
    
    /// Phase 1: Facade 模式解析
    fn parse_facade(&mut self) -> Result<bool, LexerError>;
    
    /// Phase 2: 完整解析
    fn parse_full(&mut self) -> Result<(), LexerError>;
}
```


### Parser 模块接口

```rust
impl<'a> Lexer<'a> {
    /// 尝试解析 import 语句
    fn try_parse_import_statement(&mut self) -> Result<(), LexerError>;
    
    /// 尝试解析 export 语句
    fn try_parse_export_statement(&mut self) -> Result<(), LexerError>;
    
    /// 解析 import attributes (with 子句)
    fn parse_import_attributes(&mut self, import_idx: usize) -> Result<(), LexerError>;
    
    /// 读取 import 字符串并检测 attributes
    fn read_import_string(&mut self, quote: u8, phase_keyword: Option<&str>) -> Result<(), LexerError>;
    
    /// 解析 export list { a, b as c }
    fn parse_export_list(&mut self) -> Result<(), LexerError>;
}
```

### Scanner 模块接口

```rust
impl<'a> Lexer<'a> {
    /// 扫描字符串字面量
    fn string_literal(&mut self, quote: u8) -> Result<(), LexerError>;
    
    /// 扫描正则表达式
    fn regular_expression(&mut self) -> Result<(), LexerError>;
    
    /// 跳过注释和空白
    fn comment_whitespace(&mut self, allow_regex: bool) -> Result<u8, LexerError>;
    
    /// 读取转义字符
    fn read_escaped_char(&mut self) -> Result<char, LexerError>;
    
    /// 检查是否为关键字开始
    fn is_keyword_start(&self, pos: usize) -> bool;
    
    /// 检查是否为表达式标点符号
    fn is_expression_punctuator(&self, ch: u8) -> bool;
}
```

### Napi 绑定接口

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct JsImport {
    /// 模块说明符（如果是安全字符串）
    pub n: Option<String>,
    /// Import 类型
    pub t: u8,
    /// 模块说明符开始位置
    pub s: u32,
    /// 模块说明符结束位置
    pub e: u32,
    /// 语句开始位置
    pub ss: u32,
    /// 语句结束位置
    pub se: u32,
    /// 动态 import 位置
    pub d: i32,
    /// Attributes 索引
    pub a: i32,
    /// Attributes 数组
    pub at: Option<Vec<Vec<String>>>,
}

#[napi(object)]
pub struct JsExport {
    /// 导出名称
    pub n: String,
    /// 本地名称
    pub ln: Option<String>,
    /// 导出名称开始位置
    pub s: u32,
    /// 导出名称结束位置
    pub e: u32,
    /// 本地名称开始位置
    pub ls: i32,
    /// 本地名称结束位置
    pub le: i32,
}

#[napi(object)]
pub struct JsParseResult {
    pub imports: Vec<JsImport>,
    pub exports: Vec<JsExport>,
    pub facade: bool,
    pub has_module_syntax: bool,
}

#[napi]
pub fn parse(source: String, name: Option<String>) -> Result<JsParseResult> {
    // 实现解析逻辑
}
```


## 数据模型

### 内存布局策略

与原始 C/WebAssembly 实现不同，Rust 版本使用 Rust 的所有权系统和标准库集合：

1. **Vec 替代链表**：使用 `Vec<Import>` 和 `Vec<Export>` 替代 C 的链表，提供更好的缓存局部性
2. **切片引用**：使用 `&[u8]` 和 `&str` 避免不必要的字符串复制
3. **栈分配**：小型结构（如 OpenToken）直接在栈上分配
4. **零拷贝**：位置信息存储为索引，避免复制源代码片段

### 字符串处理

```rust
// 原始实现使用 UTF-16，Rust 使用 UTF-8
// 需要在 napi 边界进行转换

// Rust 内部：UTF-8
let source_bytes = source.as_bytes();

// 提取模块说明符（零拷贝）
let module_specifier = &source[import.start..import.end];

// Napi 边界：转换为 JavaScript String (UTF-16)
let js_string = String::from_utf8_lossy(&source_bytes[start..end]).into_owned();
```

### 位置索引

所有位置使用字节索引（UTF-8），在 napi 层转换为 JavaScript 的字符索引（UTF-16）：

```rust
// Rust: 字节索引
pub struct Import {
    pub start: usize,  // UTF-8 字节索引
    pub end: usize,
}

// JavaScript: 字符索引
interface ImportSpecifier {
    s: number;  // UTF-16 字符索引
    e: number;
}

// 转换函数
fn byte_to_char_index(source: &str, byte_index: usize) -> usize {
    source[..byte_index].chars().count()
}
```

## 核心算法

### 两阶段解析算法

```rust
impl<'a> Lexer<'a> {
    pub fn parse(&mut self) -> Result<ParseResult, LexerError> {
        // Phase 1: 尝试 Facade 模式
        self.facade = true;
        let continue_full = self.parse_facade()?;
        
        if continue_full {
            // Phase 2: 完整解析
            self.facade = false;
            self.parse_full()?;
        }
        
        Ok(ParseResult {
            imports: std::mem::take(&mut self.imports),
            exports: std::mem::take(&mut self.exports),
            facade: self.facade,
            has_module_syntax: !self.imports.is_empty() || !self.exports.is_empty(),
        })
    }
    
    fn parse_facade(&mut self) -> Result<bool, LexerError> {
        while self.pos < self.end {
            let ch = self.source[self.pos];
            
            match ch {
                b'e' if self.is_keyword_start(self.pos) => {
                    if self.matches_keyword(b"export") {
                        self.try_parse_export_statement()?;
                        if !self.facade {
                            return Ok(true); // 切换到完整解析
                        }
                    }
                }
                b'i' if self.is_keyword_start(self.pos) => {
                    if self.matches_keyword(b"import") {
                        self.try_parse_import_statement()?;
                    }
                }
                b';' | b'\n' | b'\r' | b' ' | b'\t' => {
                    // 跳过空白和分号
                    self.pos += 1;
                }
                b'/' => {
                    // 跳过注释
                    if self.pos + 1 < self.end {
                        match self.source[self.pos + 1] {
                            b'/' => self.skip_line_comment()?,
                            b'*' => self.skip_block_comment()?,
                            _ => {
                                // 非模块语法，切换到完整解析
                                self.facade = false;
                                return Ok(true);
                            }
                        }
                    }
                }
                _ => {
                    // 非模块语法，切换到完整解析
                    self.facade = false;
                    return Ok(true);
                }
            }
        }
        
        Ok(false) // 完成 facade 解析
    }
}
```


### Import 解析算法

```rust
fn try_parse_import_statement(&mut self) -> Result<(), LexerError> {
    let start_pos = self.pos;
    self.pos += 6; // 跳过 "import"
    
    let ch = self.comment_whitespace(false)?;
    
    match ch {
        b'(' => {
            // 动态 import: import(...)
            self.parse_dynamic_import(start_pos)?;
        }
        b'.' => {
            // import.meta 或 import.source 或 import.defer
            self.pos += 1;
            let ch = self.comment_whitespace(false)?;
            
            if self.matches_keyword(b"meta") {
                self.add_import_meta(start_pos);
            } else if self.matches_keyword(b"source") {
                self.parse_source_phase_import(start_pos)?;
            } else if self.matches_keyword(b"defer") {
                self.parse_defer_phase_import(start_pos)?;
            }
        }
        b'"' | b'\'' => {
            // 字符串 import: import "module"
            self.parse_string_import(start_pos, ch)?;
        }
        _ => {
            // 命名 import: import { x } from "module"
            self.parse_named_import(start_pos)?;
        }
    }
    
    Ok(())
}

fn parse_dynamic_import(&mut self, start_pos: usize) -> Result<(), LexerError> {
    let dynamic_pos = self.pos;
    self.pos += 1; // 跳过 '('
    
    // 压入栈
    self.open_token_stack.push(OpenToken {
        state: OpenTokenState::ImportParen,
        pos: dynamic_pos,
    });
    
    let import_idx = self.imports.len();
    self.dynamic_import_stack.push(import_idx);
    
    // 创建 import 记录
    let mut import = Import {
        start: 0,
        end: 0,
        statement_start: start_pos,
        statement_end: 0,
        attr_index: None,
        dynamic: Some(dynamic_pos),
        safe: false,
        import_type: ImportType::Dynamic,
        attributes: Vec::new(),
    };
    
    // 尝试解析字符串字面量
    let ch = self.comment_whitespace(false)?;
    if ch == b'"' || ch == b'\'' {
        let str_start = self.pos + 1;
        self.string_literal(ch)?;
        import.start = str_start;
        import.end = self.pos - 1;
        import.safe = true;
        
        // 检测 attributes
        let ch = self.comment_whitespace(false)?;
        if ch == b',' {
            import.attr_index = Some(self.pos);
        }
    }
    
    self.imports.push(import);
    Ok(())
}
```

### Export 解析算法

```rust
fn try_parse_export_statement(&mut self) -> Result<(), LexerError> {
    let start_pos = self.pos;
    self.pos += 6; // 跳过 "export"
    
    let ch = self.comment_whitespace(false)?;
    
    match ch {
        b'{' => {
            // export { a, b as c }
            self.parse_export_list()?;
        }
        b'*' => {
            // export * from "module"
            self.parse_export_star()?;
        }
        b'd' if self.matches_keyword(b"default") => {
            // export default
            self.parse_export_default(start_pos)?;
        }
        b'v' | b'l' | b'c' => {
            // export var/let/const
            self.parse_export_declaration()?;
        }
        b'f' if self.matches_keyword(b"function") => {
            // export function
            self.parse_export_function()?;
        }
        b'c' if self.matches_keyword(b"class") => {
            // export class
            self.parse_export_class()?;
        }
        b'a' if self.matches_keyword(b"async") => {
            // export async function
            self.parse_export_async_function()?;
        }
        _ => {
            // 不是有效的 export 语法
            self.facade = false;
        }
    }
    
    Ok(())
}

fn parse_export_list(&mut self) -> Result<(), LexerError> {
    self.pos += 1; // 跳过 '{'
    
    loop {
        let ch = self.comment_whitespace(false)?;
        
        if ch == b'}' {
            self.pos += 1;
            break;
        }
        
        // 读取本地名称
        let local_start = self.pos;
        self.read_identifier()?;
        let local_end = self.pos;
        
        let ch = self.comment_whitespace(false)?;
        
        let (export_start, export_end) = if self.matches_keyword(b"as") {
            self.pos += 2;
            let ch = self.comment_whitespace(false)?;
            
            let export_start = self.pos;
            if ch == b'"' || ch == b'\'' {
                self.string_literal(ch)?;
            } else {
                self.read_identifier()?;
            }
            (export_start, self.pos)
        } else {
            (local_start, local_end)
        };
        
        self.exports.push(Export {
            start: export_start,
            end: export_end,
            local_start: Some(local_start),
            local_end: Some(local_end),
        });
        
        let ch = self.comment_whitespace(false)?;
        if ch == b',' {
            self.pos += 1;
        } else if ch != b'}' {
            return Err(LexerError::UnexpectedToken(self.pos));
        }
    }
    
    Ok(())
}
```


### 正则表达式 vs 除法运算符判断

```rust
fn handle_slash(&mut self) -> Result<(), LexerError> {
    let last_ch = if self.last_token_pos < self.end {
        self.source[self.last_token_pos]
    } else {
        0
    };
    
    // 判断是否为正则表达式
    let is_regex = self.is_expression_punctuator(last_ch)
        || (last_ch == b')' && self.is_paren_keyword())
        || (last_ch == b'}' && self.is_expression_terminator())
        || self.is_expression_keyword()
        || last_ch == 0; // 文件开头
    
    if is_regex {
        self.regular_expression()?;
        self.last_slash_was_division = false;
    } else {
        // 除法运算符，继续
        self.last_slash_was_division = true;
        self.pos += 1;
    }
    
    Ok(())
}

fn is_expression_punctuator(&self, ch: u8) -> bool {
    matches!(ch, 
        b'+' | b'-' | b'*' | b'%' | b'&' | b'|' | b'^' | b'~' |
        b'!' | b'<' | b'>' | b'=' | b'?' | b':' | b';' | b',' |
        b'(' | b'[' | b'{' | b'\n' | b'\r'
    )
}

fn is_paren_keyword(&self) -> bool {
    // 检查栈顶的 '(' 是否对应 while/for/if
    if let Some(token) = self.open_token_stack.last() {
        if token.state == OpenTokenState::AnyParen {
            // 回溯检查关键字
            return self.check_keyword_before(token.pos, &[b"while", b"for", b"if"]);
        }
    }
    false
}

fn is_expression_terminator(&self) -> bool {
    // 检查栈顶的 '{' 是否为表达式终结符
    if let Some(token) = self.open_token_stack.last() {
        if token.state == OpenTokenState::AnyBrace {
            // 检查是否为函数/类/try-catch 等
            return self.check_keyword_before(token.pos, &[
                b"function", b"class", b"try", b"catch", b"finally"
            ]);
        }
    }
    false
}
```

### Import Attributes 解析

```rust
fn parse_import_attributes(&mut self, import_idx: usize) -> Result<(), LexerError> {
    // 检测 "with" 关键字
    if !self.matches_keyword(b"with") {
        return Ok(());
    }
    
    self.pos += 4; // 跳过 "with"
    let ch = self.comment_whitespace(true)?;
    
    if ch != b'{' {
        return Ok(());
    }
    
    self.pos += 1; // 跳过 '{'
    
    let mut attributes = Vec::new();
    
    loop {
        let ch = self.comment_whitespace(true)?;
        
        if ch == b'}' {
            self.pos += 1;
            break;
        }
        
        // 解析 key
        let key_start = self.pos;
        if ch == b'"' || ch == b'\'' {
            self.string_literal(ch)?;
        } else {
            self.read_identifier()?;
        }
        let key_end = self.pos;
        
        // 期望 ':'
        let ch = self.comment_whitespace(true)?;
        if ch != b':' {
            return Err(LexerError::ExpectedColon(self.pos));
        }
        self.pos += 1;
        
        // 解析 value (必须是字符串)
        let ch = self.comment_whitespace(true)?;
        if ch != b'"' && ch != b'\'' {
            return Err(LexerError::ExpectedString(self.pos));
        }
        
        let value_start = self.pos;
        self.string_literal(ch)?;
        let value_end = self.pos;
        
        attributes.push(Attribute {
            key_start,
            key_end,
            value_start,
            value_end,
        });
        
        let ch = self.comment_whitespace(true)?;
        if ch == b',' {
            self.pos += 1;
        } else if ch != b'}' {
            return Err(LexerError::UnexpectedToken(self.pos));
        }
    }
    
    // 更新 import 的 attributes
    if let Some(import) = self.imports.get_mut(import_idx) {
        import.attributes = attributes;
    }
    
    Ok(())
}
```


### 字符串转义处理

```rust
fn read_string(&mut self, start: usize, quote: u8) -> Result<String, LexerError> {
    let mut result = String::new();
    let mut chunk_start = start;
    let mut pos = start;
    
    loop {
        if pos >= self.end {
            return Err(LexerError::UnterminatedString(start));
        }
        
        let ch = self.source[pos];
        
        if ch == quote {
            // 字符串结束
            result.push_str(&String::from_utf8_lossy(&self.source[chunk_start..pos]));
            break;
        }
        
        if ch == b'\\' {
            // 转义字符
            result.push_str(&String::from_utf8_lossy(&self.source[chunk_start..pos]));
            pos += 1;
            
            if pos >= self.end {
                return Err(LexerError::UnterminatedString(start));
            }
            
            let escaped = self.source[pos];
            pos += 1;
            
            match escaped {
                b'n' => result.push('\n'),
                b'r' => result.push('\r'),
                b't' => result.push('\t'),
                b'b' => result.push('\u{0008}'),
                b'v' => result.push('\u{000B}'),
                b'f' => result.push('\u{000C}'),
                b'0' => result.push('\0'),
                b'x' => {
                    // \xHH
                    let hex = self.read_hex_chars(2)?;
                    result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                    pos += 2;
                }
                b'u' => {
                    // \uHHHH 或 \u{HHHHHH}
                    if pos < self.end && self.source[pos] == b'{' {
                        pos += 1;
                        let hex = self.read_hex_until(b'}')?;
                        result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                        pos = self.pos;
                    } else {
                        let hex = self.read_hex_chars(4)?;
                        result.push(char::from_u32(hex).unwrap_or('\u{FFFD}'));
                        pos += 4;
                    }
                }
                b'\r' => {
                    // 行继续符
                    if pos < self.end && self.source[pos] == b'\n' {
                        pos += 1;
                    }
                }
                b'\n' => {
                    // 行继续符
                }
                _ => {
                    // 其他字符直接添加
                    result.push(escaped as char);
                }
            }
            
            chunk_start = pos;
        } else if ch == b'\r' || ch == b'\n' {
            // 未转义的换行符
            return Err(LexerError::UnterminatedString(start));
        } else {
            pos += 1;
        }
    }
    
    self.pos = pos + 1; // 跳过闭引号
    Ok(result)
}

fn read_hex_chars(&mut self, count: usize) -> Result<u32, LexerError> {
    let mut value = 0u32;
    for _ in 0..count {
        if self.pos >= self.end {
            return Err(LexerError::InvalidEscape(self.pos));
        }
        let ch = self.source[self.pos];
        let digit = match ch {
            b'0'..=b'9' => (ch - b'0') as u32,
            b'a'..=b'f' => (ch - b'a' + 10) as u32,
            b'A'..=b'F' => (ch - b'A' + 10) as u32,
            _ => return Err(LexerError::InvalidEscape(self.pos)),
        };
        value = value * 16 + digit;
        self.pos += 1;
    }
    Ok(value)
}
```


## 正确性属性

属性是一种特征或行为，应该在系统的所有有效执行中保持为真——本质上是关于系统应该做什么的形式化陈述。属性作为人类可读规范和机器可验证正确性保证之间的桥梁。

### 属性反思

在编写属性之前，我们需要识别并消除冗余：

**冗余分析**：
1. 需求 1.1 和 1.2（解析所有 import/export）可以合并为一个综合属性：解析完整性
2. 需求 3.1-3.8（各种 import 类型）可以合并为：正确的类型标记属性
3. 需求 4.1-4.7（各种 export 类型）可以合并为：正确的导出提取属性
4. 需求 6.1-6.5（正则/除法歧义）可以合并为：上下文相关的 slash 解析属性
5. 需求 8.1-8.5（字符串处理）可以合并为：字符串解析完整性属性
6. 需求 9.1-9.4（注释处理）可以合并为：注释跳过属性
7. 需求 5.2 和 5.3（转义处理）是字符串处理的一部分，可以通过 round-trip 属性测试

**保留的独立属性**：
- Facade 模式检测（需求 2.1-2.2）
- Import attributes 解析（需求 5.1, 5.4）
- 动态 import 安全性标记（需求 3.3-3.4）
- 括号匹配完整性（需求 7.5）
- UTF-16 编码转换（需求 11.4）
- 输出对齐（需求 13.2, 13.7）

### 核心属性

#### 属性 1：解析完整性

*对于任何*有效的 JavaScript 模块代码，解析后的 imports 和 exports 数组应包含源代码中的所有 import 和 export 语句，且每个语句的位置信息应准确指向源代码中的对应位置。

**验证需求**：1.1, 1.2

#### 属性 2：Import 类型标记正确性

*对于任何*包含 import 语句的代码，每个 import 的类型标记（Static, Dynamic, ImportMeta, StaticSourcePhase, DynamicSourcePhase, StaticDeferPhase, DynamicDeferPhase）应与其语法形式精确匹配。

**验证需求**：3.1, 3.2, 3.5, 3.7, 3.8

#### 属性 3：动态 Import 安全性标记

*对于任何*动态 import 表达式，如果参数是字符串字面量，则 safe 标志应为 true 且模块名称应被正确提取；如果参数是表达式，则 safe 标志应为 false。

**验证需求**：3.3, 3.4

#### 属性 4：Export 提取完整性

*对于任何*export 语句（命名导出、默认导出、重导出、通配符导出、声明导出、解构导出），导出名称和本地名称（如果不同）应被正确提取，且位置信息应准确。

**验证需求**：4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7

#### 属性 5：Import Attributes 解析完整性

*对于任何*包含 with 子句的 import 语句，所有键值对应被解析为 attributes 数组，且顺序应与源代码中的顺序一致。

**验证需求**：5.1, 5.4

#### 属性 6：字符串转义 Round-Trip

*对于任何*包含转义字符的字符串字面量（在 import 说明符或 attributes 中），解析后提取的字符串值应正确处理所有转义序列（\n, \r, \t, \xHH, \uHHHH, \u{HHHHHH} 等）。

**验证需求**：5.2, 5.3, 8.3

#### 属性 7：Facade 模式检测

*对于任何*仅包含 import/export 语句和注释/空白的代码，facade 标志应为 true；对于包含其他 JavaScript 语法的代码，facade 标志应为 false。

**验证需求**：2.1, 2.2

#### 属性 8：正则表达式 vs 除法运算符上下文判断

*对于任何*包含 '/' 字符的代码，当 '/' 前面是表达式标点符号、表达式关键字、或特定上下文的 ')' 或 '}' 时，应被解析为正则表达式；当前面是标识符或数字时，应被解析为除法运算符。

**验证需求**：6.1, 6.2, 6.3, 6.4, 6.5

#### 属性 9：字符串解析完整性

*对于任何*包含单引号或双引号字符串的代码，字符串应被正确扫描到匹配的闭引号，且转义的引号不应终止字符串。

**验证需求**：8.1, 8.2

#### 属性 10：模板字符串嵌套处理

*对于任何*包含模板字符串的代码，包括嵌套的 ${} 表达式插值，应正确跟踪嵌套层级并完整解析模板字符串。

**验证需求**：8.4, 8.5

#### 属性 11：注释跳过完整性

*对于任何*包含单行注释（//）或多行注释（/* */）的代码，注释内容应被完全跳过，不影响 import/export 解析，即使注释中包含看似 import/export 的文本。

**验证需求**：9.1, 9.2, 9.3, 9.4

#### 属性 12：动态 Import 括号匹配

*对于任何*动态 import 表达式，无论参数表达式多么复杂（包含嵌套括号），statement_end 应准确指向匹配的闭括号位置。

**验证需求**：7.5

#### 属性 13：模板字符串括号匹配

*对于任何*包含模板字符串的代码，${} 中的嵌套表达式（包括嵌套的模板字符串）应被正确处理，不影响外层模板字符串的解析。

**验证需求**：7.6

#### 属性 14：UTF-16 位置索引转换

*对于任何*包含多字节 Unicode 字符的代码，Rust 内部的 UTF-8 字节索引应正确转换为 JavaScript 的 UTF-16 字符索引，使得 JavaScript 层可以使用 slice 正确提取子字符串。

**验证需求**：11.4

#### 属性 15：输出对齐（与原始实现）

*对于任何*有效的 JavaScript 模块代码，Rust 实现的解析输出（imports 数组、exports 数组、facade 标志、hasModuleSyntax 标志）应与 es-module-lexer 原始实现的输出完全一致。

**验证需求**：13.2, 13.7


## 错误处理

### 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("Unexpected token at position {0}")]
    UnexpectedToken(usize),
    
    #[error("Unterminated string starting at position {0}")]
    UnterminatedString(usize),
    
    #[error("Unterminated comment starting at position {0}")]
    UnterminatedComment(usize),
    
    #[error("Unterminated regular expression starting at position {0}")]
    UnterminatedRegex(usize),
    
    #[error("Invalid escape sequence at position {0}")]
    InvalidEscape(usize),
    
    #[error("Expected colon at position {0}")]
    ExpectedColon(usize),
    
    #[error("Expected string at position {0}")]
    ExpectedString(usize),
    
    #[error("Stack overflow at position {0}")]
    StackOverflow(usize),
    
    #[error("Invalid UTF-8 in source code")]
    InvalidUtf8,
}
```

### 错误处理策略

1. **语法错误**：返回 `LexerError` 并指示错误位置
2. **容错解析**：对于非关键错误，尝试恢复并继续解析
3. **错误传播**：通过 `Result` 类型传播错误到 napi 层
4. **JavaScript 错误**：在 napi 层将 `LexerError` 转换为 JavaScript `Error`

### Napi 错误转换

```rust
impl From<LexerError> for napi::Error {
    fn from(err: LexerError) -> Self {
        napi::Error::new(
            napi::Status::GenericFailure,
            format!("{}", err),
        )
    }
}
```

## 测试策略

### 双重测试方法

本项目采用单元测试和属性测试相结合的方法：

- **单元测试**：验证特定示例、边缘情况和错误条件
- **属性测试**：验证跨所有输入的通用属性

两者是互补的，对于全面覆盖都是必要的。

### 单元测试

单元测试专注于：
- 特定示例，展示正确行为
- 组件之间的集成点
- 边缘情况和错误条件

**示例**：
```rust
#[test]
fn test_static_import() {
    let source = r#"import foo from 'bar';"#;
    let mut lexer = Lexer::new(source);
    let result = lexer.parse().unwrap();
    
    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].import_type, ImportType::Static);
    assert_eq!(&source[result.imports[0].start..result.imports[0].end], "bar");
}

#[test]
fn test_unterminated_string() {
    let source = r#"import foo from 'bar"#;
    let mut lexer = Lexer::new(source);
    let result = lexer.parse();
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LexerError::UnterminatedString(_)));
}
```

### 属性测试

属性测试使用 `proptest` 或 `quickcheck` 库，通过随机生成输入验证通用属性。

**配置**：
- 每个属性测试最少 100 次迭代
- 每个测试必须引用设计文档中的属性
- 标签格式：`Feature: es-module-lexer-rs, Property {number}: {property_text}`

**示例**：
```rust
use proptest::prelude::*;

// Feature: es-module-lexer-rs, Property 1: 解析完整性
proptest! {
    #[test]
    fn prop_parse_completeness(
        imports in prop::collection::vec(arb_import_statement(), 0..10),
        exports in prop::collection::vec(arb_export_statement(), 0..10)
    ) {
        let source = format!("{}\n{}", 
            imports.join("\n"), 
            exports.join("\n")
        );
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse().unwrap();
        
        // 验证所有 import 都被解析
        prop_assert_eq!(result.imports.len(), imports.len());
        
        // 验证所有 export 都被解析
        prop_assert_eq!(result.exports.len(), exports.len());
        
        // 验证位置信息准确
        for import in &result.imports {
            let extracted = &source[import.start..import.end];
            prop_assert!(is_valid_module_specifier(extracted));
        }
    }
}

// Feature: es-module-lexer-rs, Property 2: Import 类型标记正确性
proptest! {
    #[test]
    fn prop_import_type_correctness(import_type in arb_import_type()) {
        let source = generate_import_of_type(import_type);
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse().unwrap();
        
        prop_assert_eq!(result.imports.len(), 1);
        prop_assert_eq!(result.imports[0].import_type, import_type);
    }
}

// Feature: es-module-lexer-rs, Property 6: 字符串转义 Round-Trip
proptest! {
    #[test]
    fn prop_string_escape_roundtrip(s in "\\PC*") {
        let escaped = escape_string(&s);
        let source = format!("import foo from '{}';", escaped);
        
        let mut lexer = Lexer::new(&source);
        let result = lexer.parse().unwrap();
        
        if let Some(n) = &result.imports[0].n {
            prop_assert_eq!(n, &s);
        }
    }
}
```

### 对比测试（与原始实现）

```typescript
// Feature: es-module-lexer-rs, Property 15: 输出对齐
import { describe, test, expect } from 'vitest';
import { parse as parseOriginal } from 'es-module-lexer';
import { parse as parseRust } from 'es-module-lexer-rs';

describe('Output alignment', () => {
  test.each([
    `import foo from 'bar';`,
    `export const x = 1;`,
    `import('dynamic');`,
    `import foo from 'bar' with { type: 'json' };`,
    // ... 更多测试用例
  ])('should produce identical output for: %s', async (source) => {
    const [importsOrig, exportsOrig, facadeOrig, hasModuleOrig] = parseOriginal(source);
    const [importsRust, exportsRust, facadeRust, hasModuleRust] = parseRust(source);
    
    expect(importsRust).toEqual(importsOrig);
    expect(exportsRust).toEqual(exportsOrig);
    expect(facadeRust).toBe(facadeOrig);
    expect(hasModuleRust).toBe(hasModuleOrig);
  });
});
```

### 集成测试

使用真实库代码测试：
```typescript
import { readFileSync } from 'fs';
import { parse } from 'es-module-lexer-rs';

describe('Real-world code', () => {
  test('should parse angular.js', () => {
    const source = readFileSync('test/samples/angular.js', 'utf-8');
    const [imports, exports] = parse(source);
    
    expect(imports.length).toBeGreaterThan(0);
    expect(exports.length).toBeGreaterThan(0);
  });
});
```

### 性能基准测试

```typescript
import { bench, describe } from 'vitest';
import { parse as parseOriginal } from 'es-module-lexer';
import { parse as parseRust } from 'es-module-lexer-rs';

describe('Performance', () => {
  const largeFile = readFileSync('test/samples/angular.js', 'utf-8');
  
  bench('es-module-lexer (original)', () => {
    parseOriginal(largeFile);
  });
  
  bench('es-module-lexer-rs (Rust)', () => {
    parseRust(largeFile);
  });
});
```


## 性能优化策略

### Rust 特定优化

#### 1. 零拷贝字符串处理

```rust
// 避免：复制字符串
let module_name = source[start..end].to_string();

// 优先：使用切片引用
let module_name = &source[start..end];

// 只在必要时（如 napi 边界）才转换为 String
```

#### 2. 预分配容器

```rust
// 预估 imports/exports 数量，减少重新分配
let mut imports = Vec::with_capacity(estimated_count);
let mut exports = Vec::with_capacity(estimated_count);
```

#### 3. 内联小函数

```rust
#[inline(always)]
fn is_whitespace(ch: u8) -> bool {
    matches!(ch, b' ' | b'\t' | b'\n' | b'\r')
}
```

#### 4. 使用 &[u8] 而非 &str

```rust
// 对于字节级操作，使用 &[u8] 避免 UTF-8 验证开销
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),  // 一次性转换
            // ...
        }
    }
}
```

#### 5. 避免边界检查

```rust
// 使用 get_unchecked 在安全的情况下避免边界检查
// 注意：只在确保安全的情况下使用
if self.pos < self.end {
    let ch = unsafe { *self.source.get_unchecked(self.pos) };
}
```

#### 6. 使用 SmallVec 优化小集合

```rust
use smallvec::SmallVec;

// 大多数 import 没有 attributes，使用 SmallVec 避免堆分配
pub struct Import {
    // ...
    pub attributes: SmallVec<[Attribute; 2]>,
}
```

### 算法优化

#### 1. 单次遍历

整个解析过程只遍历源代码一次，避免回溯（除了正则/除法歧义判断）。

#### 2. 早期退出

```rust
// Facade 模式：遇到非模块语法立即切换
if !is_module_syntax(ch) {
    self.facade = false;
    return Ok(true); // 切换到完整解析
}
```

#### 3. 字符比较优化

```rust
// 使用字节比较而非字符串比较
fn matches_keyword(&self, keyword: &[u8]) -> bool {
    let end = self.pos + keyword.len();
    if end > self.end {
        return false;
    }
    &self.source[self.pos..end] == keyword
}
```

### 内存布局优化

#### 1. 紧凑的数据结构

```rust
// 使用 u32 而非 usize 节省内存（假设源文件 < 4GB）
pub struct Import {
    pub start: u32,
    pub end: u32,
    // ...
}
```

#### 2. 位标志

```rust
// 使用位标志压缩布尔值
pub struct Import {
    // ...
    flags: u8,  // bit 0: safe, bit 1-7: reserved
}

impl Import {
    fn is_safe(&self) -> bool {
        self.flags & 0x01 != 0
    }
    
    fn set_safe(&mut self, safe: bool) {
        if safe {
            self.flags |= 0x01;
        } else {
            self.flags &= !0x01;
        }
    }
}
```

### Napi 优化

#### 1. 批量转换

```rust
// 一次性转换所有 imports，避免多次 napi 调用
#[napi]
pub fn parse(source: String) -> Result<JsParseResult> {
    let mut lexer = Lexer::new(&source);
    let result = lexer.parse()?;
    
    // 批量转换
    let imports = result.imports
        .into_iter()
        .map(|imp| convert_import(&source, imp))
        .collect();
    
    Ok(JsParseResult { imports, /* ... */ })
}
```

#### 2. 避免不必要的字符串分配

```rust
// 只在需要时才提取字符串
pub fn convert_import(source: &str, import: Import) -> JsImport {
    JsImport {
        n: if import.safe {
            Some(source[import.start..import.end].to_string())
        } else {
            None
        },
        // ...
    }
}
```

### 性能目标

基于原始实现的性能数据，Rust 版本的目标：

| 指标 | 原始 Wasm | 目标 Rust | 提升 |
|------|-----------|-----------|------|
| 冷启动 (3MB) | 18ms | ≤14ms | ≥22% |
| 热启动 (3MB) | 14ms | ≤11ms | ≥21% |
| 吞吐量 | 223 MB/s | ≥270 MB/s | ≥21% |
| 内存使用 | 基准 | -20% | 20% 减少 |

### 性能测量

```rust
// 使用 criterion 进行精确的性能测量
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    let source = std::fs::read_to_string("test/samples/angular.js").unwrap();
    
    c.bench_function("parse angular.js", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&source));
            lexer.parse().unwrap()
        });
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

## API 设计

### Rust 公共 API

```rust
// crates/es-module-lexer/src/lib.rs

/// 解析 JavaScript 模块源代码
pub fn parse(source: &str) -> Result<ParseResult, LexerError>;

/// Lexer 结构（用于高级用法）
pub struct Lexer<'a> { /* ... */ }

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn parse(&mut self) -> Result<ParseResult, LexerError>;
}

/// 解析结果
pub struct ParseResult {
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub facade: bool,
    pub has_module_syntax: bool,
}

/// Import 说明符
pub struct Import {
    pub start: usize,
    pub end: usize,
    pub statement_start: usize,
    pub statement_end: usize,
    pub attr_index: Option<usize>,
    pub dynamic: Option<usize>,
    pub safe: bool,
    pub import_type: ImportType,
    pub attributes: Vec<Attribute>,
}

/// Export 说明符
pub struct Export {
    pub start: usize,
    pub end: usize,
    pub local_start: Option<usize>,
    pub local_end: Option<usize>,
}

/// Import 类型
pub enum ImportType {
    Static = 1,
    Dynamic = 2,
    ImportMeta = 3,
    StaticSourcePhase = 4,
    DynamicSourcePhase = 5,
    StaticDeferPhase = 6,
    DynamicDeferPhase = 7,
}

/// Import attribute
pub struct Attribute {
    pub key_start: usize,
    pub key_end: usize,
    pub value_start: usize,
    pub value_end: usize,
}
```

### JavaScript/TypeScript API

```typescript
// packages/es-module-lexer-rs/src/index.ts

/**
 * 解析 JavaScript 模块源代码
 * @param source - 源代码字符串
 * @param name - 可选的文件名（用于错误消息）
 * @returns 包含 imports、exports、facade 和 hasModuleSyntax 的元组
 */
export function parse(
  source: string,
  name?: string
): readonly [
  imports: ReadonlyArray<ImportSpecifier>,
  exports: ReadonlyArray<ExportSpecifier>,
  facade: boolean,
  hasModuleSyntax: boolean
];

/**
 * Import 说明符
 */
export interface ImportSpecifier {
  /** 模块说明符（如果是安全字符串） */
  n: string | undefined;
  /** Import 类型 */
  t: ImportType;
  /** 模块说明符开始位置 */
  s: number;
  /** 模块说明符结束位置 */
  e: number;
  /** 语句开始位置 */
  ss: number;
  /** 语句结束位置 */
  se: number;
  /** 动态 import 位置 */
  d: number;
  /** Attributes 索引 */
  a: number;
  /** Attributes 数组 */
  at: ReadonlyArray<readonly [string, string]> | null;
}

/**
 * Export 说明符
 */
export interface ExportSpecifier {
  /** 导出名称 */
  n: string;
  /** 本地名称 */
  ln: string | undefined;
  /** 导出名称开始位置 */
  s: number;
  /** 导出名称结束位置 */
  e: number;
  /** 本地名称开始位置 */
  ls: number;
  /** 本地名称结束位置 */
  le: number;
}

/**
 * Import 类型枚举
 */
export enum ImportType {
  Static = 1,
  Dynamic = 2,
  ImportMeta = 3,
  StaticSourcePhase = 4,
  DynamicSourcePhase = 5,
  StaticDeferPhase = 6,
  DynamicDeferPhase = 7,
}
```

### 使用示例

```typescript
import { parse } from 'es-module-lexer-rs';

const source = `
  import foo from 'bar';
  import('dynamic');
  export const x = 1;
`;

const [imports, exports, facade, hasModuleSyntax] = parse(source);

console.log('Imports:', imports);
// [
//   { n: 'bar', t: 1, s: 16, e: 19, ss: 3, se: 26, d: -1, a: -1, at: null },
//   { n: undefined, t: 2, s: 0, e: 0, ss: 29, se: 44, d: 36, a: -1, at: null }
// ]

console.log('Exports:', exports);
// [{ n: 'x', ln: undefined, s: 61, e: 62, ls: -1, le: -1 }]

console.log('Facade:', facade);
// false

console.log('Has module syntax:', hasModuleSyntax);
// true
```

---

**文档版本**: 1.0  
**创建日期**: 2025-01-27  
**作者**: Kiro AI Assistant

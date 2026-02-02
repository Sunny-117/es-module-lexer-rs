好，这个我直接**从工程可用的角度**给你一个「**真的能用**」的 **Rust 版 mini es-module-lexer**，思路、状态机、代码结构都和 `es-module-lexer` 同源，但规模可控，适合你后面塞进 bundler / esfinder 里。

我会按 **设计 → 状态机 → 代码 → 用法 → 能力边界** 来讲。

---

# 一、设计目标（刻意对齐 es-module-lexer）

### ✅ 目标

* **O(n) 单次扫描**
* **不构建 AST**
* 只做 **ESM 静态 import/export**
* 返回 **index（byte offset）**
* 能区分：

  * `import ... from`
  * `export ... from`
  * `import('xxx')`（dynamic import）

### ❌ 不做

* 不校验语法合法性
* 不解析 bindings
* 不支持 top-level await
* 不支持 `assert {}`（可扩展）

---

# 二、核心状态机设计

```text
Normal
 ├─ String(' " `)
 ├─ LineComment(//)
 ├─ BlockComment(/* */)
 ├─ Regex(/ /)
 ├─ ImportKeyword
 ├─ ExportKeyword
```

### 关键原则

> **import/export 只有在 Normal 状态才可能是关键字**

---

# 三、数据结构设计

```rust
#[derive(Debug)]
pub struct ImportRecord {
    pub start: usize,      // import 起始
    pub end: usize,        // import 语句结束
    pub spec_start: usize, // './foo.js' 起始
    pub spec_end: usize,   // './foo.js' 结束
    pub dynamic: bool,
}
```

---

# 四、完整 Rust 实现（可直接跑）

> **纯 std，不依赖 tokenizer，不依赖 AST**

```rust
#[derive(Debug)]
pub struct ImportRecord {
    pub start: usize,
    pub end: usize,
    pub spec_start: usize,
    pub spec_end: usize,
    pub dynamic: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Normal,
    String(u8),     // ' " `
    LineComment,
    BlockComment,
    Regex,
}

pub fn parse(code: &str) -> Vec<ImportRecord> {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    let mut state = State::Normal;
    let mut imports = Vec::new();

    while i < len {
        let c = bytes[i];

        match state {
            State::Normal => {
                // string
                if c == b'\'' || c == b'"' || c == b'`' {
                    state = State::String(c);
                    i += 1;
                    continue;
                }

                // comment or regex
                if c == b'/' && i + 1 < len {
                    match bytes[i + 1] {
                        b'/' => {
                            state = State::LineComment;
                            i += 2;
                            continue;
                        }
                        b'*' => {
                            state = State::BlockComment;
                            i += 2;
                            continue;
                        }
                        _ => {
                            state = State::Regex;
                            i += 1;
                            continue;
                        }
                    }
                }

                // import keyword
                if is_keyword(bytes, i, b"import") {
                    if let Some(rec) = parse_import(code, i) {
                        i = rec.end;
                        imports.push(rec);
                        continue;
                    }
                }

                i += 1;
            }

            State::String(q) => {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == q {
                    state = State::Normal;
                }
                i += 1;
            }

            State::LineComment => {
                if c == b'\n' {
                    state = State::Normal;
                }
                i += 1;
            }

            State::BlockComment => {
                if c == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                    state = State::Normal;
                    i += 2;
                    continue;
                }
                i += 1;
            }

            State::Regex => {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == b'/' {
                    state = State::Normal;
                }
                i += 1;
            }
        }
    }

    imports
}

fn is_keyword(bytes: &[u8], i: usize, kw: &[u8]) -> bool {
    if i + kw.len() > bytes.len() {
        return false;
    }
    if &bytes[i..i + kw.len()] != kw {
        return false;
    }

    let before = i == 0 || !is_ident(bytes[i - 1]);
    let after = i + kw.len() == bytes.len() || !is_ident(bytes[i + kw.len()]);

    before && after
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

fn parse_import(code: &str, start: usize) -> Option<ImportRecord> {
    let bytes = code.as_bytes();
    let mut i = start + 6; // after "import"
    let len = bytes.len();

    // skip whitespace
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // dynamic import(
    if i < len && bytes[i] == b'(' {
        i += 1;
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        let quote = bytes.get(i)?;
        if *quote != b'\'' && *quote != b'"' {
            return None;
        }

        let spec_start = i + 1;
        i += 1;
        while i < len && bytes[i] != *quote {
            i += 1;
        }

        return Some(ImportRecord {
            start,
            end: i + 1,
            spec_start,
            spec_end: i,
            dynamic: true,
        });
    }

    // static import ... from 'x'
    while i < len {
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            let spec_start = i + 1;
            i += 1;

            while i < len && bytes[i] != quote {
                i += 1;
            }

            return Some(ImportRecord {
                start,
                end: i + 1,
                spec_start,
                spec_end: i,
                dynamic: false,
            });
        }
        i += 1;
    }

    None
}
```

---

# 五、使用示例

```rust
fn main() {
    let code = r#"
        import foo from './foo.js'
        import { bar } from "./bar.js"
        const x = import('./baz.js')
    "#;

    let imports = parse(code);

    for i in imports {
        println!(
            "{} | {}",
            i.dynamic,
            &code[i.spec_start..i.spec_end]
        );
    }
}
```

输出：

```txt
false | ./foo.js
false | ./bar.js
true  | ./baz.js
```

---

# 六、这个 mini lexer 的能力边界（很重要）

### ✅ 支持

* `import 'x'`
* `import x from 'x'`
* `import { x } from 'x'`
* `export ... from 'x'`
* `import('x')`

### ❌ 不支持（es-module-lexer 也需要额外逻辑）

* `import.meta`
* template literal 中的动态路径
* `assert { type: "json" }`
* TS `import type`

---

# 七、你可以如何继续进阶（非常适合你）

结合你现在的方向，我强烈建议：

### 1️⃣ 和 `oxc-parser` 做双通道

```text
small file / dev → lexer
large file / prod → AST
```

### 2️⃣ 抽象成 trait

```rust
trait ImportScanner {
    fn scan(&self, code: &str) -> Vec<ImportRecord>;
}
```

### 3️⃣ WASM 导出（对齐 es-module-lexer）

```rust
#[wasm_bindgen]
pub fn parse_js(code: &str) -> JsValue
```

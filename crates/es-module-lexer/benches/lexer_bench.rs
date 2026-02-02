use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use es_module_lexer::parse;
use std::fs;

fn bench_simple_import(c: &mut Criterion) {
    let source = r#"import foo from 'bar';"#;
    c.bench_function("simple_import", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_multiple_imports(c: &mut Criterion) {
    let source = r#"
        import foo from 'bar';
        import { a, b, c } from 'module';
        import * as ns from 'namespace';
    "#;
    c.bench_function("multiple_imports", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_dynamic_imports(c: &mut Criterion) {
    let source = r#"
        import('dynamic1');
        import('dynamic2');
        const mod = import('dynamic3');
    "#;
    c.bench_function("dynamic_imports", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_exports(c: &mut Criterion) {
    let source = r#"
        export const x = 1;
        export function foo() {}
        export default class Bar {}
        export { a, b as c } from 'module';
    "#;
    c.bench_function("exports", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_import_attributes(c: &mut Criterion) {
    let source = r#"
        import data from './data.json' with { type: 'json' };
        import styles from './styles.css' with { type: 'css' };
    "#;
    c.bench_function("import_attributes", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_complex_module(c: &mut Criterion) {
    let source = r#"
        import foo from 'bar';
        import { a, b, c } from 'module';
        import * as ns from 'namespace';
        import('dynamic');
        
        export const x = 1;
        export function test() {
            const regex = /import\s+from/;
            const str = "import 'fake'";
            return `template ${import.meta.url}`;
        }
        
        export default class MyClass {
            method() {
                import('lazy').then(m => m.default);
            }
        }
    "#;
    c.bench_function("complex_module", |b| {
        b.iter(|| parse(black_box(source)))
    });
}

fn bench_real_world_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_files");
    
    // Test files of various sizes
    let test_files = vec![
        ("magic-string.js", "../../es-module-lexer/test/samples/magic-string.js"),
        ("magic-string.min.js", "../../es-module-lexer/test/samples/magic-string.min.js"),
        ("d3.js", "../../es-module-lexer/test/samples/d3.js"),
        ("d3.min.js", "../../es-module-lexer/test/samples/d3.min.js"),
        ("rollup.js", "../../es-module-lexer/test/samples/rollup.js"),
        ("rollup.min.js", "../../es-module-lexer/test/samples/rollup.min.js"),
        ("angular.js", "../../es-module-lexer/test/samples/angular.js"),
        ("angular.min.js", "../../es-module-lexer/test/samples/angular.min.js"),
    ];
    
    for (name, path) in test_files {
        if let Ok(source) = fs::read_to_string(path) {
            let size = source.len();
            group.throughput(Throughput::Bytes(size as u64));
            group.bench_with_input(BenchmarkId::new(name, size), &source, |b, s| {
                b.iter(|| parse(black_box(s)))
            });
        }
    }
    
    group.finish();
}

fn bench_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_sizes");
    
    // Generate files of different sizes
    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];
    
    for (name, size) in sizes {
        // Generate synthetic module with imports/exports
        let mut source = String::with_capacity(size);
        let import_line = "import foo from 'bar';\n";
        let export_line = "export const x = 1;\n";
        let comment_line = "// This is a comment line to fill space\n";
        
        while source.len() < size {
            source.push_str(import_line);
            source.push_str(export_line);
            source.push_str(comment_line);
        }
        
        let actual_size = source.len();
        group.throughput(Throughput::Bytes(actual_size as u64));
        group.bench_with_input(BenchmarkId::new(name, actual_size), &source, |b, s| {
            b.iter(|| parse(black_box(s)))
        });
    }
    
    group.finish();
}

fn bench_facade_vs_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("facade_vs_full");
    
    // Pure facade (only imports/exports)
    let facade_source = r#"
        import foo from 'bar';
        import { a, b } from 'module';
        export const x = 1;
        export function test() {}
    "#;
    
    group.bench_function("facade_mode", |b| {
        b.iter(|| parse(black_box(facade_source)))
    });
    
    // Full parse (mixed code)
    let full_source = r#"
        import foo from 'bar';
        const x = 1;
        function test() {
            return x + 1;
        }
        export { test };
    "#;
    
    group.bench_function("full_mode", |b| {
        b.iter(|| parse(black_box(full_source)))
    });
    
    group.finish();
}

fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");
    
    // Regex vs division
    let regex_source = r#"
        const regex = /import\s+from/;
        const division = 10 / 2;
    "#;
    group.bench_function("regex_vs_division", |b| {
        b.iter(|| parse(black_box(regex_source)))
    });
    
    // Template strings
    let template_source = r#"
        const url = `${import.meta.url}`;
        const nested = `outer ${`inner ${x}`}`;
    "#;
    group.bench_function("template_strings", |b| {
        b.iter(|| parse(black_box(template_source)))
    });
    
    // Comments
    let comment_source = r#"
        // Single line comment
        /* Multi-line
           comment */
        import foo from 'bar'; // inline comment
        /* import 'fake'; */ // commented import
    "#;
    group.bench_function("comments", |b| {
        b.iter(|| parse(black_box(comment_source)))
    });
    
    // String escapes
    let escape_source = r#"
        import foo from 'bar\n\t\r';
        import bar from "baz\u0041\x42";
    "#;
    group.bench_function("string_escapes", |b| {
        b.iter(|| parse(black_box(escape_source)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_simple_import,
    bench_multiple_imports,
    bench_dynamic_imports,
    bench_exports,
    bench_import_attributes,
    bench_complex_module,
    bench_real_world_files,
    bench_file_sizes,
    bench_facade_vs_full,
    bench_edge_cases
);
criterion_main!(benches);

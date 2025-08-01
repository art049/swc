extern crate swc_malloc;

use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use swc_common::{input::StringInput, FileName};
use swc_ecma_lexer::{lexer::Lexer, Syntax, TsSyntax};

fn bench_module(b: &mut Bencher, syntax: Syntax, src: &'static str) {
    let _ = ::testing::run_test(false, |cm, _| {
        let fm = cm.new_source_file(FileName::Anon.into(), src);

        b.iter(|| {
            let lexer = Lexer::new(syntax, Default::default(), StringInput::from(&*fm), None);
            for t in lexer {
                black_box(t);
            }
        });
        Ok(())
    });
}

fn bench_files(c: &mut Criterion) {
    c.bench_function("es/lexer/colors", |b| {
        // Copied from ratel-rust
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/colors.js"),
        )
    });

    c.bench_function("es/lexer/angular", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/angular-1.2.5.js"),
        )
    });

    c.bench_function("es/lexer/backbone", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/backbone-1.1.0.js"),
        )
    });

    c.bench_function("es/lexer/jquery", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/jquery-1.9.1.js"),
        )
    });

    c.bench_function("es/lexer/jquery mobile", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/jquery.mobile-1.4.2.js"),
        )
    });
    c.bench_function("es/lexer/mootools", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/mootools-1.4.5.js"),
        )
    });

    c.bench_function("es/lexer/underscore", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/underscore-1.5.2.js"),
        )
    });

    c.bench_function("es/lexer/three", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/three-0.138.3.js"),
        )
    });

    c.bench_function("es/lexer/yui", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/yui-3.12.0.js"),
        )
    });

    c.bench_function("es/lexer/cal-com", |b| {
        bench_module(
            b,
            Syntax::Typescript(TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            include_str!("../../swc_ecma_parser/benches/files/cal.com.tsx"),
        )
    });

    c.bench_function("es/lexer/typescript", |b| {
        bench_module(
            b,
            Default::default(),
            include_str!("../../swc_ecma_parser/benches/files/typescript.js"),
        )
    });
}

criterion_group!(benches, bench_files);
criterion_main!(benches);

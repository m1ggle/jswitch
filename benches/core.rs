use clap::Parser;
use criterion::{Criterion, criterion_group, criterion_main};
use jswitch::{Cli, config::Config, version::VersionResolver};

fn bench_cli_parsing(criterion: &mut Criterion) {
    criterion.bench_function("parse install command", |bencher| {
        bencher.iter(|| Cli::parse_from(["jswitch", "install", "1.8", "--source", "corretto"]));
    });

    criterion.bench_function("parse current command", |bencher| {
        bencher.iter(|| Cli::parse_from(["jswitch", "current"]));
    });
}

fn bench_version_resolution(criterion: &mut Criterion) {
    let mut config = Config::default();
    config.aliases.insert("lts".to_owned(), "21".to_owned());
    config.aliases.insert("legacy".to_owned(), "8".to_owned());
    let resolver = VersionResolver::new(config);

    criterion.bench_function("resolve alias", |bencher| {
        bencher.iter(|| resolver.resolve("lts"));
    });

    criterion.bench_function("resolve literal version", |bencher| {
        bencher.iter(|| resolver.resolve("17.0.10"));
    });
}

criterion_group!(benches, bench_cli_parsing, bench_version_resolution);
criterion_main!(benches);

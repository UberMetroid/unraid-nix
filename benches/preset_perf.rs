use criterion::{criterion_group, criterion_main, Criterion};

fn bench_render_presets(c: &mut Criterion) {
    c.bench_function("render_presets_dir_count", |b| {
        b.iter(|| {
            let dir = std::path::Path::new("presets");
            let _ = std::fs::read_dir(dir).map(|d| d.count()).unwrap_or(0);
        });
    });
}

fn bench_yaml_roundtrip(c: &mut Criterion) {
    use nix_helper::config::yaml::{parse_config, serialize_config};
    let sample = "version: \"0.5\"\nprocesses:\n  radarr:\n    command: \"nix run nixpkgs#radarr\"\n    availability:\n      restart: \"always\"\n";
    c.bench_function("yaml_roundtrip", |b| {
        b.iter(|| {
            let cfg = parse_config(sample).unwrap();
            let _ = serialize_config(&cfg);
        });
    });
}

criterion_group!(benches, bench_render_presets, bench_yaml_roundtrip);
criterion_main!(benches);
use look_engine::QueryEngine;
use look_storage::SqliteStore;
use std::env;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct QueryBenchStats {
    query: &'static str,
    iterations: usize,
    p50_us: u128,
    p95_us: u128,
    avg_us: u128,
    min_us: u128,
    max_us: u128,
}

fn main() {
    let db_path = default_db_path();
    let started = Instant::now();
    if let Err(err) = QueryEngine::bootstrap_sqlite(&db_path) {
        eprintln!("index bootstrap failed: {err}");
        std::process::exit(1);
    }
    let index_elapsed = started.elapsed();

    let candidate_count =
        match SqliteStore::open(&db_path).and_then(|store| store.load_candidates(None)) {
            Ok(candidates) => candidates.len(),
            Err(err) => {
                eprintln!("failed to count candidates: {err}");
                std::process::exit(1);
            }
        };

    let engine = match QueryEngine::from_sqlite(&db_path) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize engine: {err}");
            std::process::exit(1);
        }
    };

    let query_cases = [
        "",
        "sa",
        "net",
        "doc",
        "visual",
        "privacy security",
        "a\"safari",
        "f\"note",
        "d\"down",
        "r\"^visual.*",
    ];

    let mut query_stats = Vec::new();
    for query in query_cases {
        query_stats.push(bench_query(&engine, query, 40, 300));
    }

    println!("# look benchmark");
    println!("db_path={}", db_path.display());
    println!("candidate_count={candidate_count}");
    println!(
        "index_elapsed_ms={} index_throughput_per_sec={:.2}",
        index_elapsed.as_millis(),
        throughput_per_second(candidate_count, index_elapsed)
    );
    println!("query,iterations,p50_us,p95_us,avg_us,min_us,max_us");
    for stat in query_stats {
        println!(
            "{},{},{},{},{},{},{}",
            stat.query,
            stat.iterations,
            stat.p50_us,
            stat.p95_us,
            stat.avg_us,
            stat.min_us,
            stat.max_us
        );
    }
}

fn bench_query(
    engine: &QueryEngine,
    query: &'static str,
    limit: usize,
    iterations: usize,
) -> QueryBenchStats {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let started = Instant::now();
        let results = engine.search(query, limit);
        black_box(results.len());
        samples.push(started.elapsed().as_micros());
    }
    samples.sort_unstable();

    let p50_us = percentile(&samples, 50);
    let p95_us = percentile(&samples, 95);
    let min_us = *samples.first().unwrap_or(&0);
    let max_us = *samples.last().unwrap_or(&0);
    let total_us: u128 = samples.iter().copied().sum();
    let avg_us = if samples.is_empty() {
        0
    } else {
        total_us / samples.len() as u128
    };

    QueryBenchStats {
        query,
        iterations,
        p50_us,
        p95_us,
        avg_us,
        min_us,
        max_us,
    }
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    let rank = ((samples.len() - 1) * p) / 100;
    samples[rank]
}

fn throughput_per_second(count: usize, duration: Duration) -> f64 {
    let secs = duration.as_secs_f64();
    if secs <= f64::EPSILON {
        return 0.0;
    }
    count as f64 / secs
}

fn default_db_path() -> PathBuf {
    if let Ok(custom) = env::var("LOOK_DB_PATH")
        && !custom.trim().is_empty()
    {
        return PathBuf::from(custom);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("look")
        .join("look.db")
}

//! Isolated 50-resident storage and command benchmark. Never contacts a provider.
use city_core::*;
use std::{path::PathBuf, time::Instant};
fn main() -> Result<()> {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Provide a fresh disposable directory"),
    );
    if path.exists() {
        return Err(CityError::new(
            "already_exists",
            "Use a fresh benchmark directory.",
        ));
    }
    let rt = Runtime::open(&path, true)?;
    let mut world = rt.snapshot();
    drop(rt);
    let mayor = world.residents[0].clone();
    for n in 1..50 {
        let mut resident = mayor.clone();
        resident.id = id();
        resident.name = format!("Fixture {n}");
        resident.role = "Analyst".into();
        world.roads.push(Road {
            id: id(),
            from: mayor.id.clone(),
            to: resident.id.clone(),
            enabled: true,
            presentation: "auto".into(),
            rule_version: 1,
            revision: 0,
        });
        world.residents.push(resident);
    }
    {
        let mut store = store::Store::open(&path, true)?;
        store.persist(world, None)?;
    }
    let rt = Runtime::open(&path, true)?;
    let residents = rt.snapshot().residents;
    let mut elapsed = vec![];
    let start = Instant::now();
    for n in 0..500 {
        let tick = Instant::now();
        rt.command(Command {
            schema_version: 1,
            id: id(),
            expected_revision: None,
            action: Action::Rename {
                resident_id: residents[n % 50].id.clone(),
                name: format!("Fixture {} revision {n}", n % 50),
            },
        })?;
        elapsed.push(tick.elapsed().as_secs_f64() * 1000.0);
    }
    elapsed.sort_by(f64::total_cmp);
    let snapshot = serde_json::to_vec(&rt.view_snapshot())?;
    println!("residents=50 commands=500 elapsed_seconds={:.3} median_ms={:.3} p95_ms={:.3} max_ms={:.3} snapshot_bytes={} inference_calls={} database_bytes={}",start.elapsed().as_secs_f64(),elapsed[250],elapsed[475],elapsed[499],snapshot.len(),rt.snapshot().runs.len(),std::fs::metadata(path.join("city.sqlite3"))?.len());
    Ok(())
}

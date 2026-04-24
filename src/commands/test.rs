use anyhow::Result;
use lukuid_sdk::LukuidSdk;
use console::style;

pub fn run_test(json: bool) -> Result<i32> {
    let results = LukuidSdk::self_test();

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(if results.iter().all(|r| r.passed) { 0 } else { 1 });
    }

    println!("{0: <15} | {1: <10} | {2: <6} | {3}", "Algorithm", "Operation", "Result", "KAT ID");
    println!("{0:-<15}-|-{0:-<10}-|-{0:-<6}-|-{0:-<30}", "");

    let mut all_passed = true;
    for result in &results {
        let status = if result.passed {
            style("PASS").green().bold()
        } else {
            all_passed = false;
            style("FAIL").red().bold()
        };

        println!("{0: <15} | {1: <10} | {2: <6} | {3}", result.alg, result.operation, status, result.id);
    }
    println!();

    if all_passed {
        Ok(0)
    } else {
        Ok(1)
    }
}

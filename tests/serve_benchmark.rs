// SPDX-License-Identifier: MPL-2.0
//! Reproducible local profiler: set `TEAMY_BEND_BENCH_MODEL`,
//! `TEAMY_BEND_BENCH_TRANSCRIPT`, `TEAMY_BEND_BENCH_BIN`, then run this ignored test
//! with --release --ignored --nocapture. Transcript lines begin with `> `.
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use teamy_bend::kernel::check_book;
use teamy_bend::protocol::DataValue;
use teamy_bend::protocol::Request;
use teamy_bend::protocol::Response;
use teamy_bend::protocol::arguments;
use teamy_bend::syntax::load;

#[derive(Default)]
struct Timings {
    decode: Duration,
    arguments: Duration,
    evaluate: Duration,
    result: Duration,
    encode: Duration,
}

fn configured(name: &str) -> PathBuf {
    std::env::var_os(name)
        .unwrap_or_else(|| panic!("set {name} for the local benchmark"))
        .into()
}

fn decode(line: &str) -> Request {
    facet_json::from_str(line).expect("recorded request decodes")
}

fn measure() {
    let model = configured("TEAMY_BEND_BENCH_MODEL");
    let transcript = fs::read_to_string(configured("TEAMY_BEND_BENCH_TRANSCRIPT")).unwrap();
    let lines: Vec<_> = transcript
        .lines()
        .filter_map(|line| line.strip_prefix("> "))
        .collect();
    assert!(!lines.is_empty());
    let checked = check_book(&load(&model).unwrap()).unwrap();
    let mut timings = Timings::default();
    let repetitions = 20;
    let started = Instant::now();
    for _ in 0..repetitions {
        for line in &lines {
            let at = Instant::now();
            let request = decode(line);
            timings.decode += at.elapsed();
            let at = Instant::now();
            let args = arguments(&request.args).unwrap();
            timings.arguments += at.elapsed();
            let at = Instant::now();
            let value = checked.evaluate_data(&request.entry, &args).unwrap();
            timings.evaluate += at.elapsed();
            let at = Instant::now();
            let response = Response {
                id: Some(request.id),
                value: Some(DataValue::from_term(&value).unwrap()),
                error: None,
            };
            timings.result += at.elapsed();
            let at = Instant::now();
            std::hint::black_box(facet_json::to_string(&response).unwrap());
            timings.encode += at.elapsed();
        }
    }
    let local = started.elapsed();
    let mut requests = Vec::new();
    for _ in 0..repetitions {
        for line in &lines {
            let mut request = decode(line);
            request.id = u64::try_from(requests.len()).unwrap() + 1;
            requests.push(facet_json::to_string(&request).unwrap());
        }
    }
    let mut child = Command::new(configured("TEAMY_BEND_BENCH_BIN"))
        .arg("serve")
        .arg(&model)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut response = String::new();
    // A separate first request absorbs process startup and complete source checking.
    let mut warmup = decode(lines[0]);
    warmup.id = 0;
    writeln!(input, "{}", facet_json::to_string(&warmup).unwrap()).unwrap();
    input.flush().unwrap();
    output.read_line(&mut response).unwrap();
    let started = Instant::now();
    for request in &requests {
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
        response.clear();
        assert_ne!(output.read_line(&mut response).unwrap(), 0);
        assert!(response.contains("\"error\":null"), "{response}");
    }
    let transport = started.elapsed();
    drop(input);
    assert!(child.wait().unwrap().success());
    eprintln!(
        "requests={} decode_us={} arguments_us={} checked_evaluate_us={} result_us={} encode_us={} local_wall_us={} roundtrip_wall_us={}",
        requests.len(),
        timings.decode.as_micros(),
        timings.arguments.as_micros(),
        timings.evaluate.as_micros(),
        timings.result.as_micros(),
        timings.encode.as_micros(),
        local.as_micros(),
        transport.as_micros()
    );
}

#[test]
#[ignore = "local profiler requires explicit model, transcript and backend executable"]
fn profile_persistent_constructor_protocol() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(measure)
        .unwrap()
        .join()
        .unwrap();
}

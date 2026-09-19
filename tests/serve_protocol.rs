// SPDX-License-Identifier: MPL-2.0
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use teamy_bend::protocol::DataValue;
use teamy_bend::protocol::Request;
use teamy_bend::protocol::Response;

fn natural(n: usize) -> DataValue {
    let mut value = DataValue {
        constructor: "Zero".to_owned(),
        fields: vec![],
    };
    for _ in 0..n {
        value = DataValue {
            constructor: "Succ".to_owned(),
            fields: vec![value],
        };
    }
    value
}

fn run(lines: &[String]) -> Vec<Response> {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/laws.bend");
    let mut child = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .arg("serve")
        .arg(source)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start persistent CLI");
    let mut stdin = child.stdin.take().expect("piped stdin");
    for line in lines {
        writeln!(stdin, "{line}").expect("write request");
    }
    drop(stdin);
    let output = child.wait_with_output().expect("read responses");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The reflected Rust decoder, like the server, needs a bounded larger
    // stack for tagged Nat64; the wire itself is ordinary JSON.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            String::from_utf8(output.stdout)
                .expect("UTF-8 output")
                .lines()
                .map(|line| facet_json::from_str(line).expect("typed response"))
                .collect()
        })
        .expect("start response decoder")
        .join()
        .expect("response decoder")
}

fn request(id: u64, value: DataValue) -> String {
    facet_json::to_string(&Request {
        id,
        entry: "identity".to_owned(),
        args: vec![value],
    })
    .expect("encode request")
}

#[test]
fn one_session_handles_multiple_typed_calls_including_deep_naturals() {
    // Nat64 requires more than128 JSON object/array nesting levels. It is a
    // normal Poche score, so transport must handle it inside its explicit caps.
    let responses = run(&[
        request(1, natural(0)),
        request(2, natural(64)),
        request(3, natural(3)),
        request(4, natural(96)),
    ]);
    assert_eq!(responses.len(), 4);
    for (response, expected) in responses.iter().zip([0, 64, 3, 96]) {
        assert!(response.error.is_none(), "{:?}", response.error);
        assert_eq!(response.value.as_ref(), Some(&natural(expected)));
    }
    assert_eq!(responses[2].id, Some(3));
}

#[test]
fn constructor_node_budget_is_shared_and_failure_does_not_poison_session() {
    let excessive = DataValue {
        constructor: "Unknown".into(),
        fields: vec![natural(0); teamy_bend::protocol::MAX_DATA_NODES],
    };
    let half = DataValue {
        constructor: "Unknown".into(),
        fields: vec![natural(0); teamy_bend::protocol::MAX_DATA_NODES / 2],
    };
    let combined = facet_json::to_string(&Request {
        id: 2,
        entry: "identity".into(),
        args: vec![half.clone(), half],
    })
    .expect("shared budget request");
    let responses = run(&[request(1, excessive), combined, request(3, natural(1))]);
    assert!(
        responses[0]
            .error
            .as_ref()
            .is_some_and(|error| error.contains("node budget"))
    );
    assert!(
        responses[1]
            .error
            .as_ref()
            .is_some_and(|error| error.contains("node budget"))
    );
    assert_eq!(responses[2].value, Some(natural(1)));
}

#[test]
fn malformed_and_ill_typed_requests_do_not_poison_later_calls() {
    let responses = run(&[
        "{broken".to_owned(),
        request(
            7,
            DataValue {
                constructor: "Unknown".to_owned(),
                fields: vec![],
            },
        ),
        request(8, natural(2)),
        request(8, natural(3)),
        request(9, natural(4)),
    ]);
    assert_eq!(responses.len(), 5);
    assert_eq!(responses[0].id, None);
    assert!(responses[0].error.is_some());
    assert!(responses[1].error.is_some());
    assert_eq!(responses[2].value, Some(natural(2)));
    assert!(
        responses[3]
            .error
            .as_ref()
            .is_some_and(|error| error.contains("strictly increase"))
    );
    assert_eq!(responses[4].value, Some(natural(4)));
}

#[test]
fn extra_constructor_fields_and_non_data_results_are_rejected() {
    let responses = run(&[
        request(1, DataValue { constructor: "Zero".to_owned(), fields: vec![natural(0)] }),
        "{\"id\":2,\"entry\":\"identity_law\",\"args\":[{\"constructor\":\"Zero\",\"fields\":[]}] }".to_owned(),
    ]);
    assert!(
        responses
            .iter()
            .all(|response| response.error.is_some() && response.value.is_none())
    );
}

#[test]
fn excessive_constructor_depth_is_an_error_response() {
    let responses = run(&[request(1, natural(97)), request(2, natural(1))]);
    assert!(
        responses[0]
            .error
            .as_ref()
            .is_some_and(|error| error.contains("nesting"))
    );
    assert_eq!(responses[1].value, Some(natural(1)));
}

#[test]
fn trailing_text_is_rejected_without_consuming_the_request_id() {
    let responses = run(&[
        format!("{} trailing", request(1, natural(0))),
        request(1, natural(2)),
    ]);
    assert_eq!(responses[0].id, None);
    assert!(responses[0].value.is_none());
    assert!(responses[0].error.is_some());
    assert_eq!(responses[1].value, Some(natural(2)));
}

#[test]
fn two_json_requests_on_one_line_are_rejected_but_whitespace_is_valid() {
    let responses = run(&[
        format!("{} {}", request(10, natural(0)), request(99, natural(1))),
        format!("  {} \t ", request(10, natural(2))),
    ]);
    assert_eq!(responses[0].id, None);
    assert!(responses[0].error.is_some());
    assert_eq!(responses[1].id, Some(10));
    assert_eq!(responses[1].value, Some(natural(2)));
}

#[test]
fn unknown_fields_at_request_and_nested_constructor_levels_are_rejected() {
    let responses=run(&[
        r#"{"id":1,"entry":"identity","args":[{"constructor":"Zero","fields":[]}],"unused":true}"#.to_owned(),
        r#"{"id":1,"entry":"identity","args":[{"constructor":"Succ","fields":[{"constructor":"Zero","fields":[],"expression":"{==}"}]}]}"#.to_owned(),
        request(1,natural(1)),
    ]);
    for response in &responses[..2] {
        assert_eq!(response.id, None);
        assert!(response.value.is_none());
        assert!(
            response
                .error
                .as_ref()
                .is_some_and(|error| error.contains("unknown field"))
        );
    }
    assert_eq!(responses[2].value, Some(natural(1)));
}

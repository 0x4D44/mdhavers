#![cfg(all(coverage, feature = "native"))]

use mdhavers::{parse, Interpreter, Value};

fn interpret_ok(source: &str) -> Value {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("interpret failed for:\n{source}\nerr={e:?}"))
}

#[test]
fn dns_srv_uses_stubbed_lookup_to_cover_non_test_mappers_for_coverage() {
    use trust_dns_resolver::lookup::Lookup;
    use trust_dns_resolver::proto::op::Query;
    use trust_dns_resolver::proto::rr::rdata::{A, SRV};
    use trust_dns_resolver::proto::rr::{Name, RData, RecordType};

    let target = Name::from_ascii("example.com.").expect("target");
    let srv = SRV::new(10, 5, 443, target);
    let name = Name::from_ascii("_sip._udp.example.com.").expect("query name");
    let lookup = Lookup::from_rdata(Query::query(name, RecordType::SRV), RData::SRV(srv));
    mdhavers::interpreter::dns_set_next_srv_lookup_for_coverage(Ok(lookup));

    let value = interpret_ok(
        r#"
ken r = dns_srv("_sip._udp", "example.com")
r["value"][0]["port"]
"#,
    );
    assert_eq!(value, Value::Integer(443));

    // Cover the non-SRV filter path without touching the network.
    let name = Name::from_ascii("_sip._udp.example.com.").expect("query name");
    let lookup = Lookup::from_rdata(
        Query::query(name, RecordType::SRV),
        RData::A(A::new(127, 0, 0, 1)),
    );
    mdhavers::interpreter::dns_set_next_srv_lookup_for_coverage(Ok(lookup));

    let value = interpret_ok(
        r#"
ken r = dns_srv("_sip._udp", "example.com")
len(r["value"])
"#,
    );
    assert_eq!(value, Value::Integer(0));
}

#[test]
fn dns_naptr_uses_stubbed_lookup_to_cover_non_test_mappers_for_coverage() {
    use trust_dns_resolver::lookup::Lookup;
    use trust_dns_resolver::proto::op::Query;
    use trust_dns_resolver::proto::rr::rdata::{A, NAPTR};
    use trust_dns_resolver::proto::rr::{Name, RData, RecordType};

    let replacement = Name::from_ascii("example.com.").expect("replacement");
    let naptr = NAPTR::new(
        100,
        10,
        b"U".to_vec().into_boxed_slice(),
        b"SIP+D2U".to_vec().into_boxed_slice(),
        b"!^.*$!sip:info@example.com!".to_vec().into_boxed_slice(),
        replacement,
    );
    let name = Name::from_ascii("example.com.").expect("query name");
    let lookup = Lookup::from_rdata(Query::query(name, RecordType::NAPTR), RData::NAPTR(naptr));
    mdhavers::interpreter::dns_set_next_naptr_lookup_for_coverage(Ok(lookup));

    let value = interpret_ok(
        r#"
ken r = dns_naptr("example.com")
r["value"][0]["service"]
"#,
    );
    assert_eq!(value, Value::String("SIP+D2U".to_string()));

    // Cover the non-NAPTR filter path without touching the network.
    let name = Name::from_ascii("example.com.").expect("query name");
    let lookup = Lookup::from_rdata(
        Query::query(name, RecordType::NAPTR),
        RData::A(A::new(127, 0, 0, 1)),
    );
    mdhavers::interpreter::dns_set_next_naptr_lookup_for_coverage(Ok(lookup));

    let value = interpret_ok(
        r#"
ken r = dns_naptr("example.com")
len(r["value"])
"#,
    );
    assert_eq!(value, Value::Integer(0));
}

#[test]
fn dns_srv_covers_make_resolver_init_failure_branches_for_coverage() {
    mdhavers::interpreter::dns_fail_next_resolver_for_coverage();
    let value = interpret_ok(
        r#"
ken r = dns_srv("_sip._udp", "example.com")
r["error"]
"#,
    );
    let Value::String(msg) = value else {
        panic!("expected error string, got {value:?}");
    };
    assert!(
        msg.contains("dns_srv() DNS resolver init failed: injected"),
        "unexpected error: {msg}"
    );
}

#[test]
fn dns_srv_covers_system_conf_fallback_and_new_error_branches_for_coverage() {
    mdhavers::interpreter::dns_force_next_system_conf_error_for_coverage();
    mdhavers::interpreter::dns_force_next_new_error_for_coverage();
    let value = interpret_ok(
        r#"
ken r = dns_srv("_sip._udp", "example.com")
r["error"]
"#,
    );
    let Value::String(msg) = value else {
        panic!("expected error string, got {value:?}");
    };
    assert!(
        msg.contains("dns_srv() DNS resolver init failed: injected new error"),
        "unexpected error: {msg}"
    );
}

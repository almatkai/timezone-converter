use serde_json::Value;
use timezone_converter_v2::time;
#[test]
fn original_locked_luxon_oracle() {
    let o: Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let mut errors = Vec::new();
    for c in o["cases"].as_array().unwrap() {
        let r = serde_json::to_value(
            time::row(c["ms"].as_i64().unwrap(), c["zone"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        for key in ["ff", "short", "offsetSeconds"] {
            if r[key] != c[key] {
                errors.push(format!(
                    "{} {} {key}: {} != {}",
                    c["zone"], c["ms"], r[key], c[key]
                ))
            }
        }
    }
    assert!(
        errors.is_empty(),
        "{} mismatches:\n{}",
        errors.len(),
        errors.join("\n")
    );
    for c in o["custom"].as_array().unwrap() {
        assert_eq!(
            time::custom(
                c["pickerMs"].as_i64().unwrap(),
                c["zone"].as_str().unwrap(),
                c["local"].as_str().unwrap()
            )
            .unwrap(),
            c["ms"].as_i64().unwrap(),
            "{c}"
        );
    }
    assert_eq!(time::catalog().len(), 419);
}
#[test]
fn picker_gap_fold_fractional_and_bounds() {
    assert_eq!(
        time::picker("2024-11-03T01:30:00-05:00", "America/New_York").unwrap(),
        1730615400000
    );
    assert_eq!(
        time::picker("2024-03-10T02:30:00.000", "America/New_York").unwrap(),
        1710055800000
    );
    assert_eq!(
        time::picker("2024-11-03T01:30:00.000", "America/New_York").unwrap(),
        1730611800000
    );
    assert!(time::parse_wall("2024-01-01T00:00:60").is_err());
    assert!(time::parse_wall("2024-02-30T00:00").is_err());
    assert!(time::instant(i64::MAX).is_err());
    assert!(time::row(-2208988800001, "Asia/Katmandu").is_ok());
    assert!(time::row(4133980800000, "UTC").is_ok());
    assert_eq!(
        time::gmt(1705322096789, "Asia/Katmandu").unwrap(),
        "GMT+5.75"
    );
}

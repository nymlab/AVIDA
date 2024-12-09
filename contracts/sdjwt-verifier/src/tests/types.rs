#[test]
fn test_presentation_req_from_json() {
    let data = r#"[
          {
            "attribute": "age",
            "criterion": {
              "number": { "value": 30, "operator": "greater_than" }
            }
          },
          {
	          "attribute": "active",
	          "criterion": { "boolean": true }
	        }
        ]"#
    .as_bytes();

    let e = serde_json::from_slice::<Vec<crate::types::ReqAttr>>(data);
    assert!(e.is_ok());
}

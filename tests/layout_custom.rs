use hmux2::src::layout_custom::{
    parse_layout_description, serialize_layout_description, LayoutDescription, INT_MAX,
    PANE_MAXIMUM, WINDOW_MAXIMUM,
};
use hmux2::src::shared::layout::{LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE};

fn legacy_checksum(body: &[u8]) -> u16 {
    let mut checksum = 0u16;
    for byte in body {
        checksum = (checksum >> 1) | ((checksum & 1) << 15);
        checksum = checksum.wrapping_add(*byte as u16);
    }
    checksum
}

fn legacy_layout(body: &str) -> Vec<u8> {
    let body = body.as_bytes();
    let mut result = format!("{:04x},", legacy_checksum(body)).into_bytes();
    result.extend_from_slice(body);
    result
}

fn v2_nested_layout() -> Vec<u8> {
    br#"{"V":2,"L":{"t":"h","w":79,"h":23,"x":0,"y":0,"c":[{"t":"v","w":39,"h":23,"x":0,"y":0,"c":[{"t":"p","w":39,"h":11,"x":0,"y":0,"i":2,"l":4,"I":"%10"},{"t":"p","w":39,"h":11,"x":0,"y":12,"i":0,"a":true,"I":"%11"}]},{"t":"p","w":39,"h":23,"x":40,"y":0,"i":1,"z":3,"I":"%12"}]}}"#.to_vec()
}

#[test]
fn parses_nested_v2_description_and_retains_geometry_and_identifiers() {
    let description = parse_layout_description(&v2_nested_layout()).unwrap();

    assert_eq!(description.version, 2);
    assert_eq!(description.root.type_0, LAYOUT_LEFTRIGHT);
    assert_eq!(description.root.geometry.sx, 79);
    assert_eq!(description.root.children.len(), 2);
    assert_eq!(description.root.children[0].type_0, LAYOUT_TOPBOTTOM);
    assert_eq!(description.root.children[0].children.len(), 2);

    let first = description.root.children[0].children[0]
        .pane
        .as_ref()
        .unwrap();
    assert_eq!(first.index, Some(2));
    assert_eq!(first.identifier.as_deref(), Some(b"%10".as_slice()));
    assert_eq!(first.last, Some(4));

    let floating = description.root.children[1].pane.as_ref().unwrap();
    assert_eq!(description.root.children[1].type_0, LAYOUT_WINDOWPANE);
    assert_eq!(floating.zindex, Some(3));
    assert_eq!(floating.identifier.as_deref(), Some(b"%12".as_slice()));
}

#[test]
fn v2_serialization_round_trips_the_detached_description() {
    let original = parse_layout_description(&v2_nested_layout()).unwrap();
    let serialized = serialize_layout_description(&original).unwrap();
    let reparsed = parse_layout_description(&serialized).unwrap();

    assert_eq!(reparsed, original);
}

#[test]
fn v2_round_trip_preserves_the_parser_string_bytes() {
    let original = parse_layout_description(
        br#"{"V":2,"L":{"t":"p","w":79,"h":23,"x":0,"y":0,"i":0,"I":"foo\u1234"}}"#,
    )
    .unwrap();
    assert_eq!(
        original.root.pane.as_ref().unwrap().identifier.as_deref(),
        Some(b"foo\\u1234".as_slice())
    );

    let serialized = serialize_layout_description(&original).unwrap();
    assert_eq!(parse_layout_description(&serialized).unwrap(), original);
}

#[test]
fn v2_serializer_grows_for_long_identifiers() {
    let identifier = "x".repeat(4096);
    let source = format!(
        r#"{{"V":2,"L":{{"t":"p","w":79,"h":23,"x":0,"y":0,"i":0,"I":"{}"}}}}"#,
        identifier
    );
    let description = parse_layout_description(source.as_bytes()).unwrap();
    let serialized = serialize_layout_description(&description).unwrap();

    assert_eq!(serialized.as_ref(), source.as_bytes());
}

#[test]
fn legacy_serialization_round_trips_nested_pane_ids() {
    let original =
        parse_layout_description(&legacy_layout("79x23,0,0{39x23,0,0,7,39x23,40,0,8}")).unwrap();
    assert_eq!(original.version, 1);
    assert_eq!(original.root.type_0, LAYOUT_LEFTRIGHT);
    assert_eq!(original.root.children[0].pane.as_ref().unwrap().id, Some(7));
    assert_eq!(original.root.children[1].pane.as_ref().unwrap().id, Some(8));

    let serialized = serialize_layout_description(&original).unwrap();
    assert_eq!(parse_layout_description(&serialized).unwrap(), original);
}

#[test]
fn malformed_and_truncated_layouts_keep_their_diagnostics() {
    let malformed_header = parse_layout_description(b"not-a-layout").unwrap_err();
    assert_eq!(malformed_header.to_string(), "malformed layout header");

    let malformed_json = parse_layout_description(br#"{"V":2,"L":{"t":"p"}}"#).unwrap_err();
    assert!(malformed_json.to_string().contains("key \"w\" not found"));

    let json = v2_nested_layout();
    let truncated_json = parse_layout_description(&json[..json.len() - 2]).unwrap_err();
    assert!(!truncated_json.to_string().is_empty());

    let truncated_legacy = legacy_layout("79x23,0,0{39x23,0,0");
    let truncated_legacy = parse_layout_description(&truncated_legacy).unwrap_err();
    assert_eq!(truncated_legacy.to_string(), "invalid layout");

    let trailing = legacy_layout("79x23,0,0,7,");
    let trailing = parse_layout_description(&trailing).unwrap_err();
    assert_eq!(trailing.to_string(), "trailing data");
}

#[test]
fn legacy_checksum_rejection_is_reported_before_parsing() {
    let mut layout = legacy_layout("79x23,0,0,7");
    let replacement = if layout[0] == b'0' { b'1' } else { b'0' };
    layout[0] = replacement;

    assert_eq!(
        parse_layout_description(&layout).unwrap_err().to_string(),
        "invalid layout checksum"
    );
}

#[test]
fn geometry_and_metadata_limits_keep_their_diagnostics() {
    let width = format!(
        "{{\"V\":2,\"L\":{{\"t\":\"p\",\"w\":{},\"h\":23,\"x\":0,\"y\":0,\"i\":0}}}}",
        PANE_MAXIMUM + 1
    );
    assert_eq!(
        parse_layout_description(width.as_bytes())
            .unwrap_err()
            .to_string(),
        format!("invalid width {}", PANE_MAXIMUM + 1)
    );

    let x_offset = format!(
        "{{\"V\":2,\"L\":{{\"t\":\"p\",\"w\":79,\"h\":23,\"x\":{},\"y\":0,\"i\":0}}}}",
        WINDOW_MAXIMUM + 1
    );
    assert_eq!(
        parse_layout_description(x_offset.as_bytes())
            .unwrap_err()
            .to_string(),
        format!("invalid x-offset {}", WINDOW_MAXIMUM + 1)
    );

    let negative_index = br#"{"V":2,"L":{"t":"p","w":79,"h":23,"x":0,"y":0,"i":-1}}"#;
    assert_eq!(
        parse_layout_description(negative_index)
            .unwrap_err()
            .to_string(),
        "invalid index -1"
    );

    let invalid_zindex = format!(
        "{{\"V\":2,\"L\":{{\"t\":\"p\",\"w\":79,\"h\":23,\"x\":0,\"y\":0,\"i\":0,\"z\":{}}}}}",
        INT_MAX
    );
    assert_eq!(
        parse_layout_description(invalid_zindex.as_bytes())
            .unwrap_err()
            .to_string(),
        format!("invalid floating zindex {}", INT_MAX)
    );
}

#[test]
fn legacy_depth_limit_rejects_truncatedly_deep_layouts() {
    let mut body = String::new();
    for _ in 0..1002 {
        body.push_str("1x1,0,0{");
    }
    body.push_str("1x1,0,0");
    for _ in 0..1002 {
        body.push('}');
    }

    assert_eq!(
        parse_layout_description(&legacy_layout(&body))
            .unwrap_err()
            .to_string(),
        "invalid layout"
    );
}

#[test]
fn v2_validation_rejects_pane_metadata_conflicts_before_application() {
    let duplicate_index = br#"{"V":2,"L":{"t":"h","w":79,"h":23,"x":0,"y":0,"c":[{"t":"p","w":39,"h":23,"x":0,"y":0,"i":0},{"t":"p","w":39,"h":23,"x":40,"y":0,"i":0}]}}"#;
    assert_eq!(
        parse_layout_description(duplicate_index)
            .unwrap_err()
            .to_string(),
        "duplicate pane index"
    );

    let duplicate_active = br#"{"V":2,"L":{"t":"h","w":79,"h":23,"x":0,"y":0,"c":[{"t":"p","w":39,"h":23,"x":0,"y":0,"i":0,"a":true},{"t":"p","w":39,"h":23,"x":40,"y":0,"i":1,"a":true}]}}"#;
    assert_eq!(
        parse_layout_description(duplicate_active)
            .unwrap_err()
            .to_string(),
        "more than one active pane"
    );
}

#[test]
fn an_explicit_inactive_flag_keeps_the_legacy_last_field_ignored() {
    let layout =
        br#"{"V":2,"L":{"t":"p","w":79,"h":23,"x":0,"y":0,"i":0,"a":false,"l":"ignored"}}"#;
    let description = parse_layout_description(layout).unwrap();
    let pane = description.root.pane.as_ref().unwrap();

    assert!(!pane.active);
    assert_eq!(pane.last, None);
}

#[test]
fn parser_rejects_embedded_nul_without_touching_a_c_string() {
    let error = parse_layout_description(b"{}\0").unwrap_err();
    assert_eq!(error.to_string(), "embedded NUL");
}

#[test]
fn descriptions_are_owned_values() {
    let description: LayoutDescription = parse_layout_description(&v2_nested_layout()).unwrap();
    let copy = description.clone();
    assert_eq!(copy, description);
}

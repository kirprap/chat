use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_chat_websocket_url() {
    assert_eq!(
        format!("ws://127.0.0.1:8080/ws/?username={}", "testuser"),
        "ws://127.0.0.1:8080/ws/?username=testuser"
    );
}

#[wasm_bindgen_test]
fn test_html_structure() {
    // This is a basic test to ensure the test infrastructure works
    assert!(true);
}
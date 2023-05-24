use super::Opt;
use super::prepare_collab_options;

/// On wasm, some options are hardcoded, other are read from URL
pub fn options_from_url() -> Opt {
    let request = decode_request(web_sys::window().unwrap());
    let (collab_url, collab_id) = parse_request(request);
    let collab = prepare_collab_options(collab_url, collab_id);

    Opt {
        collab,
        show_debug_pane: false,
        verbose: false,
        ui: true,
    }
}

fn decode_request(window: web_sys::Window) -> Option<std::string::String> {
    Some(
        window
            .location()
            .search()
            .ok()?
            .trim_start_matches('?')
            .to_owned(),
    )
}

pub fn setup_log() {
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::ERROR)
            .build(),
    );
}

fn parse_request(request: Option<String>) -> (Option<String>, Option<u16>) {
    let Some(request) = request else { return (None, None) };

    if request.is_empty() {
        return (None, None);
    };

    let mut collab_url = None;
    let mut collab_id = None;
    for param in request.split('&') {
        let mut param = param.split('=');
        let Some(key) = param.next() else { return (None, None) };
        let Some(value) = param.next() else { return (None, None) };
        match key {
            "collab-url" => collab_url = Some(value.to_owned()),
            "collab-id" => collab_id = value.parse().ok(),
            _ => (),
        }
    }
    (collab_url, collab_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_request() {
        let request = "collab-url=ws://127.0.0.1:3536/lavagna&collab-id=6";
        let (collab_url, collab_id) = parse_request(Some(request.to_string()));
        assert_eq!(collab_url, Some("ws://127.0.0.1:3536/lavagna".to_string()));
        assert_eq!(collab_id, Some(6u16));
    }
}

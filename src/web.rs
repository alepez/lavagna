use crate::collab::CollabPluginOpt as CollabOpt;
use crate::Opt;

/// On wasm, some options are hardcoded, other are read from URL
pub fn options_from_url() -> Opt {
    web_sys::window()
        .and_then(decode_request)
        .map(|request| Opt::from(&request))
        .unwrap_or_default()
}

struct Request(String);

fn decode_request(window: web_sys::Window) -> Option<Request> {
    Some(Request(
        window
            .location()
            .search()
            .ok()?
            .trim_start_matches('?')
            .to_owned(),
    ))
}

impl TryFrom<&Request> for CollabOpt {
    type Error = ();

    fn try_from(request: &Request) -> Result<Self, ()> {
        let mut url: Option<String> = None;
        let mut collab_id: Option<u16> = None;

        for param in request.0.split('&') {
            let mut param = param.split('=');
            let Some(key) = param.next() else { break };
            let Some(value) = param.next() else { break };
            match key {
                "collab-url" => url = Some(value.to_owned()),
                "collab-id" => collab_id = value.parse().ok(),
                _ => (),
            }
        }

        if let Some(url) = url {
            // If collab-url is set, then collab-id must be set too. Randomize it if not.
            let collab_id = collab_id.unwrap_or_else(|| rand::random());
            Ok(CollabOpt { url, collab_id })
        } else {
            Err(())
        }
    }
}

impl From<&Request> for Opt {
    fn from(request: &Request) -> Self {
        let mut opt = Self::default();

        opt.collab = CollabOpt::try_from(request).ok();

        for param in request.0.split('&') {
            let mut param = param.split('=');
            let Some(key) = param.next() else { break };
            let Some(v) = param.next() else { break };
            match key {
                "v" | "verbose" => opt.verbose = v.parse().unwrap_or_default(),
                "dbg" | "show-debug-pane" => opt.show_debug_pane = v.parse().unwrap_or_default(),
                "ui" => opt.ui = v.parse().unwrap_or_default(),
                _ => (),
            }
        }

        opt
    }
}

pub fn setup_log() {
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::ERROR)
            .build(),
    );
}

use req1_server::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    print!(
        "{}",
        ApiDoc::openapi()
            .to_pretty_json()
            .expect("failed to serialize OpenAPI spec")
    );
}

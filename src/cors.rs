use rocket::{fairing::AdHoc, http::Header};

pub fn stage() -> AdHoc {
    AdHoc::on_response("Cors", |_req, resp| {
        Box::pin(async move {
            resp.set_header(Header::new("Access-Control-Allow-Origin", "*"));
            resp.set_header(Header::new(
                "Access-Control-Allow-Methods",
                "POST, GET, PATCH, OPTIONS,PUT",
            ));
            resp.set_header(Header::new("Access-Control-Allow-Headers", "*"));
            resp.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        })
    })
}

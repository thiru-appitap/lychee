#[cfg(test)]
mod cli {
    use assert_cmd::Command;
    use predicates::str::contains;
    use std::collections::HashMap;

    use http::StatusCode;
    use wiremock::{matchers::path, Mock, MockServer, ResponseTemplate};

    pub async fn get_mock_server_map<S>(pages: HashMap<&str, (S, Option<&str>)>) -> MockServer
    where
        S: Into<StatusCode>,
    {
        let mock_server = MockServer::start().await;

        for (route, (response_code, content)) in pages {
            let template = ResponseTemplate::new(response_code.into());

            let template = if let Some(s) = content {
                template.set_body_string(s)
            } else {
                template
            };

            Mock::given(path(route))
                .respond_with(template)
                .mount(&mock_server)
                .await;
        }

        mock_server
    }

    fn main_command() -> Command {
        // this gets the "main" binary name (e.g. `lychee`)
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("Couldn't get cargo package name")
    }

    #[tokio::test]
    async fn test_recursion() {
        let mut cmd = main_command();

        let mut routes = HashMap::new();
        routes.insert("/", (http::StatusCode::OK, Some("./foo.html")));
        routes.insert("/foo.html", (http::StatusCode::OK, Some("./bar.html")));
        routes.insert(
            "/bar.html",
            (
                http::StatusCode::OK,
                Some("./baz.html ./path/to/frabz.html ./foo.html"),
            ),
        );
        routes.insert("/path/to/frabz.html", (http::StatusCode::OK, Some("ok")));

        let mock_server = get_mock_server_map(routes).await;

        let endpoint = mock_server.uri();

        // println!("{}", endpoint);
        // let ten_millis = time::Duration::from_millis(100000000000);
        // thread::sleep(ten_millis);

        cmd.arg("--recursive")
            .arg("--base-url")
            .arg(&endpoint)
            .arg("--")
            .arg(&endpoint)
            .assert()
            .success()
            .stdout(contains("Total............4"))
            .stdout(contains("Excluded.........0"))
            .stdout(contains("Successful.......4"))
            .stdout(contains("Errors...........0"));
    }
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct User {
    username: String,
    password: String,
}

#[cfg(test)]
mod qb_req {
    use std::error::Error;

    use qbit_rs::{model::Credential, Qbit};
    use reqwest::blocking::Client;

    #[test]
    fn get_req() -> Result<(), Box<dyn Error>> {
        let client = Client::new();

        let url = String::from("https://www.qb.com/api/v2/auth/login");

        let params = [("username", "username"), ("password", "password")];

        let response = client
            .post(url)
            .header("Referer", "https://www.qb.com")
            .form(&params)
            .send()?;

        if response.status().is_success() {
            let headers = response.headers().clone();
            let body = response.text()?;
            println!("res: {}, headers: {:?}", body, headers);
        } else {
            println!("fail status: {}", response.status());
        }
        Ok(())
    }

    #[tokio::test]
    async fn qb_login() {
        let credential = Credential::new("username", "password");
        let api = Qbit::builder()
            .endpoint("https://xxx.xxx.xxx")
            .credential(credential)
            .build();

        let _ = api.login(false).await;

        if let Some(cookie) = api.get_cookie().await {
            println!("cookie {}", cookie);
        } else {
            println!("login failure");
        }

        if let Ok(torrents) = api.get_categories().await {
            println!("category {:?}", torrents);
        }
    }
}

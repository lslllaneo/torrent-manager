use std::sync::OnceLock;

use anyhow::{anyhow, Error, Result};
use qbit_rs::{
    model::{Credential, GetTorrentListArg, Torrent},
    Qbit,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    url: String,
    username: String,
    password: String,
}

static CLIENT: OnceLock<Qbit> = OnceLock::new();

pub async fn qb_login(user: User) -> anyhow::Result<()> {
    let credential = Credential::new(user.username, user.password);

    let qbit = Qbit::builder()
        .endpoint(&(*user.url))
        .credential(credential)
        .build();

    if let Ok(()) = CLIENT.set(qbit) {
        Ok(())
    } else {
        Err(anyhow!("the client logined repeatlly"))
    }
}

pub async fn get_torrents() -> anyhow::Result<Vec<Torrent>> {
    let client = CLIENT.get();

    if let Some(qbit) = client {
        let arg = GetTorrentListArg::builder().build();

        qbit.get_torrent_list(arg)
            .await
            .map_err( anyhow::Error::new)

    } else {
        Err(anyhow!("the client is not inited."))
    }
}

#[cfg(test)]
mod qb_req {
    use crate::req::{get_torrents, User};
    use std::{error::Error, sync::Arc};

    use qbit_rs::{model::Credential, Qbit};
    use reqwest::blocking::Client;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn rwlock_test() {
        let lock = Arc::new(RwLock::new(6));

        let lock_clone = lock.clone();

        let handle = tokio::spawn(async move {
            let read_guard = lock_clone.read().await;
            println!("Read: {}", *read_guard);
        });

        handle.await.unwrap();
    }

    #[test]
    fn get_req() -> Result<(), Box<dyn Error>> {
        let client = Client::new();

        let url = String::from("https://xxx.qb_user.com");

        let params = [("username", "qb_user"), ("password", "password_qb")];

        let response = client
            .post(url)
            .header("Referer", "https://xxx.qb_user.com")
            .form(&params)
            .send()?;

        if response.status().is_success() {
            let headers = response.headers().clone();
            let body = response.text()?;
            println!("res: {}, headers: {:?}", body, headers);
        } else {
            eprintln!("fail status: {}", response.status());
        }
        Ok(())
    }

    #[tokio::test]
    async fn qb_login() {
        let credential = Credential::new("qb_user", "password_qb");
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

    #[tokio::test]
    async fn qb_login_test() {
        let user = User {
            url: "https://xxx.qb_user.com".to_string(),
            username: "qb_user".to_string(),
            password: "password_qb".to_string(),
        };

        if let Err(error) = super::qb_login(user).await {
            eprintln!("error login {}", error)
        }

        match get_torrents().await {
            Ok(version) => println!("version:{:?}", version[0]),
            Err(error) => eprintln!("get version failed, {}", error),
        }
    }
}

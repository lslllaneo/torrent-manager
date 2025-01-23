use std::{borrow::Borrow, sync::OnceLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use qbit_rs::{
    model::{Credential, GetTorrentListArg},
    Qbit,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use transmission_rpc::{
    types::{BasicAuth, ErrorType},
    TransClient,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    url: String,
    username: String,
    password: String,
}

#[allow(unused)]
static QB_CLIENT: OnceLock<Qbit> = OnceLock::new();

//static TR_CLIENT: OnceLock<TransClient> = OnceLock::new();

lazy_static! {
    static ref TR_CLIENT: Mutex<Option<TransClient>> = Mutex::new(None);
}

#[allow(unused)]
pub async fn transmission_login(user: User) -> anyhow::Result<()> {
    let basic_auth = BasicAuth {
        user: user.username,
        password: user.password,
    };

    let client = TransClient::with_auth((&(*user.url)).try_into()?, basic_auth);

    let mut tr_client_gurad = TR_CLIENT.lock().await;
    *tr_client_gurad = Some(client);

    Ok(())
}

#[allow(unused)]
pub async fn tran_login_barely(url_str: &str) -> anyhow::Result<()> {
    let url: Url = url_str.try_into()?;

    let client = TransClient::new(url);

    let mut tr_client_guard = TR_CLIENT.lock().await;
    *tr_client_guard = Some(client);

    Ok(())
}

#[allow(unused)]
pub async fn qb_login(user: User) -> anyhow::Result<()> {
    let credential = Credential::new(user.username, user.password);

    let qbit = Qbit::builder()
        .endpoint(&(*user.url))
        .credential(credential)
        .build();

    if let Ok(()) = QB_CLIENT.set(qbit) {
        Ok(())
    } else {
        Err(anyhow!("the client logined repeatlly"))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Torrent {
    pub file: Option<String>,
    pub source: String,
}

#[allow(unused)]
/// todo
/// get the torrent list from qbittorrent and tranmission
pub async fn get_torrents() -> anyhow::Result<Vec<Torrent>> {
    let qb_client = QB_CLIENT.get();

    if let Some(qbit) = qb_client {
        let arg = GetTorrentListArg::builder().build();

        let qb_torrents: Vec<Torrent> = qbit
            .get_torrent_list(arg)
            .await
            .map_err(anyhow::Error::new)?
            .into_iter()
            .map(|torrent| Torrent {
                file: torrent.save_path,
                source: "qb".to_string(),
            })
            .collect();
    }

    if let Some(ref mut tr_client) = *TR_CLIENT.lock().await {
        let tr_torrents: Vec<Torrent> = tr_client
            .torrent_get(None, None)
            .await
            .map(|res| res.arguments.torrents)
            .map_err(|e| anyhow!(e))?
            .into_iter()
            .map(|torrent| -> Torrent {
                Torrent {
                    file: torrent.download_dir,
                    source: "tr".to_string(),
                }
            })
            .collect();
    }

    Ok(Vec::new())
}

/// Query the seeds that match the given information among the seeds whose status is Error or Alarm
#[allow(unused)]
pub async fn match_torrent_info(
    info: &str,
) -> Result<Vec<transmission_rpc::types::Torrent>, anyhow::Error> {
    let mut tr_gurad = TR_CLIENT.lock().await;

    if let Some(ref mut tr_client) = *tr_gurad {
        let res_torrents: Vec<transmission_rpc::types::Torrent> = tr_client
            .torrent_get(None, None)
            .await
            .map(|res| res.arguments.torrents)
            .map_err(|e| anyhow!(e))?
            .into_iter()
            .filter(|torrent| {
                let is_ok = torrent.error.unwrap_or(ErrorType::Ok) == ErrorType::Ok;
                if is_ok {
                    return false;
                }
                torrent
                    .error_string
                    .borrow()
                    .as_deref()
                    .unwrap_or("")
                    .contains(info)
            })
            .collect();
        Ok(res_torrents)
    } else {
        Err(anyhow!("tranmission client is not exists"))
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
    async fn match_info_test() {
        let url = "http://192.168.31.151:19091/transmission/rpc";
        let error_info = "Ensure your drives are connected or use";
        match super::tran_login_barely(url).await {
            Ok(_) => match super::match_torrent_info(error_info).await {
                Ok(torrents) => {
                    println!("the torrents:{:?}", torrents);
                }
                Err(e) => {
                    println!("no matched torrents, {}", e);
                }
            },
            Err(e) => {
                eprintln!("login failed, {}", e);
            }
        }
    }

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

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumString};
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

type Users = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;
type Usernames = Arc<RwLock<Vec<String>>>;


#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum RequestActionTypes {
    AuthRequest,
    GetClientsRequest,
    RunRequest,
    RunResponse,
}


#[derive(Debug, Serialize, Deserialize)]
struct BasicRequest {
    action: String,
    data: Value,
}


#[derive(Debug, Serialize, Deserialize)]
struct AuthRequestBody {
    app_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BasicRequestResponse {
    action: String,
    data: Value,
}

impl BasicRequestResponse {
    fn new(action: String, data: Value) -> BasicRequestResponse {
        BasicRequestResponse {
            action,
            data,
        }
    }

    fn to_json_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct RunRequestBody {
    target: String,
    module: String,
    params: Value,
}


#[derive(Clone)]
pub(crate) struct RequestsHandler {
    pub connected_users: Users,
    logged_in_clients: Usernames,
    logged_in_admins: Usernames,
}

impl RequestsHandler {
    pub fn new() -> RequestsHandler {
        RequestsHandler {
            connected_users: Users::default(),
            logged_in_clients: Usernames::default(),
            logged_in_admins: Usernames::default(),
        }
    }

    pub async fn handle_new_socket_connection(&self, username: &String, tx: &mpsc::UnboundedSender<Message>) {
        tracing::info!("{} connected", username);
        self.connected_users.write().await.insert(username.clone(), tx.clone());
    }

    pub async fn handle_disconnected_socket(&self, username: &String) {
        tracing::info!("{} disconnected", username);
        self.connected_users.write().await.remove(username);

        // if user is logged in, remove him from the logged in users list
        if self.logged_in_clients.read().await.contains(username) {
            self.logged_in_clients.write().await.retain(|x| x != username);

            // propagate the new list of logged in users to all the admins
            self.send_clients_updates().await;
        }

        // if admin is logged in, remove him from the logged in admin list
        if self.logged_in_admins.read().await.contains(username) {
            self.logged_in_admins.write().await.retain(|x| x != username);
        }
    }

    pub async fn handle_request(&self, message: Message, username: &String) {
        tracing::debug!("Received message: {:?}", message);

        let parsed_message: BasicRequest = match serde_json::from_str(message.to_str().unwrap()) {
            Ok(parsed_message) => parsed_message,
            Err(_) => return, // If the message is not a valid JSON, ignore it
        };

        tracing::debug!("Parsed message: {:?}", parsed_message);
        tracing::info!("New action request: {:?}", parsed_message.action);

        // dispatch action to the corresponding function
        match RequestActionTypes::from_str(&*parsed_message.action) {
            Ok(RequestActionTypes::AuthRequest) => self.handle_auth_request(parsed_message.data, username).await,
            Ok(RequestActionTypes::GetClientsRequest) => self.handle_get_clients_request(username).await,
            Ok(RequestActionTypes::RunRequest) => self.handle_run_request(parsed_message.data, username).await,
            Ok(RequestActionTypes::RunResponse) => self.handle_run_response(parsed_message.data).await,
            Err(_) => tracing::error!("Invalid action {:?}", parsed_message.action),
        }
    }

    async fn send_clients_updates(&self) {
        self.send_messages(
            &self.logged_in_admins.read().await.clone(),
            &BasicRequestResponse::new(
                "clients_update".to_string(),
                serde_json::json!({
                            "connected_clients": self.logged_in_clients.read().await.clone(),
                        }),
            ).to_json_string(),
        ).await;
    }

    async fn get_tx(&self, username: &String) -> Option<mpsc::UnboundedSender<Message>> {
        match self.connected_users.read().await.get(username) {
            Some(tx) => Some(tx.clone()),
            None => {
                tracing::info!("{} is not connected", username);
                None
            }
        }
    }

    async fn send_messages(&self, usernames: &Vec<String>, message: &String) {
        for username in usernames {
            let tx = match self.get_tx(username).await {
                Some(tx) => tx,
                None => continue,
            };

            tx.send(Message::text(message.clone())).unwrap();
        }
    }

    async fn handle_get_clients_request(&self, username: &String) {
        if !self.logged_in_admins.read().await.contains(username) {
            tracing::info!("{} is not an admin", username);
            return;
        }

        let logged_in_clients = self.logged_in_clients.read().await.clone();

        self.send_messages(
            &vec![username.clone()],
            &serde_json::to_string(&logged_in_clients).unwrap(),
        ).await;
    }

    async fn handle_auth_request(&self, data: Value, username: &String) {
        let data = match serde_json::from_value::<AuthRequestBody>(data) {
            Ok(auth_request_body) => auth_request_body,
            Err(_) => {
                tracing::info!("Invalid auth request");
                return;
            }
        };

        match data.app_key.as_str() {
            env!("CLIENT_KEY") => {
                self.logged_in_clients.write().await.push(username.clone());
                self.send_clients_updates().await; // propagate the new list of logged in users to all the admins
            }
            env!("ADMIN_KEY") => self.logged_in_admins.write().await.push(username.clone()),
            _ => tracing::error!("Invalid app key"),
        }
    }

    async fn handle_run_request(&self, data: Value, username: &String) {
        let data = match serde_json::from_value::<RunRequestBody>(data) {
            Ok(run_request_body) => run_request_body,
            Err(_) => {
                tracing::error!("Invalid run request body");
                return;
            }
        };

        let target = data.target;
        let module = data.module;
        let params = data.params;

        tracing::info!("Target: {}", target);
        tracing::info!("Module: {}", module);
        tracing::info!("Params: {:?}", params);

        if !self.logged_in_clients.read().await.contains(&target) {
            tracing::error!("{} is not a client", target);
            self.send_messages(
                &vec![username.clone()],
                &BasicRequestResponse::new(
                    "run".to_string(),
                    serde_json::json!({
                        "error": "Target isn't a client",
                    }),
                ).to_json_string(),
            ).await;
            return;
        }


        self.send_messages(
            &vec![target],
            &BasicRequestResponse::new(
                "run".to_string(),
                serde_json::json!({
                    "module": module,
                    "params": params,
                }),
            ).to_json_string(),
        ).await;
    }

    async fn handle_run_response(&self, data: Value) {
        // propagate the response to all the admins
        self.send_messages(
            &self.logged_in_admins.read().await.clone(),
            &BasicRequestResponse::new(
                "run_response".to_string(),
                data,
            ).to_json_string(),
        ).await;
    }
}
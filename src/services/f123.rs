use crate::{
    config::Database,
    dtos::F123Packet,
    error::{AppResult, SocketError},
};
use ahash::AHashMap;
use bincode::serialize;
use redis::Commands;
use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, process::Command, sync::RwLock, task::JoinHandle};
use tracing::{error, info};

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<AHashMap<String, JoinHandle<()>>>>,
    ip_addresses: Arc<RwLock<AHashMap<String, IpAddr>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            sockets: Arc::new(RwLock::new(AHashMap::new())),
            ip_addresses: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: i16, championship_id: Arc<String>) -> AppResult<()> {
        {
            let sockets = self.sockets.read().await;

            if sockets.contains_key(&championship_id.to_string()) {
                return Err(SocketError::AlreadyExists.into());
            }
        }

        let db = self.db_conn.clone();
        let championship_clone = championship_id.clone();
        let ip_addresses = self.ip_addresses.clone();

        // TODO: Close socket when championship is finished or when the server is idle for a long time
        let socket = tokio::task::spawn(async move {
            let mut closed_ports = false;
            let mut buf = [0; 1460];
            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let (session, mut redis) = (db.get_scylla(), db.get_redis());
            let (open_machine_port, close_port_for_all_except) =
                (Self::open_machine_port, Self::close_port_for_all_except);
            let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)).await else {
                error!("There was an error binding to the socket");
                return;
            };

            open_machine_port(port).await.unwrap();

            // TODO: Save all this data in redis and only save it in the database when the session is finished
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, address)) => {
                        if !closed_ports {
                            close_port_for_all_except(port as u16, address.ip())
                                .await
                                .unwrap();

                            closed_ports = true;

                            {
                                let mut ip_addresses = ip_addresses.write().await;
                                ip_addresses.insert(championship_id.to_string(), address.ip());
                            }
                        }

                        let Ok(header) = F123Packet::parse_header(&buf[..size]) else {
                            continue;
                        };

                        let session_id = header.m_sessionUID as i64;
                        if session_id == 0 {
                            continue;
                        }

                        let Ok(Some(packet)) = F123Packet::parse(header.m_packetId, &buf[..size])
                        else {
                            continue;
                        };

                        match packet {
                            F123Packet::SessionHistory(session_history) => {
                                let Ok(data) = serialize(&session_history) else {
                                    error!("There was an error serializing the session history data");
                                    continue;
                                };

                                redis
                                    .set_ex::<String, Vec<u8>, String>(
                                        format!("f123:championship:{championship_id}:session:{session_id}:history:car:{}", session_history.m_carIdx),
                                        data,
                                        60 * 60,
                                    )
                                    .unwrap();
                            }

                            F123Packet::Motion(motion_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_car_motion_update)
                                    >= Duration::from_millis(500)
                                {
                                    let Ok(data) = serialize(&motion_data) else {
                                        error!("There was an error serializing the motion data");
                                        continue;
                                    };

                                    redis
                                        .set_ex::<String, Vec<u8>, String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:motion",
                                                championship_id
                                            ),
                                            data,
                                            60 * 60,
                                        )
                                        .unwrap();

                                    last_car_motion_update = now;
                                }
                            }

                            F123Packet::Session(session_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_session_update)
                                    >= Duration::from_secs(30)
                                {
                                    let Ok(data) = serialize(&session_data) else {
                                        error!("There was an error serializing the session data");
                                        continue;
                                    };

                                    redis
                                        .set_ex::<String, Vec<u8>, String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:session",
                                                championship_id
                                            ),
                                            data,
                                            60 * 60,
                                        )
                                        .unwrap();

                                    last_session_update = now;
                                }
                            }

                            // We don't save events in redis because redis doesn't support lists of lists
                            F123Packet::Event(event_data) => {
                                let Ok(event) = serialize(&event_data.m_eventDetails) else {
                                    error!("There was an error serializing the event data");
                                    continue;
                                };

                                let table_exists = session
                                    .execute(
                                        db.statements.get("select_event_data").unwrap(),
                                        (session_id, event_data.m_eventStringCode),
                                    )
                                    .await
                                    .unwrap()
                                    .rows_or_empty();

                                if table_exists.is_empty() {
                                    session
                                        .execute(
                                            db.statements.get("insert_event_data").unwrap(),
                                            (session_id, event_data.m_eventStringCode, vec![event]),
                                        )
                                        .await
                                        .unwrap();
                                } else {
                                    session
                                        .execute(
                                            db.statements.get("update_event_data").unwrap(),
                                            (vec![event], session_id, event_data.m_eventStringCode),
                                        )
                                        .await
                                        .unwrap();
                                }
                            }

                            F123Packet::Participants(participants_data) => {
                                let Ok(participants) = serialize(&participants_data.m_participants)
                                else {
                                    error!("There was an error serializing the participants data");
                                    continue;
                                };

                                redis
                                    .set_ex::<String, Vec<u8>, String>(
                                        format!(
                                        "f123:championship:{}:session:{session_id}:participants",
                                        championship_id
                                    ),
                                        participants.clone(),
                                        60 * 60,
                                    )
                                    .unwrap();
                            }

                            F123Packet::FinalClassification(classification_data) => {
                                let Ok(classifications) =
                                    serialize(&classification_data.m_classificationData)
                                else {
                                    error!("There was an error serializing the final classification data");
                                    continue;
                                };

                                // TODO: Save all laps for each driver in the final classification
                                session
                                    .execute(
                                        db.statements
                                            .get("insert_final_classification_data")
                                            .unwrap(),
                                        (session_id, classifications),
                                    )
                                    .await
                                    .unwrap();
                            }
                        }
                    }

                    Err(e) => {
                        error!("Error receiving packet: {}", e);
                    }
                }
            }
        });

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(championship_clone.to_string(), socket);
        }

        Ok(())
    }

    pub async fn active_sockets(&self) -> AppResult<Vec<String>> {
        let sockets = self.sockets.read().await;

        Ok(sockets.keys().cloned().collect())
    }

    // pub async fn stop_all_sockets(&self) {
    //     let mut sockets = self.sockets.write().await;

    //     for socket in sockets.iter() {
    //         socket.1.abort();
    //     }

    //     sockets.clear();
    // }

    pub async fn stop_socket(&self, championship_id: String, port: i16) -> AppResult<()> {
        {
            let mut sockets = self.sockets.write().await;
            let Some(socket) = sockets.remove(&championship_id) else {
                Err(SocketError::NotFound)?
            };

            socket.abort();
        }
        // TODO: Check if the port is closed
        {
            let ip_addresses = self.ip_addresses.read().await;
            let ip = ip_addresses.get(&championship_id).unwrap();
            Self::close_machine_port(port, *ip).await.unwrap();
        }

        Ok(())
    }

    async fn open_machine_port(port: i16) -> tokio::io::Result<()> {
        let port_str = port.to_string();

        if cfg!(unix) {
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-A")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(port_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to open port with iptables",
                ));
            }
        } else {
            info!("The machine is not running a unix based OS, so the port will not be opened automatically");
        }

        Ok(())
    }

    async fn close_port_for_all_except(port: u16, ip: IpAddr) -> std::io::Result<()> {
        if cfg!(unix) {
            let port_str = port.to_string();
            let ip_str = ip.to_string();

            // Primero, borramos cualquier regla existente que afecte al puerto especificado.
            let _ = Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            let _ = Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await?;

            // Luego, agregamos las nuevas reglas.
            // Bloquear todas las conexiones a este puerto
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-A")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to close port for all with iptables",
                ));
            }

            // Permitir conexiones desde la IP específica
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-I")
                .arg("INPUT")
                .arg("1")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-s")
                .arg(&ip_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to open port for specific IP with iptables",
                ));
            }
        }

        Ok(())
    }

    async fn close_machine_port(port: i16, ip: IpAddr) -> tokio::io::Result<()> {
        if cfg!(unix) {
            let port_str = port.to_string();
            let ip_str = ip.to_string();

            // Elimina la regla que permite las conexiones desde una IP específica
            match Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-s")
                .arg(&ip_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("Failed to remove specific IP rule with iptables");
                    }
                }
                Err(e) => eprintln!("Failed to execute command: {}", e),
            }

            // Elimina la regla que bloquea todas las demás conexiones
            match Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("Failed to remove drop rule with iptables");
                    }
                }
                Err(e) => eprintln!("Failed to execute command: {}", e),
            }
        } else {
            info!("The machine is not running a unix based OS, so the port will not be closed automatically");
        }

        Ok(())
    }
}

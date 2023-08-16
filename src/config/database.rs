use ahash::AHashMap;
use dotenvy::var;
use redis::{aio::Connection, Client};
use scylla::{prepared_statement::PreparedStatement, Session, SessionBuilder};
use std::sync::Arc;
use tokio::{fs, try_join};
use tracing::info;

pub struct Database {
    redis: Client,
    pub scylla: Arc<Session>,
    pub statements: Arc<AHashMap<String, PreparedStatement>>,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let scylla = SessionBuilder::new()
            .known_node(var("SCYLLA_URI").unwrap())
            // .user(var("SCYLLA_USER").unwrap(), var("SCYLLA_PASS").unwrap())
            .use_keyspace(var("SCYLLA_KEYSPACE").unwrap(), true)
            .build()
            .await
            .unwrap();

        let redis = Client::open(var("REDIS_URL").unwrap()).unwrap();

        info!("Connected To Database! Parsing Schema & Saving Prepared Statements...");
        Self::parse_schema(&scylla).await;
        let statements = Self::prepared_statements(&scylla).await;

        info!("Prepared Statements Saved!, Returning Database Instance");
        Self {
            redis,
            scylla: Arc::new(scylla),
            statements: Arc::new(statements),
        }
    }

    async fn parse_schema(session: &Session) {
        let schema = fs::read_to_string("src/config/schema.cql").await.unwrap();

        for q in schema.split(';') {
            let query = q.to_owned() + ";";

            if query.len() > 1 {
                session.query(query, &[]).await.unwrap();
            }
        }
    }

    async fn prepared_statements(session: &Session) -> AHashMap<String, PreparedStatement> {
        let mut statements: AHashMap<String, PreparedStatement> = AHashMap::new();

        //* User Tasks
        let insert_user_task = session
            .prepare("INSERT INTO users (id, username, password, email, active, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)");

        let user_email_by_email_task =
            session.prepare("SELECT email FROM users where email = ? ALLOW FILTERING");

        let user_by_id_task = session.prepare("SELECT * FROM users where id = ? ALLOW FILTERING");

        let user_by_email_task =
            session.prepare("SELECT * FROM users where email = ? ALLOW FILTERING");

        let delete_user_task = session.prepare("DELETE FROM users WHERE id = ?");

        let activate_user_task = session.prepare("UPDATE users SET active = true WHERE id = ?");

        let deactivate_user_task = session.prepare("UPDATE users SET active = false WHERE id = ?");

        //* Championships Tasks
        let insert_championships_task = session
            .prepare(
                "INSERT INTO championships (id, name, port, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            );

        let championships_by_user_id_task =
            session.prepare("SELECT * FROM championships where user_id = ? ALLOW FILTERING");

        let championship_by_id_task = session.prepare("SELECT * FROM championships where id = ?");

        let championship_by_name_task =
            session.prepare("SELECT name FROM championships where name = ? ALLOW FILTERING");

        let championships_ports_task = session.prepare("SELECT port FROM championships");

        let delete_championship_task = session.prepare("DELETE FROM championships WHERE id = ?");

        //* Event Data Tasks
        let select_event_data_task =
            session.prepare("SELECT * FROM event_data WHERE session_id = ? AND string_code = ?;");

        let insert_event_data_task = session
            .prepare("INSERT INTO event_data (session_id, string_code, events) VALUES (?,?,?);");

        let update_event_data_task = session.prepare(
            "UPDATE event_data SET events = events + ? WHERE session_id = ? AND string_code = ?;",
        );

        let events_data_task = session.prepare("SELECT * FROM event_data WHERE session_id = ?;");

        //* Other Tasks
        let insert_lap_data_task =
            session.prepare("INSERT INTO lap_data (session_id, lap) VALUES (?,?);");

        let insert_game_session_task =
            session.prepare("INSERT INTO game_sessions (id, data) VALUES (?,?);");

        let insert_participant_data_task = session
            .prepare("INSERT INTO participants_data (session_id, participants) VALUES (?,?);");

        let insert_final_classification_data_task = session.prepare(
            "INSERT INTO final_classification_data (session_id, classification) VALUES (?,?);",
        );

        let (
            insert_user,
            user_email_by_email,
            user_by_id,
            user_by_email,
            delete_user,
            activate_user,
            deactivate_user,
            insert_championships,
            championship_by_name,
            championships_ports,
            championship_by_id,
            insert_game_session,
            insert_lap_data,
            select_event_data,
            insert_event_data,
            update_event_data,
            insert_participant_data,
            insert_final_classification_data,
            events_data,
            championships_by_user_id,
            delete_championship,
        ) = try_join!(
            insert_user_task,
            user_email_by_email_task,
            user_by_id_task,
            user_by_email_task,
            delete_user_task,
            activate_user_task,
            deactivate_user_task,
            insert_championships_task,
            championship_by_name_task,
            championships_ports_task,
            championship_by_id_task,
            insert_game_session_task,
            insert_lap_data_task,
            select_event_data_task,
            insert_event_data_task,
            update_event_data_task,
            insert_participant_data_task,
            insert_final_classification_data_task,
            events_data_task,
            championships_by_user_id_task,
            delete_championship_task
        )
        .unwrap();

        //* User Statements
        statements.insert("user.insert".to_string(), insert_user);
        statements.insert("user.email_by_email".to_string(), user_email_by_email);
        statements.insert("user.by_id".to_string(), user_by_id);
        statements.insert("user.by_email".to_string(), user_by_email);
        statements.insert("user.delete".to_string(), delete_user);
        statements.insert("user.activate".to_string(), activate_user);
        statements.insert("user.deactivate".to_string(), deactivate_user);

        //* Championship Statements
        statements.insert("championship.insert".to_owned(), insert_championships);
        statements.insert("championship.ports".to_string(), championships_ports);
        statements.insert("championship.by_id".to_string(), championship_by_id);
        statements.insert("championship.by_user".to_string(), championships_by_user_id);
        statements.insert("championship.delete".to_string(), delete_championship);
        statements.insert(
            "championship.name_by_name".to_string(),
            championship_by_name,
        );

        //* Event Data Statements
        statements.insert("event_data.get".to_string(), select_event_data);
        statements.insert("event_data.insert".to_string(), insert_event_data);
        statements.insert("event_data.update".to_string(), update_event_data);
        statements.insert("event_data.events_by_id".to_string(), events_data);

        // TODO: Update this to use the new statements
        statements.insert("insert_game_session".to_string(), insert_game_session);
        statements.insert("insert_lap_data".to_string(), insert_lap_data);
        statements.insert(
            "insert_participant_data".to_string(),
            insert_participant_data,
        );
        statements.insert(
            "insert_final_classification_data".to_string(),
            insert_final_classification_data,
        );

        statements
    }

    pub fn get_redis(&self) -> redis::Connection {
        self.redis.get_connection().unwrap()
    }

    pub async fn get_redis_async(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}

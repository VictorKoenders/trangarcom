use trangarcom::DbConnection;

pub struct AppState {
    pub db: DbConnection,
}

pub struct StateProvider {
    db: DbConnection,
}

impl StateProvider {
    pub fn new() -> StateProvider {
        let db = ::trangarcom::establish_connection().unwrap();
        StateProvider {
            db,
        }
    }

    pub fn create_state(&self) -> AppState {
        AppState {
            db: self.db.clone()
        }
    }
}


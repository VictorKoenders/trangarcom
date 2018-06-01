use trangarcom::DbConnection;
use failure::Error;

pub struct AppState {
    pub db: DbConnection,
}

pub struct StateProvider {
    db: DbConnection,
}

impl StateProvider {
    pub fn new() -> Result<StateProvider, Error> {
        let db = ::trangarcom::establish_connection()?;
        Ok(StateProvider {
            db,
        })
    }

    pub fn create_state(&self) -> AppState {
        AppState {
            db: self.db.clone()
        }
    }
}


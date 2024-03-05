#![allow(unused)]

use super::*;

#[derive(Clone, Default)]
pub struct LocalStore {
    connstr: String,
}

impl LocalStore {
    pub fn open(connstr: String) -> Result<Self, Error> {
        let this = Self { connstr };
        // this.migrate()?;
        Ok(this)
    }

    #[cfg(test)]
    pub fn memory() -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let db_name = (0..7)
            .map(|_| rand::Rng::sample(&mut rng, rand::distributions::Alphanumeric) as char)
            .collect::<String>();
        let connstr = format!("file:{db_name}?mode=memory&cache=shared");
        Box::leak(Box::new(rusqlite::Connection::open(&connstr)?));
        Self::open(connstr)
    }

    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(rusqlite::Connection) -> Result<T, Error>,
    ) -> Result<T, Error> {
        f(rusqlite::Connection::open(&*self.connstr)?)
    }

    pub fn with_tx<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.with_conn(|mut conn| {
            let tx = conn.transaction()?;
            let res = f(&tx)?;
            tx.commit()?;
            Ok(res)
        })
    }

    const fn migrations() -> &'static [&'static str] {
        &[]
    }
}

impl Store for LocalStore {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        todo!()
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        todo!()
    }

    async fn delete_share(&self, share: ShareId) -> Result<(), Error> {
        todo!()
    }

    async fn put_key(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        todo!()
    }

    async fn get_key(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        todo!()
    }

    async fn delete_key(&self, id: KeyId) -> Result<(), Error> {
        todo!()
    }

    async fn create_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
        expiry: u64,
        nonce: Nonce,
    ) -> Result<Option<Permit>, Error> {
        todo!()
    }

    async fn read_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<Option<Permit>, Error> {
        todo!()
    }

    async fn delete_permit(
        &self,
        identity: IdentityLocator,
        recipient: Address,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn get_chain_state(&self, chain: u64) -> Result<Option<ChainState>, Error> {
        todo!()
    }

    async fn update_chain_state(&self, chain: u64, update: ChainStateUpdate) -> Result<(), Error> {
        todo!()
    }

    #[cfg(test)]
    async fn clear_chain_state(&self, chain: u64) -> Result<(), Error> {
        todo!()
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }

    async fn update_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
        config: Vec<u8>,
        version: EventIndex,
    ) -> Result<(), Error> {
        todo!()
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityId,
    ) -> Result<(), Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // crate::make_store_tests!(LocalStore::memory().unwrap());
}

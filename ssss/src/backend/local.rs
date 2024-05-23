#![allow(unused)]

use super::*;

#[derive(Clone, Default)]
pub struct Local {
    connstr: String,
}

impl Local {
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
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, Error>,
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

impl Store for Local {
    async fn put_share(&self, id: ShareId, share: SecretShare) -> Result<bool, Error> {
        todo!()
    }

    async fn commit_share(&self, id: ShareId) -> Result<bool, Error> {
        todo!()
    }

    async fn get_share(&self, id: ShareId) -> Result<Option<SecretShare>, Error> {
        todo!()
    }

    async fn get_current_share_version(
        &self,
        identity: IdentityLocator,
        name: String,
    ) -> Result<Option<(ShareVersion, bool /* pending */)>, Error> {
        todo!()
    }

    async fn delete_share(&self, share: ShareId) -> Result<(), Error> {
        todo!()
    }

    async fn put_secret(&self, id: KeyId, key: WrappedKey) -> Result<bool, Error> {
        todo!()
    }

    async fn get_secret(&self, id: KeyId) -> Result<Option<WrappedKey>, Error> {
        todo!()
    }

    async fn delete_secret(&self, id: KeyId) -> Result<(), Error> {
        todo!()
    }

    async fn put_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
        config: Vec<u8>,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn get_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }

    #[cfg(test)]
    async fn clear_verifier(
        &self,
        permitter: PermitterLocator,
        identity: IdentityLocator,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl Signer for Local {
    async fn sign(&self, hash: H256) -> Result<Signature, Error> {
        todo!()
    }

    async fn signer_address(&self) -> Result<Address, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // crate::make_store_tests!(LocalStore::memory().unwrap());
}

use super::*;

#[derive(Clone)]
pub struct Client {
    inner: aws_sdk_dynamodb::Client,
    table: String,
}

impl Client {
    pub fn new(config: &aws_config::SdkConfig, table: String) -> Self {
        Self {
            inner: aws_sdk_dynamodb::Client::new(config),
            table,
        }
    }
}

impl ShareStore for Client {
    async fn create(&self, id: ShareId) -> Result<(), Error> {
        todo!()
    }

    async fn get(&self, id: ShareId, version: ShareVersion) -> Result<Option<WrappedShare>, Error> {
        todo!()
    }
}

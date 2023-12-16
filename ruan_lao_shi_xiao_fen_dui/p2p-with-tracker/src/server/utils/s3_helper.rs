use s3::{creds::Credentials, error::S3Error, serde_types::HeadObjectResult, Bucket, Region};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Client {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl S3Client {
    pub fn new(
        endpoint: String,
        bucket: String,
        region: String,
        access_key: String,
        secret_key: String,
    ) -> Self {
        Self {
            endpoint,
            bucket,
            region,
            access_key,
            secret_key,
        }
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub fn get_bucket(&self) -> String {
        self.bucket.clone()
    }

    pub fn get_region(&self) -> String {
        self.region.clone()
    }

    pub fn get_access_key(&self) -> String {
        self.access_key.clone()
    }

    pub fn get_secret_key(&self) -> String {
        self.secret_key.clone()
    }

    pub async fn get_object_metadata(&self, key: String) -> Result<HeadObjectResult, S3Error> {
        let endpoint = self.get_endpoint();
        let bucket = self.get_bucket();
        let region = self.get_region();
        let access_key = self.get_access_key();
        let secret_key = self.get_secret_key();

        let region = Region::Custom {
            region: region.to_owned(),
            endpoint: endpoint.to_owned(),
        };
        let credentials = Credentials::new(
            Some(access_key.to_owned().as_str()),
            Some(secret_key.to_owned().as_str()),
            None,
            None,
            None,
        )
        .expect("Failed to create credentials");

        let bucket_object =
            Bucket::new(bucket.as_str(), region.clone(), credentials.clone())?.with_path_style();

        // Get meta data
        let (data, _code) = bucket_object.head_object(key.as_str()).await?;

        Ok(data)
    }

    pub async fn get_object_range_data(
        &self,
        key: String,
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>, S3Error> {
        let endpoint = self.get_endpoint();
        let bucket = self.get_bucket();
        let region = self.get_region();
        let access_key = self.get_access_key();
        let secret_key = self.get_secret_key();

        let region = Region::Custom {
            region: region.to_owned(),
            endpoint: endpoint.to_owned(),
        };
        let credentials = Credentials::new(
            Some(access_key.to_owned().as_str()),
            Some(secret_key.to_owned().as_str()),
            None,
            None,
            None,
        )
        .expect("Failed to create credentials");

        let bucket_object =
            Bucket::new(bucket.as_str(), region.clone(), credentials.clone())?.with_path_style();

        // Get range data
        let response_data = bucket_object
            .get_object_range(key.as_str(), start, Some(end))
            .await?;
        let data = response_data.to_vec();

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_object_metadata() {
        let endpoint = "http://127.0.0.1:39000";
        let bucket = "hackathon";
        let region = "";
        let access_key = "hackathon";
        let secret_key = "hackathon";

        let s3 = S3Client::new(
            endpoint.to_string(),
            bucket.to_string(),
            region.to_string(),
            access_key.to_string(),
            secret_key.to_string(),
        );
        let res = s3
            .get_object_metadata("output_10MB.txt".to_string())
            .await
            .unwrap();
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_get_object_range_data() {
        let endpoint = "http://127.0.0.1:39000";
        let bucket = "hackathon";
        let region = "";
        let access_key = "hackathon";
        let secret_key = "hackathon";

        let s3 = S3Client::new(
            endpoint.to_string(),
            bucket.to_string(),
            region.to_string(),
            access_key.to_string(),
            secret_key.to_string(),
        );
        let res = s3
            .get_object_range_data("output_10MB.txt".to_string(), 0, 1024)
            .await
            .unwrap();

        println!("{:?}", res);
    }
}

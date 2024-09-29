use std::{collections::HashMap, path::Path};

use serde_json::to_vec;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::storage::CacheData;

#[derive(Debug)]
pub enum SnapshotError {
    FailedToCreateFile,
    FailedToOpenFile,
    FailedToReadFile,
    FailedToWriteFile,
    FailedToSerializeData,
    FailedToDeserializeData,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub directory: String,
    pub filename: String,
}

impl Snapshot {
    pub fn new(directory: String, filename: String) -> Snapshot {
        Snapshot {
            directory,
            filename,
        }
    }

    pub async fn write_storage(
        self,
        storage: HashMap<String, CacheData>,
    ) -> Result<(), SnapshotError> {
        let buf = to_vec(&storage).map_err(|_| SnapshotError::FailedToSerializeData)?;
        self.backup(buf).await
    }

    pub async fn read_storage(self) -> Result<HashMap<String, CacheData>, SnapshotError> {
        let buf = self.load().await;
        match buf {
            Ok(buf) => {
                let data: HashMap<String, CacheData> =
                    serde_json::from_slice(&buf).unwrap_or(HashMap::new());
                Ok(data)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn load(self) -> Result<Vec<u8>, SnapshotError> {
        let dirpath = Path::new(self.directory.as_str());
        let filepath = Path::new(self.filename.as_str());
        let fullpath = Path::join(dirpath, filepath);

        if !dirpath.exists() {
            fs::create_dir(dirpath)
                .await
                .map_err(|_| SnapshotError::FailedToCreateFile)?;
        }

        let mut file: fs::File;
        if !fullpath.exists() {
            match fs::File::create(fullpath.clone()).await {
                Ok(f) => file = f,
                Err(_) => return Err(SnapshotError::FailedToCreateFile),
            }
            println!("create snapshot at {}", fullpath.to_str().unwrap_or("-"));
        } else {
            match fs::File::open(fullpath).await {
                Ok(f) => file = f,
                Err(_) => return Err(SnapshotError::FailedToOpenFile),
            }
        }

        let mut contents = vec![];
        match file.read_to_end(&mut contents).await {
            Ok(_) => (),
            Err(_) => return Err(SnapshotError::FailedToReadFile),
        };

        Ok(contents)
    }

    pub async fn backup(self, data: Vec<u8>) -> Result<(), SnapshotError> {
        let dirpath = Path::new(self.directory.as_str());
        let filepath = Path::new(self.filename.as_str());
        let fullpath = Path::join(dirpath, filepath);

        if !dirpath.exists() {
            fs::create_dir(dirpath)
                .await
                .map_err(|_| SnapshotError::FailedToCreateFile)?;
        }

        let mut file = match fs::File::create(fullpath).await {
            Ok(f) => f,
            Err(_) => return Err(SnapshotError::FailedToCreateFile),
        };

        match file.write_all(&data).await {
            Ok(_) => Ok(()),
            Err(_) => Err(SnapshotError::FailedToWriteFile),
        }
    }
}

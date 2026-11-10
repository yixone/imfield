use imfield_storage::Storage;
use imfield_storage_address::StoragePath;
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

const TEST_DATA: &[u8] = &[0, 0x5, 0xFF, 0x41, 0xFF, 0x42];

async fn prepare() -> Storage {
    let temp = TempDir::new().unwrap();
    Storage::new(temp.path()).await.unwrap()
}

mod write {
    use super::*;

    #[tokio::test]
    async fn write_and_commit() {
        let storage = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_writer(&path).await.unwrap();
        writer.write_reader(TEST_DATA).await.unwrap();

        assert!(!storage.exists(&path).await.unwrap());

        let written = writer.commit().await.unwrap();

        assert_eq!(written, TEST_DATA.len() as u64);

        assert!(storage.exists(&path).await.unwrap());

        let mut reader = storage.get_reader(path).await.unwrap();

        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();

        assert_eq!(out, TEST_DATA);
    }

    #[tokio::test]
    async fn write_and_abort() {
        let storage = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_writer(&path).await.unwrap();
        writer.write_reader(TEST_DATA).await.unwrap();

        writer.abort().await.unwrap();

        assert!(!storage.exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn abort_on_drop() {
        let storage = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_writer(&path).await.unwrap();
        writer.write_reader(TEST_DATA).await.unwrap();

        // Dropping the writer aborts the staged file
        drop(writer);

        assert!(!storage.exists(&path).await.unwrap());
    }

    #[tokio::test]
    async fn return_error_on_conflict() {
        let storage = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_writer(&path).await.unwrap();
        writer.write_reader(TEST_DATA).await.unwrap();
        writer.commit().await.unwrap();

        let err = storage.create_writer(&path).await.err().unwrap();
        assert!(matches!(err, imfield_storage::StorageError::AlreadyExists))
    }
}

mod read {
    use super::*;

    #[tokio::test]
    async fn read_file_seek() {
        let storage = prepare().await;

        let path = StoragePath::new("GYre0rpM5w").push("file");

        let mut writer = storage.create_writer(&path).await.unwrap();
        writer.write_reader(TEST_DATA).await.unwrap();
        writer.commit().await.unwrap();

        let mut reader = storage.get_reader_seek(&path, 3, None).await.unwrap();

        let mut out = Vec::new();
        reader.read_to_end(&mut out).await.unwrap();

        assert_eq!(out[0], TEST_DATA[3]);
        assert_eq!(out.len(), TEST_DATA.len() - 3);
    }
}

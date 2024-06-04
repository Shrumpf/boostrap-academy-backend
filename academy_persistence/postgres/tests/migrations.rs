use academy_persistence_postgres::MIGRATIONS;
use common::{setup, setup_clean};

mod common;

#[tokio::test]
async fn migrations_clean() {
    let db = setup_clean().await;

    let names = MIGRATIONS.iter().map(|m| m.name).collect::<Vec<_>>();

    let applied = db.run_migrations(None).await.unwrap();
    assert_eq!(applied, names);

    for i in 1..=MIGRATIONS.len() {
        let mut reverted = db.revert_migrations(Some(i)).await.unwrap();
        eprintln!("reverted {reverted:?}");
        reverted.reverse();
        assert_eq!(reverted, names[MIGRATIONS.len() - i..]);

        let applied = db.run_migrations(None).await.unwrap();
        eprintln!("applied {applied:?}");
        assert_eq!(applied, names[MIGRATIONS.len() - i..]);
    }
}

#[tokio::test]
async fn migrations_with_data() {
    let db = setup().await;

    let names = MIGRATIONS.iter().map(|m| m.name).collect::<Vec<_>>();

    for i in 1..=MIGRATIONS.len() {
        let mut reverted = db.revert_migrations(Some(i)).await.unwrap();
        eprintln!("reverted {reverted:?}");
        reverted.reverse();
        assert_eq!(reverted, names[MIGRATIONS.len() - i..]);

        let applied = db.run_migrations(None).await.unwrap();
        eprintln!("applied {applied:?}");
        assert_eq!(applied, names[MIGRATIONS.len() - i..]);
    }
}

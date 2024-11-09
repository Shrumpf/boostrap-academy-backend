use std::sync::LazyLock;

use academy_models::job::Job;
use academy_persistence_contracts::jobs::JobsRepository;
use uuid::uuid;

pub static JOB1: LazyLock<Job> = LazyLock::new(|| Job {
    id: uuid!("c4dba95e-8498-40fb-8412-7faecaf94787").into(),
    title: "Demo Job Title".try_into().unwrap(),
});

pub async fn create<Txn: Send + Sync + 'static>(
    txn: &mut Txn,
    repo: impl JobsRepository<Txn>,
) -> anyhow::Result<()> {
    repo.create(txn, &JOB1).await?;

    Ok(())
}

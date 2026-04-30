use evento::Executor;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub async fn scheduler<E: Executor + Clone>(
    evento: &E,
    pool: &SqlitePool,
) -> Result<JobScheduler, JobSchedulerError> {
    let sched = JobScheduler::new().await?;
    let evento = evento.clone();
    let pool = pool.clone();

    // Auto generate user weeks
    sched
        .add(Job::new_async("1/5 * * * * *", move |uuid, mut l| {
            let evento = evento.clone();
            let pool = pool.clone();

            Box::pin(async move {
                if let Err(err) = auto_generate_weeks::<E>(evento, pool).await{
                    tracing::error!(err = %err, "failed to auto generate mealplan user weeks");
                }

                if let Err(err) = l.next_tick_for_job(uuid).await{
                    tracing::error!(err = %err, "failed to get next tick for auto generate mealplan user weeks");
                }
            })
        })?)
        .await?;

    Ok(sched)
}

pub async fn auto_generate_weeks<E: Executor + Clone>(
    evento: E,
    pool: SqlitePool,
) -> anyhow::Result<()> {
    let week =
        super::service::current_week_monday(OffsetDateTime::now_utc().unix_timestamp() as u64)?;

    println!("{week}");

    Ok(())
}

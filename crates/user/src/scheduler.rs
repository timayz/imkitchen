use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_db::table::UserSubscription;
use imkitchen_shared::user::subscription::{Cancelled, StripePaymentIntentSucceeded};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;
use stripe_core::payment_intent::{CreatePaymentIntent, CreatePaymentIntentOffSession};
use stripe_types::Currency;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub async fn scheduler<E: Executor + Clone>(
    evento: &E,
    r_pool: &SqlitePool,
    w_pool: &SqlitePool,
    stripe: &stripe::Client,
) -> Result<JobScheduler, JobSchedulerError> {
    let sched = JobScheduler::new().await?;
    let stripe = stripe.clone();

    let state = imkitchen_shared::State {
        executor: evento.clone(),
        read_db: r_pool.clone(),
        write_db: w_pool.clone(),
    };

    // Renew subscriptions
    sched
        .add(Job::new_async("0 * * * * *", move |uuid, mut l| {
            let stripe = stripe.clone();
            let state = state.clone();

            Box::pin(async move {
                if let Err(err) = renew_subscriptions(state, stripe).await{
                    tracing::error!(err = %err, "failed to renew user subscriptions");
                }

                if let Err(err) = l.next_tick_for_job(uuid).await{
                    tracing::error!(err = %err, "failed to get next tick for auto generate mealplan user weeks");
                }
            })
        })?)
        .await?;

    Ok(sched)
}

async fn renew_subscriptions<E: Executor + Clone>(
    state: imkitchen_shared::State<E>,
    stripe: stripe::Client,
) -> anyhow::Result<()> {
    let Ok(now): Result<u64, _> = time::UtcDateTime::now().unix_timestamp().try_into() else {
        anyhow::bail!("failed to get current timestamp");
    };

    let statement = sea_query::Query::select()
        .columns([UserSubscription::Id, UserSubscription::ExpireAt])
        .from(UserSubscription::Table)
        .and_where(Expr::col(UserSubscription::ExpireAt).lte(now + 600))
        .order_by_expr(Expr::col(UserSubscription::ExpireAt), sea_query::Order::Asc)
        .limit(30)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    let subscriptions = sqlx::query_as_with::<_, (String, u64), _>(&sql, values)
        .fetch_all(&state.read_db)
        .await?;

    let command = crate::Command::new(state.clone());
    for user_sub in subscriptions {
        let subscription = command.subscription.load(&user_sub.0).await?;

        let statement = Query::delete()
            .from_table(UserSubscription::Table)
            .and_where(Expr::col(UserSubscription::Id).eq(&user_sub.0))
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values)
            .execute(&state.write_db)
            .await?;

        if let Err(e) = renew_subscription(&stripe, &command.subscription, subscription).await {
            tracing::error!(err = %e, "failed to renew subscription for {}", &user_sub.0);

            continue;
        }
    }

    Ok(())
}

async fn renew_subscription<E: Executor + Clone>(
    stripe: &stripe::Client,
    command: &crate::subscription::Command<E>,
    subscription: crate::subscription::Subscription,
) -> anyhow::Result<()> {
    let Some(customer_id) = subscription.customer_id else {
        anyhow::bail!(
            "failed to renew user subscription, no customer_id found for {}",
            &subscription.id
        );
    };

    let Some(payment_method_id) = subscription.payment_method_id else {
        anyhow::bail!(
            "failed to renew user subscription, no payment_method_id found for {}",
            &subscription.id
        );
    };

    let Some(payment_details) = subscription.payment_details else {
        anyhow::bail!(
            "failed to renew user subscription, no payment_details found for {}",
            &subscription.id
        );
    };

    let amount = payment_details.price + payment_details.tax;

    let payment_intent = CreatePaymentIntent::new(amount, Currency::USD)
        .expand(&["payment_method".to_owned()])
        .customer(customer_id)
        .payment_method(payment_method_id)
        .off_session(CreatePaymentIntentOffSession::Bool(true))
        .confirm(true)
        .send(stripe)
        .await?;

    command
        .update_stripe_payment_intent_status(payment_intent, &subscription.id)
        .await?;

    Ok(())
}

pub fn shed_subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("user-scheduler")
        .handler(handle_stripe_payment_intent_succeeded())
        .handler(handle_cancelled())
}

#[evento::subscription]
async fn handle_stripe_payment_intent_succeeded<E: Executor>(
    context: &Context<'_, E>,
    event: Event<StripePaymentIntentSucceeded>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(UserSubscription::Table)
        .columns([UserSubscription::Id, UserSubscription::ExpireAt])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.expire_at.into(),
        ])
        .on_conflict(
            OnConflict::column(UserSubscription::Id)
                .update_column(UserSubscription::ExpireAt)
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::subscription]
async fn handle_cancelled<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Cancelled>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::delete()
        .from_table(UserSubscription::Table)
        .and_where(Expr::col(UserSubscription::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

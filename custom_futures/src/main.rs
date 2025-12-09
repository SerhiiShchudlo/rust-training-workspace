use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

///
/// sleeps on a dedicated thread, then finishes the future
///
struct Sleep {
    // ...
}

impl Sleep {
    fn new(duration: Duration) -> Self {
        todo!()
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

///
/// finishes when any future finishes and returns its result
///
struct Race<F1, F2> {
    future1: F1,
    future2: F2,
}

impl<F1, F2> Race<F1, F2> {
    fn new(future1: F1, future2: F2) -> Self {
        Self { future1, future2 }
    }
}

impl<T, F1, F2> Future for Race<F1, F2>
where
    F1: Future<Output = T>,
    F2: Future<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

///
/// finishes when all futures finish
///
struct Join {
    // ...
}

impl Join {
    fn new() -> Self {
        todo!()
    }

    fn push(&mut self, future: impl Future<Output = ()>) -> Self {
        todo!()
    }
}

/// `anything.await` will always call `IntoFuture::into_future(anything).await` under the hood
/// this is similar to how `for` and `IntoIterator` works
/// each future implements IntoFuture automatically
impl IntoFuture for Join {
    type Output = ();
    type IntoFuture = JoinFuture;

    fn into_future(self) -> Self::IntoFuture {
        todo!()
    }
}

struct JoinFuture {
    // ...
}

impl Future for JoinFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn sleep() {
        let instant = Instant::now();

        Sleep::new(Duration::from_secs(1)).await;

        assert!(instant.elapsed() < Duration::from_secs_f32(1.5));
    }

    #[tokio::test]
    async fn race1() {
        let instant = Instant::now();
        let mut future1_finished = false;
        let mut future2_finished = false;

        let future1 = async {
            Sleep::new(Duration::from_secs(1)).await;
            future1_finished = true;
            13
        };
        let future2 = async {
            Sleep::new(Duration::from_secs(2)).await;
            future2_finished = true;
            42
        };

        let result = Race::new(future1, future2).await;
        assert_eq!(result, 13);

        assert!(future1_finished);
        assert!(!future2_finished);
        assert!(instant.elapsed() < Duration::from_secs_f32(1.5));
    }

    #[tokio::test]
    async fn race2() {
        let instant = Instant::now();
        let mut future1_finished = false;
        let mut future2_finished = false;

        let future1 = async {
            Sleep::new(Duration::from_secs(2)).await;
            future1_finished = true;
            13
        };
        let future2 = async {
            Sleep::new(Duration::from_secs(1)).await;
            future2_finished = true;
            42
        };

        let result = Race::new(future1, future2).await;
        assert_eq!(result, 42);

        assert!(!future1_finished);
        assert!(future2_finished);
        assert!(instant.elapsed() < Duration::from_secs_f32(1.5));
    }

    #[tokio::test]
    async fn join() {
        let instant = Instant::now();
        let mut future1_finished = false;
        let mut future2_finished = false;

        let mut join = Join::new();
        join.push(async {
            Sleep::new(Duration::from_secs(1)).await;
            future1_finished = true;
        });
        join.push(async {
            Sleep::new(Duration::from_secs(2)).await;
            future2_finished = true;
        });
        join.await;

        assert!(future1_finished);
        assert!(future2_finished);
        assert!(instant.elapsed() < Duration::from_secs_f32(2.5));
    }
}
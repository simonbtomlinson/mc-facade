use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub enum RaceResult<U, V>
where
    U: Future,
    V: Future,
{
    Left(U::Output),
    Right(V::Output),
}

pin_project! {
    pub struct RaceFuture<U, V>
    where U : Future, V : Future
    {
        #[pin]
        left: U,
        #[pin]
        right: V
    }
}

impl<U, V> Future for RaceFuture<U, V>
where
    U: Future,
    V: Future,
{
    type Output = RaceResult<U, V>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Poll::Ready(val) = this.left.poll(cx) {
            return Poll::Ready(RaceResult::Left(val));
        }

        if let Poll::Ready(val) = this.right.poll(cx) {
            return Poll::Ready(RaceResult::Right(val));
        }

        Poll::Pending
    }
}

pub fn race<U: Future, V: Future>(left: U, right: V) -> RaceFuture<U, V> {
    RaceFuture { left, right }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_race_left() {
        let left = sleep(Duration::from_millis(5));
        let right = sleep(Duration::from_millis(10));
        let result = race(left, right).await;
        match result {
            RaceResult::Left(_) => (),
            RaceResult::Right(_) => panic!("Expected left side to win the race"),
        }
    }

    #[tokio::test]
    async fn test_race_oneshot() {
        use tokio::sync::oneshot;
        let (_tx1, rx1) = oneshot::channel::<i64>();
        let (tx2, rx2) = oneshot::channel::<i64>();
        tokio::spawn(async move { tx2.send(1) });
        match race(rx1, rx2).await {
            RaceResult::Left(_) => panic!("expected right side to win the race"),
            RaceResult::Right(n) => assert_eq!(n, Ok(1)),
        }
    }
}

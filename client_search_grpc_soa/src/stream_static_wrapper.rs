use std::pin::Pin;
use tokio_stream::Stream;
use std::task::{Context, Poll};

// use futures::{
//     prelude::*,
//     task::{Context, Poll},
// };
use pin_project_lite::pin_project;
//--------------------------------------------------------------------------------------------------


pin_project! {
    pub struct FooStream<St>
    where
        St: Stream,
    {
        #[pin]
        inner: St,
    }
}
impl<St: Stream> Stream for FooStream<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

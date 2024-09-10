use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;
//--------------------------------------------------------------------------------------------------



// Code from https://www.reddit.com/r/rust/comments/f8vc1v/how_can_i_create_a_future_value/

#[allow(dead_code)]
fn into_fut<T>(value: T) -> impl Future<Output=T> where T: Unpin {
    ValueFuture(Some(value))
}


struct ValueFuture<Data>(Option<Data>);

impl<Data> Future for ValueFuture<Data> where Data: Unpin {
    type Output = Data;

    /// #Note
    /// this panics if poll is called twice, but most executors will not
    /// do that or require that the future is 'fused'
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        Poll::Ready(self.get_mut().0.take().unwrap())
    }
}


// fn to_future<Data>(data: Data) -> impl Future<Output = Data> {
//     async { data }
// }

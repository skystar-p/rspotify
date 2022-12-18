//! Asynchronous implementation of automatic pagination requests.

use crate::{model::Page, ClientResult};

use std::pin::Pin;

use futures::{future::Future, stream::Stream};

/// Alias for `futures::stream::Stream<Item = T>`, since async mode is enabled.
pub type Paginator<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a + Send>>;

pub type RequestFuture<'a, T> = Pin<Box<dyn 'a + Future<Output = ClientResult<Page<T>>> + Send>>;

/// This is used to handle paginated requests automatically.
pub fn paginate_with_ctx<'a, Ctx, T, Request>(
    ctx: Ctx,
    req: Request,
    page_size: u32,
) -> Paginator<'a, ClientResult<T>>
where
    T: 'a + Unpin + Send,
    Ctx: 'a + Send + Sync,
    Request: 'a + for<'ctx> Fn(&'ctx Ctx, u32, u32) -> RequestFuture<'ctx, T> + Send + Sync,
{
    use async_stream::stream;
    let mut offset = 0;
    Box::pin(stream! {
        loop {
            let page = req(&ctx, page_size, offset).await?;
            offset += page.items.len() as u32;
            for item in page.items {
                yield Ok(item);
            }
            if page.next.is_none() {
                break;
            }
        }
    })
}

pub fn paginate<'a, T, Fut, Request>(req: Request, page_size: u32) -> Paginator<'a, ClientResult<T>>
where
    T: 'a + Unpin + Send,
    Fut: Future<Output = ClientResult<Page<T>>> + Send,
    Request: 'a + Fn(u32, u32) -> Fut + Send + Sync,
{
    use async_stream::stream;
    let mut offset = 0;
    Box::pin(stream! {
        loop {
            let page = req(page_size, offset).await?;
            offset += page.items.len() as u32;
            for item in page.items {
                yield Ok(item);
            }
            if page.next.is_none() {
                break;
            }
        }
    })
}

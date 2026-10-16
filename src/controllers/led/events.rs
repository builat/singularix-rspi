use std::time::Duration;

use crate::led::engine::LedEngineHandle;
use actix_web::{Error, web};

use actix_web_lab::sse;
use futures::Stream;

pub async fn controller(
    engine: web::Data<LedEngineHandle>,
) -> Result<sse::Sse<impl Stream<Item = Result<sse::Event, Error>>>, Error> {
    let mut rx = engine.subscribe();
    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            yield Ok::<sse::Event, actix_web::Error>(
                sse::Event::Data(sse::Data::new(msg))
            );
        }
    };

    Ok(sse::Sse::from_stream(stream).with_keep_alive(Duration::from_secs(15)))
}

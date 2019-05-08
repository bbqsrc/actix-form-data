use bytes::{Bytes, BytesMut};
use failure::Fail;
use futures::{
    sync::mpsc::{channel, SendError},
    task, Async, AsyncSink, Future, Poll, Sink, StartSend, Stream,
};
use std::{
    fs::File,
    io::{Error, Write},
    path::Path,
};

#[derive(Clone, Debug, Fail)]
#[fail(display = "Error in Channel")]
struct ChannelError;

pub fn write(
    filename: impl AsRef<Path> + Clone + Send + 'static,
) -> impl Sink<SinkItem = Bytes, SinkError = SendError<Bytes>> {
    let (tx, rx) = channel(50);

    actix_rt::spawn(
        actix_threadpool::run(move || {
            CreateFuture::new(filename.clone())
                .from_err()
                .and_then(|file| {
                    rx.map_err(|_| failure::Error::from(ChannelError))
                        .forward(WriteSink::new(file))
                })
                .wait()
        })
        .map_err(|_| ())
        .map(|_| ()),
    );

    tx
}

struct CreateFuture<P>(P)
where
    P: AsRef<Path> + Clone;

impl<P> CreateFuture<P>
where
    P: AsRef<Path> + Clone,
{
    fn new(path: P) -> Self {
        CreateFuture(path)
    }
}

impl<P> Future for CreateFuture<P>
where
    P: AsRef<Path> + Clone,
{
    type Item = File;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        File::create(self.0.clone()).map(Async::Ready)
    }
}

struct WriteSink {
    buffer: BytesMut,
    file: File,
}

impl WriteSink {
    fn new(file: File) -> Self {
        WriteSink {
            buffer: BytesMut::new(),
            file,
        }
    }
}

impl Sink for WriteSink {
    type SinkItem = Bytes;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if let Async::NotReady = self.poll_complete()? {
            return Ok(AsyncSink::NotReady(item));
        }

        self.buffer = BytesMut::new();
        self.buffer.extend_from_slice(&item);

        self.poll_complete()?;

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        if self.buffer.is_empty() {
            return Ok(Async::Ready(()));
        }

        let written = self.file.write(&self.buffer)?;
        if written == 0 {
            return Err(Error::last_os_error());
        }
        self.buffer.advance(written);

        if self.buffer.is_empty() {
            Ok(Async::Ready(()))
        } else {
            task::current().notify();
            Ok(Async::NotReady)
        }
    }
}

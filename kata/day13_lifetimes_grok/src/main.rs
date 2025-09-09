use bytes::Bytes;
use futures::{stream, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::io::StreamReader;

// A fake stream that yields exactly 2 chunks
struct MyStream {
    step: usize,
}

impl Stream for MyStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.step {
            0 => {
                self.step += 1;
                // First chunk has 6 bytes
                Poll::Ready(Some(Ok(Bytes::from_static(b"abcdef"))))
            }
            1 => {
                self.step += 1;
                // Second chunk has 6 bytes
                Poll::Ready(Some(Ok(Bytes::from_static(b"ghijkl"))))
            }
            _ => Poll::Ready(None),
        }
    }
}

#[tokio::main]
async fn main() {
    let stream = MyStream { step: 0 };
    let mut reader = StreamReader::new(stream);

    let mut buf = [0u8; 3];

    // First read — takes "abc" from first chunk ("def" is unused)
    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 1: {:?}", &buf[..n]);

    // Second read — should ideally get "def", but actually gets "ghi"
    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 2: {:?}", &buf[..n]);

    // Third read — should ideally get "ghi", but actually gets "ghi"
    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 3: {:?}", &buf[..n]);

    use tokio::io::BufReader;

    let stream = MyStream { step: 0 };
    let mut reader = BufReader::new(StreamReader::new(stream));

    let mut buf = [0u8; 3];

    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 1: {:?}", &buf[..n]);

    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 2: {:?}", &buf[..n]);

    let n = reader.read(&mut buf).await.unwrap();
    println!("Read 3: {:?}", &buf[..n]);
    
}


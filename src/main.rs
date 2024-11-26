use std::future::Future;
use futures_lite::future::{zip};
use futures_lite::io::AsyncReadExt;
use futures_lite::{AsyncWriteExt, StreamExt};
use glommio::net::{TcpListener, TcpStream};
use glommio::{timer::sleep, LocalExecutor, LocalExecutorBuilder, Placement};
use reinterpret::reinterpret_mut_slice;
use ringbuf::{consumer::Consumer, producer::Producer, traits::Observer, SharedRb};
use ringbuf::HeapRb;
use std::io::{Result, Error, IoSliceMut};
use std::mem::MaybeUninit;
use std::time::Duration;
use ringbuf::storage::Heap;
use futures_lite::FutureExt;

async fn main_async() -> Result<()> {
    let ex = async_executor::LocalExecutor::new();

    let foo = async {
        let mut listener = TcpListener::bind("127.0.0.1:10000")?;
        loop {
            let client = listener.accept().await?;
            ex.spawn(async move {
                let mut client = client;
                let mut read_buffer = HeapRb::<u8>::new({8 * 1024});
                loop {
                    read_to_buffer(&mut client, &mut read_buffer).await?;

                    println!("{}", read_buffer.occupied_len());

                    let (foo, bar) = read_buffer.as_slices();
                    unsafe { read_buffer.advance_read_index(read_buffer.occupied_len()) }
                }

                Ok::<(), Error>(())
            }).detach()
        }

        Ok::<(), Error>(())
    };

    let qux = async move{
        let client = TcpStream::connect("127.0.0.1:10000").await?;
        let mut client = client.buffered();


        let buf = [0u8; 512];
        client.write(&buf).await?;
        loop {
            let buf = [255u8; 1024];
            client.write(&buf).await?;
            sleep(Duration::from_millis(100)).await;
        }
        Ok::<(), Error>(())
    };

    foo.or(qux).or(async { loop { ex.tick().await } }).await?;

    Ok(())
}

async fn read_to_buffer(stream: &mut TcpStream, read_buffer: &mut SharedRb<Heap<u8>>) -> Result<()> {
    unsafe {
        let foo = read_buffer.vacant_slices_mut();
        //let foo: &mut [u8] = reinterpret_mut_slice(foo);
        //let bar: &mut [u8] = reinterpret_mut_slice(bar);



        let mut bar = reinterpret_mut_slice(foo.0);
        let mut baz = reinterpret_mut_slice(foo.1);

        bar.fill(0);
        baz.fill(0);

        let advance = {
            let mut bar = IoSliceMut::new(bar);
            let mut baz = IoSliceMut::new(baz);

            println!("Bar is {} sized", bar.len());

            stream.read_vectored(&mut [bar, baz]).await?
        };
        read_buffer.advance_write_index(advance);
    };
    Ok(())
}

fn main() -> Result<()> {
    let executor = LocalExecutor::default();

    executor.run(async {
        main_async().await
    })
}

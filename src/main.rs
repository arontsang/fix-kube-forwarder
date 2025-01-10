use futures_lite::io::AsyncReadExt;
use futures_lite::FutureExt;
use futures_lite::{AsyncWriteExt};
use glommio::net::{TcpListener, TcpStream};
use glommio::{timer::sleep, LocalExecutor};
use reinterpret::reinterpret_mut_slice;
use ringbuf::storage::Heap;
use ringbuf::HeapRb;
use ringbuf::{consumer::Consumer, producer::Producer, traits::Observer, SharedRb};
use std::io::{Error, IoSliceMut, Result};
use std::{mem, str};
use std::time::Duration;
use twoway::find_bytes;


const FIX_CHECKSUM_TAG: &[u8; 4] = b"\x0110=";

async fn main_async() -> Result<()> {
    let ex = async_executor::LocalExecutor::new();

    let foo = async {
        let listener = TcpListener::bind("0.0.0.0:10000")?;
        loop {
            let client = listener.accept().await?;
            ex.spawn(async move {
                let mut client = client;
                let mut read_buffer = HeapRb::<u8>::new({8 * 1024});
                loop {
                    read_to_buffer(&mut client, &mut read_buffer).await?;

                    println!("{}", read_buffer.occupied_len());

                    let (packet, _) = read_buffer.as_slices();

                    let Some(index) = find_bytes(packet, FIX_CHECKSUM_TAG) else { continue };
                    let index_end_msg = {
                        let tail = &packet[index..][size_of_val(FIX_CHECKSUM_TAG)..];
                        tail.iter().position(|&x| x == 0x01)
                    };
                    let Some(index_end_msg) = index_end_msg else { continue };
                    let index_end_msg = index_end_msg + index + size_of_val(FIX_CHECKSUM_TAG) + 1;

                    let log_on_message = &packet[0..index_end_msg];
                    unsafe { read_buffer.advance_read_index(index) }

                    let target_comp_id = {
                        const TARGET_COMP_ID_PREAMBLE: &[u8; 4] = b"\x0156=";
                        let comp_id_start_position = find_bytes(log_on_message, TARGET_COMP_ID_PREAMBLE).unwrap();
                        let tail = &log_on_message[comp_id_start_position..][size_of_val(TARGET_COMP_ID_PREAMBLE)..];
                        let end_comp_id = tail.iter().position(|&x| x == 0x01).unwrap();
                        let comp_id = &tail[0..end_comp_id];
                        let comp_id = str::from_utf8(comp_id).unwrap();
                        comp_id
                    };

                    println!("{}", target_comp_id);
                }

                Ok::<(), Error>(())
            }).detach()
        }

        Ok::<(), Error>(())
    };

    // let qux = async move{
    //     let client = TcpStream::connect("127.0.0.1:10000").await?;
    //     let mut client = client.buffered();
    //
    //
    //     let buf = [0u8; 512];
    //     client.write(&buf).await?;
    //     loop {
    //         let buf = [255u8; 1024];
    //         client.write(&buf).await?;
    //         sleep(Duration::from_millis(100)).await;
    //     }
    // };

    foo
        //.or(qux)
        .or(async { loop { ex.tick().await } }).await?;

    Ok(())
}

async fn read_to_buffer(stream: &mut TcpStream, read_buffer: &mut SharedRb<Heap<u8>>) -> Result<()> {
    unsafe {
        let foo = read_buffer.vacant_slices_mut();
        //let foo: &mut [u8] = reinterpret_mut_slice(foo);
        //let bar: &mut [u8] = reinterpret_mut_slice(bar);



        let bar = reinterpret_mut_slice(foo.0);
        let baz = reinterpret_mut_slice(foo.1);

        bar.fill(0);
        baz.fill(0);

        let advance = {
            let bar = IoSliceMut::new(bar);
            let baz = IoSliceMut::new(baz);

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

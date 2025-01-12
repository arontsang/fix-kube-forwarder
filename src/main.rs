
use futures_lite::FutureExt;
use glommio::net::{TcpListener, TcpStream};
use glommio::{GlommioError, LocalExecutor};
use reinterpret::reinterpret_mut_slice;
use std::io::{Error, Result};
use std::str;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use twoway::find_bytes;


const FIX_CHECKSUM_TAG: &[u8; 4] = b"\x0110=";

async fn main_async() -> Result<()> {
    let ex = async_executor::LocalExecutor::new();

    let accept_client_task = async {
        let listener = TcpListener::bind("0.0.0.0:10000")?;
        loop {
            let Ok(mut client) = listener.accept().await else { break; };
            ex.spawn(async move {
                handle_client(& mut client).await }
            ).detach()
        }
        Ok::<(), Error>(())
    };

    let handle_client_task = async {
        loop {
            ex.tick().await
        }
    };

    accept_client_task
        .or(handle_client_task).await?;

    Ok(())
}

async fn handle_client(client: &mut TcpStream) {
    let mut read_buffer = Vec::<u8>::with_capacity(1024);
    let work: core::result::Result<(), Error> = {
        loop {
            unsafe  {
                let buf = read_buffer.spare_capacity_mut();
                let buf: &mut [u8] = reinterpret_mut_slice(buf);
                let Ok(read_size) = client.read(buf).await else { break };
                if read_size == 0 { break; }

                read_buffer.set_len(read_buffer.len() + read_size);
            };
            let packet = read_buffer.as_slice();

            let Some(index) = find_bytes(packet, FIX_CHECKSUM_TAG) else { continue };
            let index_end_msg = {
                let tail = &packet[index..][size_of_val(FIX_CHECKSUM_TAG)..];
                tail.iter().position(|&x| x == 0x01)
            };
            let Some(index_end_msg) = index_end_msg else { continue };
            let index_end_msg = index_end_msg + index + size_of_val(FIX_CHECKSUM_TAG) + 1;

            let log_on_message = &packet[0..index_end_msg];
            let target_comp_id = {
                const TARGET_COMP_ID_PREAMBLE: &[u8; 4] = b"\x0156=";
                let comp_id_start_position = find_bytes(log_on_message, TARGET_COMP_ID_PREAMBLE).unwrap();
                let tail = &log_on_message[comp_id_start_position..][size_of_val(TARGET_COMP_ID_PREAMBLE)..];
                let end_comp_id = tail.iter().position(|&x| x == 0x01).unwrap();
                let comp_id = &tail[0..end_comp_id];
                let comp_id = str::from_utf8(comp_id).unwrap();
                comp_id
            };

            let Ok(mut fix_acceptor) = get_proxy_target(target_comp_id).await else { break; };

            async fn copy_stream<TRead: AsyncRead + Unpin, TWrite: AsyncWrite + Unpin>(mut source: TRead, mut target: TWrite) {
                let mut buffer = [0u8; 1024 * 8];
                let buffer = &mut buffer;
                loop {
                    let Ok(read_size) = source.read(buffer).await else { break };
                    if read_size == 0 { break; }

                    let buffer = &buffer[..read_size];
                    if !target.write(buffer).await.is_ok() {
                        break;
                    }
                }
            }
            
            fix_acceptor.write(packet).await.expect("TODO: panic message");

            let (client_read, client_write) = client.split();
            let (acceptor_read, acceptor_write) = fix_acceptor.split();

            let copy_to_acceptor = copy_stream(client_read, acceptor_write);
            let copy_from_acceptor = copy_stream(acceptor_read, client_write);

            copy_from_acceptor.or(copy_to_acceptor).await
        };
        Ok(())
    };

    work.unwrap_or_default()
}

async fn get_proxy_target(_target_comp_id: &str) -> std::result::Result<TcpStream, GlommioError<()>> {
    TcpStream::connect("172.20.8.3:9880").await
}

fn main() -> Result<()> {
    let executor = LocalExecutor::default();

    executor.run(async {
        main_async().await
    })
}

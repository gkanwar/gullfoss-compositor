// Library for clients to talk to libcompositor (and vice versa). Currently
// implemented using unix sockets in this emulator stage. IPC could be replaced.

use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::thread;

const DEFAULT_SOCK_PATH: &str = "/tmp/gfcomp_sock";

#[derive(Serialize, Deserialize)]
enum Message {
  Hello // FORNOW: just a dummy
}

fn check_hello(msg: Message) {
  match msg {
    Message::Hello => {
      println!("Got Hello");
    }
    _ => {
      panic!("Interlocutor is speaking an unexpected protocol");
    }
  }
}

fn send_message(msg: Message, stream: &mut UnixStream) -> std::io::Result<()> {
  let encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
  let l: usize = encoded.len();
  let l_encoded: Vec<u8> = bincode::serialize(&l).unwrap();
  assert_eq!(l_encoded.len(), 4);
  stream.write_all(&l_encoded[..])?;
  stream.write_all(&encoded[..])?;
  return Ok(());
}
fn recv_message(stream: &mut UnixStream) -> std::io::Result<Message> {
  let mut l_encoded: Vec<u8> = Vec::with_capacity(4);
  stream.read(&mut l_encoded[..]);
  let l: usize = bincode::deserialize(&l_encoded[..]).unwrap();
  let mut encoded: Vec<u8> = Vec::with_capacity(l);
  stream.read(&mut encoded[..]);
  let msg: Message = bincode::deserialize(&encoded[..]).unwrap();
  return Ok(msg);
}

fn server_thread(mut stream: UnixStream) -> std::io::Result<()> {
  send_message(Message::Hello, &mut stream)?;
  check_hello(recv_message(&mut stream)?);
  return Ok(());
}

fn bind_unix_listener() -> std::io::Result<()> {
  let listener = UnixListener::bind(DEFAULT_SOCK_PATH)?;
  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        thread::spawn(|| server_thread(stream));
      }
      Err(_) => {
        break;
      }
    }
  }
  return Ok(());
}

fn client_thread(
  mut stream: UnixStream, msg_queue: mpsc::Receiver<Message>
) -> std::io::Result<()> {
  check_hello(recv_message(&mut stream)?);
  send_message(Message::Hello, &mut stream)?;
  let (in_msg_sender, in_msg_queue) = mpsc::channel();
  let mut stream_reader = stream.try_clone().unwrap();
  thread::spawn(move || -> std::io::Result<()> {
    loop {
      let m = recv_message(&mut stream_reader)?;
      in_msg_sender.send(m).unwrap();
    }
  });
  loop {
    if let Ok(m) = msg_queue.try_recv() {
      send_message(m, &mut stream)?;
    }
    if let Ok(_m) = in_msg_queue.try_recv() {}
  }
}

fn connect_to_server(msg_queue: mpsc::Receiver<Message>) -> () {
  match UnixStream::connect(DEFAULT_SOCK_PATH) {
    Ok(sock) => {
      thread::spawn(move || client_thread(sock, msg_queue));
    }
    Err(e) => {
      println!("Failed to connect: {:?}", e);
      return;
    }
  };
}

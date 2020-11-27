use std::{io, thread};
use std::{process::Command, time::Duration};

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame, Message},
    BoxedSocket, Client,
};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};

const SERVER_URL: &str = "wss://fourinarow.ml/game/";

pub fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    loop {
        let sys = System::new("websocket-client");
        Arbiter::spawn(async {
            let (response, framed) = Client::new()
                .ws(SERVER_URL)
                .connect()
                .await
                .map_err(|e| {
                    println!("Error: {}", e);
                })
                .unwrap();

            println!("{:?}", response);
            let (sink, stream) = framed.split();
            let addr = ChatClient::create(|ctx| {
                ChatClient::add_stream(stream, ctx);
                ChatClient(SinkWrite::new(sink, ctx))
            });

            // start console input loop
            thread::spawn(move || loop {
                let mut cmd = String::new();
                if io::stdin().read_line(&mut cmd).is_err() {
                    println!("error");
                    return;
                }
                addr.do_send(ClientCommand(cmd));
            });
        });
        sys.run().unwrap();
    }
}

struct ChatClient(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);

impl Actor for ChatClient {
    type Context = Context<Self>;

    // fn started(&mut self, ctx: &mut Context<Self>) {
    //     // start heartbeats otherwise server will disconnect after 10 seconds
    // }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl ChatClient {
    fn hb(&mut self) {
        self.0.write(Message::Ping(Bytes::default()));
    }
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        print!("\x1B[F");
        print_time(Direction::Out);
        print!("{}", msg.0);
        self.0.write(Message::Text(msg.0.trim().to_string()));
    }
}

enum Direction {
    In,
    Out,
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
        match msg {
            Ok(Frame::Text(txt)) => {
                print!("\x1B[{}D", 1024);
                print_time(Direction::In);
                if let Ok(msg) = String::from_utf8(txt.into_iter().collect()) {
                    if msg.starts_with("CURRENT_SERVER_STATE") {
                        let parts: Vec<&str> = msg.split(":").clone().collect();
                        print!("{} online", parts[1],);
                        if parts[2] == "true" {
                            print!(" - someone in queue!");
                            notify("Someone is in queue!");
                        }
                        println!();
                    } else {
                        println!("{:?}", msg);
                    }
                }
                print!("<< ");
                flush();
            }
            Ok(Frame::Ping(bytes)) => {
                self.0.write(Message::Pong(bytes));
            }
            _ => {}
        }
    }

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Connected");
        ctx.run_interval(Duration::new(1, 0), |act, _ctx| {
            act.hb();
        });
    }

    fn finished(&mut self, _ctx: &mut Context<Self>) {
        println!("Server disconnected");
        // reconnect_delayed();
    }

    // fn reconnect_delayed(&mut self, ctx: &mut Context<Self>) {
    //     ctx.run_later(
    //         std::time::Duration::from_secs(10 * self.connection_retries),
    //         |act, ctx| {
    //             if reconnect().is_err() {
    //                 act.connection_retries += 1;
    //                 act.reconnect_delayed();
    //             }
    //         },
    //     );
    // }

    // fn reconnect() -> Result<(), ()> {
    //     Ok(())
    // }
}

fn notify(msg: &str) {
    let cmdres = Command::new("./notify.sh").arg(msg).output();

    if let Err(e) = cmdres {
        println!("Failed to notify: {:?}", e);
    } else if let Ok(c) = cmdres {
        if !c.status.success() {
            println!("Error notifying. {:?}", c);
        }
    };
}

fn print_time(dir: Direction) {
    let date = chrono::Local::now();
    print!(
        "{} [{}]: ",
        match dir {
            Direction::In => ">>",
            Direction::Out => "<<",
        },
        date.format("%d.%m.%y - %H:%M:%S")
    );
}

fn flush() {
    use std::io::Write;
    let _ = std::io::stdout().lock().flush();
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}

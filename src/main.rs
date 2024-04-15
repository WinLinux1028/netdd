use std::{env, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use tokio::{
    fs::File,
    io::{AsyncBufRead, BufReader},
    net::{TcpListener, TcpStream},
};

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() {
    let mut options = CommandOptions {
        input: None,
        output: None,
    };

    let mut args = env::args();
    args.next();

    for i in args {
        if let Some(s) = i.strip_prefix("if=") {
            options.input = Some(ifof_option(s).unwrap().into());
        } else if let Some(s) = i.strip_prefix("of=") {
            options.output = Some(ifof_option(s).unwrap().into());
        } else {
            help();
        }
    }

    let (input, output) = match (options.input, options.output) {
        (Some(i), Some(o)) => (Arc::new(i), o),
        _ => help(),
    };

    match output {
        Output::Path(path) => {
            let mut output = File::create(path).await.unwrap();
            let mut input = input.io_start().await.unwrap();
            tokio::io::copy_buf(&mut input, &mut output).await.unwrap();
            output.sync_all().await.unwrap();
        }
        Output::SendingServer(addr) => {
            let listener = TcpListener::bind(addr).await.unwrap();
            loop {
                let (mut output, _) = match listener.accept().await {
                    Ok(o) => o,
                    Err(_) => continue,
                };
                let input = Arc::clone(&input);
                tokio::spawn(async move {
                    let mut input = match input.io_start().await {
                        Ok(o) => o,
                        Err(_) => return,
                    };
                    let _ = tokio::io::copy_buf(&mut input, &mut output).await;
                });
            }
        }
    }
}

fn ifof_option(option: &str) -> Result<InputOutput, Error> {
    if let Some(s) = option.strip_prefix("netdd://") {
        Ok(InputOutput::SocketAddr(SocketAddr::from_str(s)?))
    } else {
        Ok(InputOutput::Path(PathBuf::from(option)))
    }
}

fn help() -> ! {
    println!("使い方: netdd [オプション]...");
    println!("オプション一覧");
    println!("if=/path/to/file : ファイルから入力します");
    println!("if=netdd://114.51.48.10:4545 : サーバーからデータを受信します");
    println!("of=/path/to/file : ファイルに出力します");
    println!("of=netdd://0.0.0.0:4545 : サーバーとして動作し、クライアントにデータを送ります");

    std::process::exit(1);
}

struct CommandOptions {
    input: Option<Input>,
    output: Option<Output>,
}

enum InputOutput {
    Path(PathBuf),
    SocketAddr(SocketAddr),
}

impl From<InputOutput> for Input {
    fn from(value: InputOutput) -> Self {
        match value {
            InputOutput::Path(p) => Self::Path(p),
            InputOutput::SocketAddr(s) => Self::ReceivingClient(s),
        }
    }
}

impl From<InputOutput> for Output {
    fn from(value: InputOutput) -> Self {
        match value {
            InputOutput::Path(p) => Self::Path(p),
            InputOutput::SocketAddr(s) => Self::SendingServer(s),
        }
    }
}

enum Input {
    Path(PathBuf),
    ReceivingClient(SocketAddr),
}

impl Input {
    async fn io_start(&self) -> Result<Box<dyn AsyncBufRead + Unpin + Send>, Error> {
        if let Input::Path(path) = self {
            let file = BufReader::new(File::open(path).await?);
            Ok(Box::new(file))
        } else if let Input::ReceivingClient(addr) = self {
            let stream = BufReader::new(TcpStream::connect(addr).await?);
            Ok(Box::new(stream))
        } else {
            panic!("不明なエラー");
        }
    }
}

enum Output {
    Path(PathBuf),
    SendingServer(SocketAddr),
}

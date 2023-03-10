use std::{net::{TcpListener, TcpStream}, any::Any, io::{self, Write}, time::Duration, thread, io::Read};

use anyhow::bail;
use log::info;

use super::*;

const TCP_SERVER_KEY: &str = concat!("microservice-rs/", file!(), ":", line!());
const SOME_MODULE_KEY: &str = concat!("microservice-rs/", file!(), ":", line!());

struct TcpServer {
    socket: TcpListener,
    routes: Vec<&'static str>,
}
impl TcpServer {
    fn key() -> &'static str { TCP_SERVER_KEY }
    fn register(collector: &mut ModuleCollector) -> anyhow::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        Self::register_with_args(listener, collector)
    }
    fn register_with_args(listener: TcpListener, collector: &mut ModuleCollector) -> anyhow::Result<()> {
        collector.register(Self::key(), Self{socket: listener, routes: vec!()}, TcpServerConfig{});
        Ok(())
        
    }
    fn add_route(&mut self, path: &'static str) {
        self.routes.push(path)
    }
}
struct TcpServerConfig {}

impl Configurator for TcpServerConfig {
    fn configure(
            &mut self,
            module: Box<dyn Any>,
            binder: &mut ModuleBinder,
            server: &mut ServerHealth,
        ) -> anyhow::Result<()> {
        let module: TcpServer = *module.downcast().unwrap();
        server.register_on_healthy(move |ctx| {
            let socket = module.socket;
            socket.set_nonblocking(true)?;
            loop {
                match socket.accept() {
                    Ok(mut s) => {
                        s.0.write(module.routes[0].as_bytes()).unwrap();
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // do nothing
                    },
                    Err(e) => bail!("error waiting for socket: {:?}", e),
                };
                if ctx.is_done() {
                    break;
                }
                // sleep for a moment
                thread::sleep(Duration::from_millis(50));
            }
            Ok(())
        });
        Ok(())
    }

    fn depends_on(&self) -> Vec<&'static str> {
        vec!()
    }
}

struct SomeModule {
    msg: &'static str,
}
impl SomeModule {
    fn key() -> &'static str { SOME_MODULE_KEY }
    fn register(message: &'static str, collector: &mut ModuleCollector) -> anyhow::Result<()> {
        collector.register(Self::key(), Self{msg: message}, SomeModuleConfig{});
        Ok(())
    }
}
struct SomeModuleConfig {}

impl Configurator for SomeModuleConfig {
    fn configure(
            &mut self,
            module: Box<dyn Any>,
            binder: &mut ModuleBinder,
            server: &mut ServerHealth,
        ) -> anyhow::Result<()> {
        let path = module.downcast::<SomeModule>().unwrap().msg;
        let tcp_server: &mut TcpServer = binder.get(TcpServer::key())?;
        tcp_server.add_route(path);
        Ok(())
    }
    fn depends_on(&self) -> Vec<&'static str> {
        vec!(TcpServer::key())
    }
}


#[test]
fn sanity_test() {
    let msg = "Hello world!";
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let local = listener.local_addr().unwrap();
    let mut binder = ModuleCollector::new();
    TcpServer::register_with_args(listener, &mut binder).unwrap();
    SomeModule::register(msg, &mut binder).unwrap();

    let ctx = Context::new();
    let ctx2 = ctx.clone();
    let jh = thread::spawn(move || {
        binder.start(ctx2)
    });

    // Make sure we can connect, and expect the response from module
    let mut client = TcpStream::connect(local).unwrap();
    let mut buff = vec![0u8; 1024];
    let count = client.read(&mut buff).unwrap();

    assert_eq!(&buff[..count], msg.as_bytes());

    ctx.cancel();
    let result = jh.join();
    assert!(result.is_ok());
}
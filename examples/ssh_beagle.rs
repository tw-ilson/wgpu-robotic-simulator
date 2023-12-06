fn main() {
    use std::io::prelude::*;
    use std::net::TcpStream;
    use ssh2::Session;

    // Connect to the local SSH server
    let tcp = TcpStream::connect("beaglebone.local:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password("debian", "temppwd").unwrap();
    assert!(sess.authenticated());
    
    let mut channel = sess.channel_session().unwrap();
    channel.exec("ls").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    println!("{}", s);
    channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
}

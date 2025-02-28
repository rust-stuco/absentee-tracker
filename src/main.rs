use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use base64::encode;
use dotenv::dotenv;
use config::Config;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct SmtpConfig {
    smtp: SmtpDetails,
}

#[derive(Debug, Deserialize)]
struct SmtpDetails {
    sender: String,
    receiver: String,
}

fn load_config() -> Result<SmtpDetails, Box<dyn std::error::Error>> {
    // Load configuration from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    // Deserialize the configuration into the SmtpConfig struct
    let smtp_config: SmtpConfig = settings.try_deserialize()?;

    Ok(smtp_config.smtp)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Load SMTP configuration
    let config = load_config()?;
    println!("Loaded config: {:?}", config);

    // Load SMTP password from environment variable
    let password = env::var("SMTP_PASSWORD")?;

    // Connect to the SMTP server (e.g., Gmail's SMTP server)
    let mut stream = TcpStream::connect("smtp.gmail.com:587")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Read the server's welcome message
    let mut response = [0; 512];
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send EHLO command
    stream.write_all(b"EHLO example.com\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send STARTTLS command
    stream.write_all(b"STARTTLS\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Upgrade the connection to TLS
    let connector = TlsConnector::new()?;
    let mut stream = connector.connect("smtp.gmail.com", stream)?;

    // Re-send EHLO after STARTTLS
    stream.write_all(b"EHLO example.com\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Authenticate using AUTH LOGIN
    stream.write_all(b"AUTH LOGIN\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send base64-encoded username (your Gmail address)
    let username = encode(&config.sender);
    stream.write_all(format!("{}\r\n", username).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send base64-encoded password (from .env file)
    let password_encoded = encode(&password);
    stream.write_all(format!("{}\r\n", password_encoded).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send MAIL FROM command
    stream.write_all(format!("MAIL FROM:<{}>\r\n", config.sender).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send RCPT TO command
    stream.write_all(format!("RCPT TO:<{}>\r\n", config.receiver).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send DATA command
    stream.write_all(b"DATA\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send email headers and body
    let email_body = format!(
        "From: {}\r\n\
         To: {}\r\n\
         Subject: Hello from Rust!\r\n\
         \r\n\
         This is a test email sent using raw SMTP in Rust.\r\n.\r\n",
        config.sender, config.receiver
    );
    stream.write_all(email_body.as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send QUIT command
    stream.write_all(b"QUIT\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    Ok(())
}
use crate::manager::AttendanceManager;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use config::Config;
use diesel::QueryResult;
use dotenv::dotenv;
use native_tls::TlsConnector;
use serde::Deserialize;
use std::fs;
use std::net::TcpStream;
use std::time::Duration;
use std::{
    env,
    io::{self, Read, Write},
};

#[derive(Debug, Deserialize)]
struct SmtpConfig {
    smtp: SmtpDetails,
}

#[derive(Debug, Deserialize)]
struct SmtpDetails {
    sender: String,
    cc: String,
    email_subject: String,
    email_body_path: String,
}

fn load_config() -> Result<SmtpDetails, Box<dyn std::error::Error>> {
    // Load from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;
    let smtp_config: SmtpConfig = settings.try_deserialize()?;

    Ok(smtp_config.smtp)
}

fn parse_recipients(recipients: &str) -> Vec<String> {
    recipients
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

fn read_email_body(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = fs::read_to_string(file_path)?;
    Ok(body)
}

pub fn send_mail(recipients: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    dotenv().ok();

    // Get password from environment variable
    let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    // Create all_recipients vector including CC
    let mut all_recipients = recipients.to_vec();
    all_recipients.extend(parse_recipients(&config.cc));
    println!("{:?}", all_recipients);

    // Connect to the SMTP server (e.g., Gmail's SMTP server)
    println!("here");
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

    // Send base64-encoded username
    let username = BASE64.encode(&config.sender);
    stream.write_all(format!("{}\r\n", username).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send base64-encoded password
    let password_encoded = BASE64.encode(&password);
    stream.write_all(format!("{}\r\n", password_encoded).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send MAIL FROM command
    stream.write_all(format!("MAIL FROM:<{}>\r\n", config.sender).as_bytes())?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send RCPT TO commands for all recipients
    for recipient in all_recipients {
        stream.write_all(format!("RCPT TO:<{}>\r\n", recipient).as_bytes())?;
        stream.read(&mut response)?;
        println!("Server: {}", String::from_utf8_lossy(&response));
    }

    // Send DATA command
    stream.write_all(b"DATA\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send email headers and body
    let email_body = read_email_body(&config.email_body_path)?;
    let email_headers = format!(
        "From: {}\r\n\
         To: undisclosed-recipients\r\n\
         CC: {}\r\n\
         Subject: {}\r\n\
         Content-Type: text/html; charset=UTF-8\r\n\
         \r\n",
        config.sender, config.cc, config.email_subject
    );

    stream.write_all(email_headers.as_bytes())?;
    stream.write_all(email_body.as_bytes())?;
    stream.write_all(b"\r\n.\r\n")?; // End of email
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    // Send QUIT command
    stream.write_all(b"QUIT\r\n")?;
    stream.read(&mut response)?;
    println!("Server: {}", String::from_utf8_lossy(&response));

    Ok(())
}

/// Emails students who have more than the specified number of absences after a given week.
pub fn email_absentees(after_week: i32, min_absences: i32) -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();
    let roster = manager.get_roster()?;

    // Get all attendance records in one batch using transaction
    let mut absentees = Vec::new();
    let mut recipient_emails = Vec::new();
    for student in roster {
        let attendance = manager.get_student_attendance(&student.id)?;
        let absences = attendance
            .absent
            .iter()
            .filter(|(week, _)| *week >= after_week)
            .count();

        if absences >= min_absences as usize {
            recipient_emails.push(student.email.clone());
            absentees.push((student, absences));
        }
    }

    // Display results
    println!(
        "\nStudents with {} or more absences after week {}:",
        min_absences, after_week
    );
    println!("{:<30} {:<30} {:<10}", "Name", "Email", "Absences");
    println!("{}", "-".repeat(70));

    for (student, absences) in &absentees {
        println!(
            "{:<30} {:<30} {:<10}",
            format!("{} {}", student.first_name, student.last_name),
            student.email,
            absences
        );
    }

    if absentees.is_empty() {
        println!(
            "\nNo students found with {} or more absences after week {}.",
            min_absences, after_week
        );
        return Ok(());
    }

    // Ask for confirmation
    print!("\nWould you like to email these students? [y/N] ");
    if let Err(e) = io::stdout().flush() {
        eprintln!("Error flushing stdout: {}", e);
        return Ok(());
    }

    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Error reading input: {}", e);
        return Ok(());
    }

    if input.trim().to_lowercase() != "y" {
        println!("Operation cancelled.");
        return Ok(());
    }

    // Send emails
    println!("\nSending emails...");
    send_mail(&recipient_emails);
    println!("Done");

    Ok(())
}

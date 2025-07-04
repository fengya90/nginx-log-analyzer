use lettre::{Message, SmtpTransport, Transport, transport::smtp::authentication::Credentials};
use lettre::message::{Mailbox, MultiPart, SinglePart};
use pulldown_cmark::{Parser, Options, html};

const GITHUB_TABLE_CSS: &str = r#"
<style>
.markdown-body table {
  border-collapse: collapse;
  border-spacing: 0;
  width: 100%;
  font-size: 14px;
}
.markdown-body th, .markdown-body td {
  border: 1px solid #dfe2e5;
  padding: 6px 13px;
}
.markdown-body tr {
  background-color: #fff;
  border-top: 1px solid #c6cbd1;
}
.markdown-body tr:nth-child(2n) {
  background-color: #f6f8fa;
}
.markdown-body th {
  background-color: #f6f8fa;
  font-weight: 600;
}
</style>
"#;

pub fn send_mail(
    smtp_host: &str,
    sender: &str,
    password: &str,
    recipients: &[String],
    subject: &str,
    body: String, // markdown
) -> Result<(), Box<dyn std::error::Error>> {
    let creds = Credentials::new(sender.to_string(), password.to_string());

    let recipient_list: Vec<Mailbox> = recipients
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    // markdown 转 html
    let parser = Parser::new_ext(&body, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // 包裹 GitHub 风格表格样式
    let html_with_style = format!(
        "{}<div class=\"markdown-body\">{}</div>",
        GITHUB_TABLE_CSS,
        html_output
    );

    let mut email_builder = Message::builder()
        .from(sender.parse()?);
    for recipient in &recipient_list {
        email_builder = email_builder.to(recipient.clone());
    }

    let email = email_builder
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(body.clone()))
                .singlepart(SinglePart::html(html_with_style))
        )?;

    let mailer = SmtpTransport::relay(smtp_host)?
        .credentials(creds)
        .build();

    let result = mailer.send(&email);

    if result.is_ok() {
        println!("Email sent successfully!");
        Ok(())
    } else {
        println!("Could not send email: {:?}", result);
        Err("Send failed".into())
    }
} 
/****
 * Module: Terminal Logger
 * Chức năng: Ghi log đa sắc trên Terminal với cấu trúc dạng cây và bắt sự kiện từ các macro của Tracing.
 ****/
use chrono::Local;
use colored::{Color, Colorize};
use textwrap::{fill, Options};
use terminal_size::{Width, terminal_size};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::DefaultFields, FormattedFields},
    layer::Context,
    Layer,
};

pub fn hash_string_to_color(s: &str) -> Color {
    let palette = [
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::White,
        Color::Magenta,
        Color::Cyan,
    ];
    let mut hash: usize = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33) ^ (b as usize);
    }
    palette[hash % palette.len()]
}

pub struct ColorTerminalLayer;

impl<S> Layer<S> for ColorTerminalLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let time = Local::now().format("%H:%M:%S%.3f").to_string().bright_black();
        let file = event.metadata().file().unwrap_or("unknown");
        let line = event.metadata().line().unwrap_or(0);
        let location = format!("{}:{}", file, line).bright_black();

        let mut span_context = String::from("App{...}");
        if let Some(span) = ctx.lookup_current() {
            let root_span = span.scope().from_root().next().unwrap_or(span);
            let ext = root_span.extensions();
            if let Some(fields) = ext.get::<FormattedFields<DefaultFields>>() {
                let clean_fields = fields.to_string().replace("\"", "");
                span_context = format!("{}{{{}}}", root_span.name(), clean_fields);
            }
        }
        let colored_span = span_context.cyan();

        let raw_category = format!("[{}]", visitor.category);
        let padded_category = format!("{:<10}", raw_category);

        let (icon, category_tag) = match visitor.category.as_str() {
            "System" => ("⚙️ ", padded_category.bright_blue().bold()),
            "Task" => ("🚀", padded_category.cyan().bold()), 
            "Warning" | "Warn" => ("⚠️ ", padded_category.bright_yellow().bold()),
            "Complete" => ("✅", padded_category.green().bold()),
            "Error" => ("❌", padded_category.red().bold()),
            "Database" => ("🗄️ ", padded_category.bright_magenta().bold()),
            other_category => {
                if let Some(custom_color) = visitor.color.as_deref() {
                    ("⚡", padded_category.color(custom_color).bold())
                } else {
                    let auto_color = hash_string_to_color(other_category);
                    ("⚡", padded_category.color(auto_color).bold())
                }
            }
        };

        let mut lines = Vec::new();
        lines.push(format!("\n{} │ {}", time, colored_span));
        
        // 🚀 THUẬT TOÁN TỰ ĐỘNG THỤT LỀ (WORD WRAP) CHO TERMINAL
        // 1. Tính toán chiều rộng Terminal hiện tại (Mặc định 100 nếu không lấy được)
        let term_width = if let Some((Width(w), _)) = terminal_size() {
            w as usize
        } else {
            100
        };

        // 2. Tính toán độ dài phần tiền tố (Prefix) để lề không bị lệch
        let prefix_plain = format!("├── {} [{}] ", icon, visitor.category); 
        let prefix_len = prefix_plain.chars().count();
        
        // Đệm ký tự `│` ở đầu thay vì toàn bộ là khoảng trắng để nối cây thư mục
        let indent_spaces = format!("│{}", " ".repeat(prefix_len.saturating_sub(1)));
        let prefix_colored = format!("├── {} {} ", icon, category_tag);

        // 3. Cấu hình Wrap cho tin nhắn chính
        let max_text_width = term_width.saturating_sub(prefix_len).max(20);
        let wrap_options = Options::new(max_text_width)
            .initial_indent("") // Dòng đầu tiên đã có prefix ở ngoài
            .subsequent_indent(&indent_spaces); // Các dòng sau tự lùi vào đúng bằng prefix

        // 4. Wrap tin nhắn
        let wrapped_msg = fill(&visitor.message, &wrap_options);
        
        // 5. Nối chuỗi
        // Chỉ dùng prefix_colored ở dòng đầu, các dòng sau textwrap đã tự thêm indent
        if wrapped_msg.contains('\n') {
            lines.push(format!("{}{}", prefix_colored, wrapped_msg));
        } else {
            lines.push(format!("{}{}", prefix_colored, visitor.message));
        }

        if let Some(code) = visitor.status {
            let reason_str = visitor.reason.as_deref().unwrap_or("Unknown");
            let latency_text = match visitor.latency_ms {
                Some(ms) => format!(" │ {}ms", ms),
                None => String::new(),
            };

            let status_text = format!("({} {}){}", code, reason_str, latency_text);
            let colored_status = match code {
                200..=299 => status_text.green(),
                300..=399 => status_text.cyan(),
                400..=499 => status_text.bright_yellow(),
                500..=599 => status_text.red().bold(),
                _ => status_text.white(),
            };
            lines.push(format!("├── {}", colored_status));
        }

        if !visitor.extras.is_empty() {
            let ctx_str = format!("Context: {{{}}}", visitor.extras.join(", "));
            // Wrap luôn cả đoạn Context
            let wrapped_ctx = fill(&ctx_str, &wrap_options);
            
            if wrapped_ctx.contains('\n') {
                lines.push(format!("├── {}", wrapped_ctx).bright_magenta().to_string());
            } else {
                lines.push(format!("├── {}", ctx_str).bright_magenta().to_string());
            }
        }

        lines.push(format!("└── [{}]", location));
        println!("{}", lines.join("\n"));
    }
}

#[derive(Default)]
pub struct LogVisitor {
    category: String,
    message: String,
    status: Option<u16>,
    reason: Option<String>,
    latency_ms: Option<u128>,
    color: Option<String>,
    extras: Vec<String>,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "message" => self.message = format!("{:?}", value).trim_matches('"').to_string(),
            "category" => self.category = format!("{:?}", value).trim_matches('"').to_string(),
            "reason" => self.reason = Some(format!("{:?}", value).trim_matches('"').to_string()),
            "color" => self.color = Some(format!("{:?}", value).trim_matches('"').to_string()), 
            "status" => {
                if let Ok(code) = format!("{:?}", value).parse::<u16>() {
                    self.status = Some(code);
                }
            },
            "latency_ms" => {
                if let Ok(ms) = format!("{:?}", value).parse::<u128>() {
                    self.latency_ms = Some(ms);
                }
            },
            _ => self.extras.push(format!("{}={:?}", field.name(), value)),
        }
    }
    
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "status" => self.status = Some(value as u16),
            "latency_ms" => self.latency_ms = Some(value as u128),
            _ => self.extras.push(format!("{}={}", field.name(), value)),
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        match field.name() {
            "status" => self.status = Some(value as u16),
            "latency_ms" => self.latency_ms = Some(value as u128),
            _ => self.extras.push(format!("{}={}", field.name(), value)),
        }
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        if field.name() == "latency_ms" { self.latency_ms = Some(value); }
        else { self.extras.push(format!("{}={}", field.name(), value)); }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "category" => self.category = value.to_string(),
            "reason" => self.reason = Some(value.to_string()),
            "color" => self.color = Some(value.to_string()), 
            _ => self.extras.push(format!("{}={}", field.name(), value)),
        }
    }
}

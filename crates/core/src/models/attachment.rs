//! Attachment and embed models for messages

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A file attached to a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    /// Content hash for deduplication and integrity
    pub hash: String,
    /// Image/video width (if applicable)
    pub width: Option<u32>,
    /// Image/video height (if applicable)
    pub height: Option<u32>,
    pub created_at: DateTime<Utc>,
}

impl Attachment {
    pub fn new(message_id: Uuid, filename: String, size_bytes: u64, hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            filename,
            size_bytes,
            content_type: None,
            hash,
            width: None,
            height: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Whether this attachment is an image
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    /// Whether this attachment is a video
    pub fn is_video(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|ct| ct.starts_with("video/"))
            .unwrap_or(false)
    }

    /// Format size for display
    pub fn format_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else if self.size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// An embed within a message (link preview, rich content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    pub id: Uuid,
    pub message_id: Uuid,
    pub embed_type: EmbedType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub footer_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Type of embed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbedType {
    /// Auto-generated link preview
    Link,
    /// Rich embed (from webhooks/bots)
    Rich,
    /// Image embed
    Image,
    /// Video embed
    Video,
    /// Article embed
    Article,
}

impl EmbedType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmbedType::Link => "link",
            EmbedType::Rich => "rich",
            EmbedType::Image => "image",
            EmbedType::Video => "video",
            EmbedType::Article => "article",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "link" => EmbedType::Link,
            "rich" => EmbedType::Rich,
            "image" => EmbedType::Image,
            "video" => EmbedType::Video,
            "article" => EmbedType::Article,
            _ => EmbedType::Link,
        }
    }
}

impl Embed {
    pub fn new_link(message_id: Uuid, url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            embed_type: EmbedType::Link,
            title: None,
            description: None,
            url: Some(url),
            color: None,
            thumbnail_url: None,
            image_url: None,
            author_name: None,
            author_url: None,
            footer_text: None,
            created_at: Utc::now(),
        }
    }

    pub fn new_rich(message_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            embed_type: EmbedType::Rich,
            title: None,
            description: None,
            url: None,
            color: None,
            thumbnail_url: None,
            image_url: None,
            author_name: None,
            author_url: None,
            footer_text: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_thumbnail(mut self, url: String) -> Self {
        self.thumbnail_url = Some(url);
        self
    }

    pub fn with_image(mut self, url: String) -> Self {
        self.image_url = Some(url);
        self
    }
}
